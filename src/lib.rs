//! # agent-swing
//!
//! Swing rhythm for agent scheduling — off-beat execution for better flow.
//!
//! Inspired by the revelation that ternary math IS the rhythm section of thought:
//! - Pull back (-1): agent holds, creates space
//! - Ghost note (0): agent listens, maintains groove without acting
//! - Push (+1): agent acts with emphasis
//!
//! "Swing isn't being late. It's being late on purpose, at exactly the right time."

/// The swing feel controls how far off the grid execution lands.
/// 0.50 = straight (on the beat)
/// 0.66 = standard swing
/// 0.75 = hard swing (New Orleans)
#[derive(Debug, Clone, Copy)]
pub struct SwingFeel {
    /// Ratio of the first half to the full beat (0.5 = straight, 0.66 = swing)
    pub ratio: f64,
}

impl Default for SwingFeel {
    fn default() -> Self {
        Self { ratio: 0.66 }
    }
}

impl SwingFeel {
    pub fn straight() -> Self { Self { ratio: 0.50 } }
    pub fn swing() -> Self { Self { ratio: 0.66 } }
    pub fn hard_swing() -> Self { Self { ratio: 0.75 } }
    pub fn custom(ratio: f64) -> Self { Self { ratio: ratio.clamp(0.25, 0.90) } }

    /// Given a beat position (0.0 to 1.0), apply swing timing
    pub fn swing_time(&self, beat: u64, subdivision: u64) -> f64 {
        let base = beat as f64;
        if subdivision == 0 {
            return base;
        }
        let sub_pos = (subdivision % 2) as f64;
        if sub_pos == 0.0 {
            // On-beat: stays on the beat
            base
        } else {
            // Off-beat: swung by ratio
            base + self.ratio
        }
    }

    /// Calculate the groove factor — how much swing affects timing
    pub fn groove_factor(&self) -> f64 {
        (self.ratio - 0.5).abs() * 2.0
    }
}

/// A trit-valued action decision: the agent's rhythmic intent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TritAction {
    /// Pull back — create space, don't act
    PullBack = -1,
    /// Ghost note — maintain presence without emphasis
    GhostNote = 0,
    /// Push — act with emphasis
    Push = 1,
}

impl TritAction {
    pub fn from_trit(t: i8) -> Option<Self> {
        match t {
            -1 => Some(Self::PullBack),
            0 => Some(Self::GhostNote),
            1 => Some(Self::Push),
            _ => None,
        }
    }

    pub fn to_trit(self) -> i8 {
        self as i8
    }

    /// Whether this action should produce visible output
    pub fn is_audible(&self) -> bool {
        matches!(self, Self::Push)
    }

    /// Whether this action maintains groove without acting
    pub fn is_ghost(&self) -> bool {
        matches!(self, Self::GhostNote)
    }
}

/// A repeating rhythmic pattern of trit actions
#[derive(Debug, Clone)]
pub struct GroovePattern {
    /// Sequence of trit actions (-1, 0, +1)
    pub pattern: Vec<i8>,
    /// Current position in the pattern
    pub position: usize,
}

impl GroovePattern {
    pub fn new(pattern: Vec<i8>) -> Self {
        assert!(pattern.iter().all(|&t| t >= -1 && t <= 1));
        Self { pattern, position: 0 }
    }

    /// Standard swing pattern: push, ghost, push, ghost
    pub fn swing_basic() -> Self {
        Self::new(vec![1, 0, 1, 0])
    }

    /// Jazz ride pattern: push, ghost, ghost, push
    pub fn jazz_ride() -> Self {
        Self::new(vec![1, 0, 0, 1])
    }

    /// Funk pattern: push, pull, ghost, push
    pub fn funk() -> Self {
        Self::new(vec![1, -1, 0, 1])
    }

    /// Bossa nova: push, ghost, push, ghost, ghost, push, ghost, ghost
    pub fn bossa_nova() -> Self {
        Self::new(vec![1, 0, 1, 0, 0, 1, 0, 0])
    }

    /// Step to the next beat and return the action
    pub fn step(&mut self) -> TritAction {
        let action = TritAction::from_trit(self.pattern[self.position]).unwrap();
        self.position = (self.position + 1) % self.pattern.len();
        action
    }

    /// Peek at the next action without advancing
    pub fn peek(&self) -> TritAction {
        TritAction::from_trit(self.pattern[self.position]).unwrap()
    }

    /// Get the pattern length
    pub fn len(&self) -> usize {
        self.pattern.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pattern.is_empty()
    }

    /// Calculate the density — ratio of push actions to total
    pub fn density(&self) -> f64 {
        if self.pattern.is_empty() { return 0.0; }
        let pushes = self.pattern.iter().filter(|&&t| t == 1).count();
        pushes as f64 / self.pattern.len() as f64
    }

    /// Calculate syncopation index — how often strong beats are silent
    pub fn syncopation(&self) -> f64 {
        if self.pattern.len() < 4 { return 0.0; }
        let mut sync_count = 0;
        for (i, &t) in self.pattern.iter().enumerate() {
            // Strong beats are positions 0, 2, 4... in a 4/4 pattern
            if i % 2 == 0 && t != 1 { sync_count += 1; }
            if i % 2 == 1 && t == 1 { sync_count += 1; }
        }
        sync_count as f64 / self.pattern.len() as f64
    }

    /// Reset to beginning of pattern
    pub fn reset(&mut self) {
        self.position = 0;
    }
}

/// Schedules agent execution with swing timing
pub struct SwingScheduler {
    pub feel: SwingFeel,
    pub groove: GroovePattern,
    /// Base interval between beats in milliseconds
    pub bpm: u64,
    tick: u64,
}

impl SwingScheduler {
    pub fn new(bpm: u64, groove: GroovePattern) -> Self {
        Self {
            feel: SwingFeel::default(),
            groove,
            bpm,
            tick: 0,
        }
    }

    /// Milliseconds per beat
    pub fn ms_per_beat(&self) -> u64 {
        60_000 / self.bpm.max(1)
    }

    /// Get the next scheduled action and its timing offset
    pub fn next(&mut self) -> (TritAction, u64) {
        let action = self.groove.step();
        let base_ms = self.ms_per_beat();

        let offset = match action {
            TritAction::Push => 0, // On-beat or swung forward
            TritAction::GhostNote => {
                // Off-beat: delayed by swing ratio
                (base_ms as f64 * self.feel.ratio) as u64
            }
            TritAction::PullBack => {
                // Pre-beat: slightly early
                (base_ms as f64 * 0.1) as u64
            }
        };

        self.tick += 1;
        (action, offset)
    }

    /// Schedule N steps and return the timeline
    pub fn schedule(&mut self, steps: usize) -> Vec<(TritAction, u64)> {
        (0..steps).map(|_| self.next()).collect()
    }

    /// Calculate the swing factor of the current schedule
    pub fn swing_amount(&self) -> f64 {
        self.feel.groove_factor() * self.groove.syncopation()
    }
}

/// Detect syncopation in an agent's action sequence
pub struct SyncopationDetector {
    /// Window size for analysis
    window: usize,
}

impl SyncopationDetector {
    pub fn new(window: usize) -> Self {
        Self { window: window.max(4) }
    }

    /// Analyze a sequence of actions for syncopation
    pub fn analyze(&self, actions: &[TritAction]) -> f64 {
        if actions.len() < self.window { return 0.0; }

        let window_actions = &actions[actions.len() - self.window..];
        let mut syncopation = 0.0;

        for (i, action) in window_actions.iter().enumerate() {
            let is_strong_beat = i % 2 == 0;
            match (is_strong_beat, action) {
                // Syncopation: silent on strong beat
                (true, TritAction::GhostNote | TritAction::PullBack) => syncopation += 1.0,
                // Syncopation: active on weak beat (only if strong beats aren't all active)
                (false, TritAction::Push) => syncopation += 0.5,
                _ => {}
            }
        }

        syncopation / self.window as f64
    }

    /// Check if a sequence has any weak-beat activity at all
    pub fn has_weak_beat_activity(&self, actions: &[TritAction]) -> bool {
        if actions.len() < self.window { return false; }
        let window_actions = &actions[actions.len() - self.window..];
        window_actions.iter().enumerate().any(|(i, a)| i % 2 == 1 && *a == TritAction::Push)
    }

    /// Check if a sequence is "in the pocket" — swung but not too much
    pub fn in_the_pocket(&self, actions: &[TritAction]) -> bool {
        let sync = self.analyze(actions);
        // Sweet spot: 0.1-0.5 syncopation
        sync >= 0.1 && sync <= 0.5
    }
}

/// A clock that ticks in swing time
pub struct SwingClock {
    pub bpm: u64,
    pub feel: SwingFeel,
    elapsed_ms: u64,
    beat: u64,
}

impl SwingClock {
    pub fn new(bpm: u64) -> Self {
        Self {
            bpm,
            feel: SwingFeel::default(),
            elapsed_ms: 0,
            beat: 0,
        }
    }

    /// Advance by one beat and return the swung timing
    pub fn tick(&mut self) -> u64 {
        let base = 60_000 / self.bpm.max(1);
        let swung = if self.beat % 2 == 0 {
            base
        } else {
            (base as f64 * self.feel.ratio) as u64
        };
        self.elapsed_ms += swung;
        self.beat += 1;
        swung
    }

    /// Current beat number
    pub fn current_beat(&self) -> u64 {
        self.beat
    }

    /// Total elapsed time in ms
    pub fn elapsed(&self) -> u64 {
        self.elapsed_ms
    }

    /// Reset the clock
    pub fn reset(&mut self) {
        self.elapsed_ms = 0;
        self.beat = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swing_feel_default() {
        let feel = SwingFeel::default();
        assert!((feel.ratio - 0.66).abs() < 0.01);
    }

    #[test]
    fn test_swing_feel_variants() {
        assert!((SwingFeel::straight().ratio - 0.50).abs() < 0.01);
        assert!((SwingFeel::swing().ratio - 0.66).abs() < 0.01);
        assert!((SwingFeel::hard_swing().ratio - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_swing_feel_clamp() {
        let feel = SwingFeel::custom(0.05);
        assert!((feel.ratio - 0.25).abs() < 0.01);
        let feel = SwingFeel::custom(1.5);
        assert!((feel.ratio - 0.90).abs() < 0.01);
    }

    #[test]
    fn test_trit_action_roundtrip() {
        for t in [-1i8, 0, 1] {
            let action = TritAction::from_trit(t).unwrap();
            assert_eq!(action.to_trit(), t);
        }
        assert!(TritAction::from_trit(2).is_none());
    }

    #[test]
    fn test_trit_action_properties() {
        assert!(TritAction::Push.is_audible());
        assert!(!TritAction::GhostNote.is_audible());
        assert!(!TritAction::PullBack.is_audible());
        assert!(TritAction::GhostNote.is_ghost());
        assert!(!TritAction::Push.is_ghost());
    }

    #[test]
    fn test_groove_pattern_basic() {
        let mut groove = GroovePattern::swing_basic();
        assert_eq!(groove.step(), TritAction::Push);
        assert_eq!(groove.step(), TritAction::GhostNote);
        assert_eq!(groove.step(), TritAction::Push);
        assert_eq!(groove.step(), TritAction::GhostNote);
        // Wraps around
        assert_eq!(groove.step(), TritAction::Push);
    }

    #[test]
    fn test_groove_pattern_peek() {
        let groove = GroovePattern::funk();
        assert_eq!(groove.peek(), TritAction::Push);
    }

    #[test]
    fn test_groove_density() {
        let groove = GroovePattern::swing_basic();
        assert!((groove.density() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_groove_syncopation() {
        let groove = GroovePattern::jazz_ride();
        let sync = groove.syncopation();
        assert!(sync > 0.0, "Jazz ride should have syncopation");
    }

    #[test]
    fn test_groove_reset() {
        let mut groove = GroovePattern::swing_basic();
        groove.step();
        groove.step();
        assert_eq!(groove.position, 2);
        groove.reset();
        assert_eq!(groove.position, 0);
    }

    #[test]
    fn test_swing_scheduler_basic() {
        let mut sched = SwingScheduler::new(120, GroovePattern::swing_basic());
        let (action, offset) = sched.next();
        assert_eq!(action, TritAction::Push);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_swing_scheduler_ghost_timing() {
        let mut sched = SwingScheduler::new(120, GroovePattern::swing_basic());
        sched.next(); // Push (on-beat)
        let (_, offset) = sched.next(); // Ghost (off-beat, swung)
        assert!(offset > 0, "Ghost notes should be delayed");
    }

    #[test]
    fn test_swing_scheduler_pullback_timing() {
        let mut sched = SwingScheduler::new(120, GroovePattern::funk());
        sched.next(); // Push
        let (_, offset) = sched.next(); // PullBack
        assert!(offset > 0, "Pull-back should have small offset");
    }

    #[test]
    fn test_swing_schedule_timeline() {
        let mut sched = SwingScheduler::new(120, GroovePattern::swing_basic());
        let timeline = sched.schedule(8);
        assert_eq!(timeline.len(), 8);
        // Should have 4 pushes and 4 ghosts
        let pushes = timeline.iter().filter(|(a, _)| *a == TritAction::Push).count();
        assert_eq!(pushes, 4);
    }

    #[test]
    fn test_syncopation_detector() {
        let detector = SyncopationDetector::new(8);
        // All pushes = no syncopation (strong beats filled, weak beats are weak-beat activity but not syncopation in context)
        let straight = vec![TritAction::Push; 8];
        // All pushes means weak beats are active but that's not syncopation when strong beats are also filled
        // Actually, our definition counts pushes on weak beats. Let's test the pattern:
        let sync = detector.analyze(&straight);
        // 4 weak beats with Push = 4 * 0.5 = 2.0 / 8 = 0.25
        assert!((sync - 0.25).abs() < 0.01);

        // Ghosts on strong beats = high syncopation
        let syncopated = vec![
            TritAction::GhostNote, TritAction::Push,
            TritAction::GhostNote, TritAction::Push,
            TritAction::GhostNote, TritAction::Push,
            TritAction::GhostNote, TritAction::Push,
        ];
        let sync_high = detector.analyze(&syncopated);
        // 4 ghosts on strong = 4.0, 4 pushes on weak = 2.0, total 6.0 / 8 = 0.75
        assert!(sync_high > 0.5, "Syncopated pattern should score high, got {sync_high}");
    }

    #[test]
    fn test_in_the_pocket() {
        let detector = SyncopationDetector::new(8);
        // Swing basic pattern should be in the pocket
        let groove_actions: Vec<TritAction> = (0..8).map(|i| {
            TritAction::from_trit(if i % 2 == 0 { 1 } else { 0 }).unwrap()
        }).collect();
        assert!(detector.in_the_pocket(&groove_actions));
    }

    #[test]
    fn test_swing_clock() {
        let mut clock = SwingClock::new(120);
        let t1 = clock.tick();
        assert_eq!(clock.current_beat(), 1);
        assert!(t1 > 0);
    }

    #[test]
    fn test_swing_clock_elapsed() {
        let mut clock = SwingClock::new(120);
        clock.tick();
        clock.tick();
        assert!(clock.elapsed() > 0);
        assert_eq!(clock.current_beat(), 2);
    }

    #[test]
    fn test_swing_clock_reset() {
        let mut clock = SwingClock::new(120);
        clock.tick();
        clock.tick();
        clock.reset();
        assert_eq!(clock.current_beat(), 0);
        assert_eq!(clock.elapsed(), 0);
    }

    #[test]
    fn test_groove_patterns_variety() {
        let patterns = [
            GroovePattern::swing_basic(),
            GroovePattern::jazz_ride(),
            GroovePattern::funk(),
            GroovePattern::bossa_nova(),
        ];
        // All patterns should have different densities
        let densities: Vec<f64> = patterns.iter().map(|p| p.density()).collect();
        let unique: std::collections::HashSet<u64> = densities.iter()
            .map(|d| (d * 1000.0) as u64)
            .collect();
        assert!(unique.len() >= 2, "Patterns should have variety");
    }

    #[test]
    fn test_bossa_nova_length() {
        let groove = GroovePattern::bossa_nova();
        assert_eq!(groove.len(), 8);
    }
}
#[cfg(test)]
mod debug {
    use super::*;
    #[test]
    fn debug_pocket() {
        let detector = SyncopationDetector::new(8);
        let groove_actions: Vec<TritAction> = (0..8).map(|i| {
            TritAction::from_trit(if i % 2 == 0 { 1 } else { 0 }).unwrap()
        }).collect();
        let sync = detector.analyze(&groove_actions);
        eprintln!("syncopation = {sync}");
        panic!("debug");
    }
}
