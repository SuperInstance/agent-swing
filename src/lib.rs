//! # agent-swing
//!
//! Swing rhythm applied to agent scheduling. Instead of rigid grid-based
//! execution, agents run on a swung timeline — slightly off the beat —
//! producing a more natural, groovy flow. 50% = straight, 66% = swing,
//! 75% = hard swing.

use std::collections::HashMap;

/// Triplet ratio that defines the swing feel.
///
/// - 0.50 = straight (equal subdivisions)
/// - 0.66 = standard swing
/// - 0.75 = hard swing (dotted-note feel)
#[derive(Debug, Clone, PartialEq)]
pub struct SwingFeel {
    /// Ratio of the first subdivision in a beat [0.5, 0.95].
    pub ratio: f64,
}

impl SwingFeel {
    pub fn new(ratio: f64) -> Self {
        Self { ratio: ratio.clamp(0.5, 0.95) }
    }

    pub fn straight() -> Self { Self { ratio: 0.5 } }
    pub fn swing() -> Self { Self { ratio: 0.66 } }
    pub fn hard_swing() -> Self { Self { ratio: 0.75 } }

    /// Given a beat duration in ms, return the durations of the two subdivisions.
    pub fn subdivide(&self, beat_ms: u64) -> (u64, u64) {
        let first = (beat_ms as f64 * self.ratio) as u64;
        let second = beat_ms - first;
        (first, second)
    }

    /// Given a beat index and subdivision (0 or 1), compute the offset in ms
    /// from the start of the beat.
    pub fn subdivision_offset(&self, beat_ms: u64, subdivision: u8) -> u64 {
        let (first, _) = self.subdivide(beat_ms);
        match subdivision {
            0 => 0,
            1 => first,
            _ => beat_ms,
        }
    }

    /// How "swung" this feel is as a 0–1 measure (0 = straight, 1 = hard).
    pub fn swing_amount(&self) -> f64 {
        (self.ratio - 0.5) / 0.45
    }
}

impl Default for SwingFeel {
    fn default() -> Self {
        Self::swing()
    }
}

/// A repeating rhythmic pattern of onsets across beats.
#[derive(Debug, Clone, PartialEq)]
pub struct GroovePattern {
    /// Number of beats per measure.
    pub beats_per_measure: u8,
    /// For each beat, whether the first and second subdivisions are active.
    /// Vec of (on_downbeat: bool, on_upbeat: bool).
    pub pattern: Vec<(bool, bool)>,
}

impl GroovePattern {
    pub fn new(beats_per_measure: u8, pattern: Vec<(bool, bool)>) -> Self {
        assert_eq!(pattern.len(), beats_per_measure as usize);
        Self { beats_per_measure, pattern }
    }

    /// Four-on-the-floor: every downbeat, no upbeats.
    pub fn four_on_the_floor() -> Self {
        Self {
            beats_per_measure: 4,
            pattern: vec![(true, false); 4],
        }
    }

    /// Classic swing pattern: downbeats + syncopated upbeats.
    pub fn classic_swing() -> Self {
        Self {
            beats_per_measure: 4,
            pattern: vec![
                (true, false),
                (true, true),
                (true, false),
                (true, true),
            ],
        }
    }

    /// Generate onset times in ms for one measure, given beat duration and swing feel.
    pub fn onsets(&self, beat_ms: u64, feel: &SwingFeel) -> Vec<u64> {
        let mut result = Vec::new();
        for (beat_idx, &(down, up)) in self.pattern.iter().enumerate() {
            let beat_start = beat_idx as u64 * beat_ms;
            if down {
                result.push(beat_start);
            }
            if up {
                let (first, _) = feel.subdivide(beat_ms);
                result.push(beat_start + first);
            }
        }
        result
    }

    /// Total duration of one measure in ms.
    pub fn measure_duration(&self, beat_ms: u64) -> u64 {
        self.beats_per_measure as u64 * beat_ms
    }

    /// Number of onsets per measure.
    pub fn onset_count(&self) -> usize {
        self.pattern.iter().map(|(d, u)| *d as usize + *u as usize).sum()
    }
}

/// Maps which agents syncopate (execute on the off-beat).
#[derive(Debug, Clone)]
pub struct SyncopationMap {
    /// agent_id → whether this agent syncopates.
    map: HashMap<String, bool>,
}

impl SyncopationMap {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn set(&mut self, agent_id: impl Into<String>, syncopates: bool) {
        self.map.insert(agent_id.into(), syncopates);
    }

    pub fn syncopates(&self, agent_id: &str) -> bool {
        self.map.get(agent_id).copied().unwrap_or(false)
    }

    /// Apply syncopation offset to a time position.
    /// Syncopating agents are shifted by the upbeat offset.
    pub fn apply(&self, agent_id: &str, base_time_ms: u64, feel: &SwingFeel, beat_ms: u64) -> u64 {
        if self.syncopates(agent_id) {
            let (first, _) = feel.subdivide(beat_ms);
            base_time_ms + first
        } else {
            base_time_ms
        }
    }

    pub fn agents(&self) -> Vec<&str> {
        self.map.keys().map(|s| s.as_str()).collect()
    }

    pub fn syncopating_agents(&self) -> Vec<&str> {
        self.map.iter().filter(|(_, s)| **s).map(|(k, _)| k.as_str()).collect()
    }
}

/// A time source with swing feel. Instead of a linear tick, it produces
/// swung ticks that follow the groove.
#[derive(Debug, Clone)]
pub struct SwingClock {
    /// Beat duration in ms.
    pub beat_ms: u64,
    /// Swing feel.
    pub feel: SwingFeel,
    /// Current beat index.
    current_beat: u64,
    /// Current subdivision (0 = downbeat, 1 = upbeat).
    current_sub: u8,
    /// Total elapsed ms (wall clock).
    elapsed_ms: u64,
}

impl SwingClock {
    pub fn new(beat_ms: u64, feel: SwingFeel) -> Self {
        Self {
            beat_ms,
            feel,
            current_beat: 0,
            current_sub: 0,
            elapsed_ms: 0,
        }
    }

    /// Advance to the next tick and return the absolute time in ms.
    pub fn tick(&mut self) -> u64 {
        let (first, second) = self.feel.subdivide(self.beat_ms);
        let offset = match self.current_sub {
            0 => {
                let t = self.current_beat * self.beat_ms;
                self.advance(first);
                t
            }
            1 => {
                let t = self.current_beat * self.beat_ms + first;
                self.advance(second);
                t
            }
            _ => 0,
        };

        self.current_sub = if self.current_sub == 0 { 1 } else {
            self.current_sub = 0;
            self.current_beat += 1;
            0
        };

        offset
    }

    fn advance(&mut self, dt: u64) {
        self.elapsed_ms += dt;
    }

    /// Get a series of tick times for `n` ticks.
    pub fn schedule(&mut self, n: usize) -> Vec<u64> {
        (0..n).map(|_| self.tick()).collect()
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed_ms
    }

    pub fn current_beat(&self) -> u64 {
        self.current_beat
    }

    pub fn reset(&mut self) {
        self.current_beat = 0;
        self.current_sub = 0;
        self.elapsed_ms = 0;
    }
}

/// A scheduler that executes tasks with swing timing.
#[derive(Debug, Clone)]
pub struct SwingScheduler {
    pub feel: SwingFeel,
    pub beat_ms: u64,
    pub groove: GroovePattern,
    pub syncopation: SyncopationMap,
}

impl SwingScheduler {
    pub fn new(feel: SwingFeel, beat_ms: u64) -> Self {
        Self {
            feel,
            beat_ms,
            groove: GroovePattern::classic_swing(),
            syncopation: SyncopationMap::new(),
        }
    }

    /// Schedule `n` measures of execution for an agent, returning tick times.
    pub fn schedule_agent(&self, agent_id: &str, measures: u8) -> Vec<u64> {
        let mut times = Vec::new();
        let measure_ms = self.groove.measure_duration(self.beat_ms);

        for m in 0..measures {
            let measure_start = m as u64 * measure_ms;
            let onsets = self.groove.onsets(self.beat_ms, &self.feel);

            for onset in onsets {
                let scheduled = self.syncopation.apply(
                    agent_id, measure_start + onset, &self.feel, self.beat_ms,
                );
                times.push(scheduled);
            }
        }

        times.sort();
        times
    }

    /// Schedule multiple agents across measures.
    pub fn schedule_all(&self, agents: &[&str], measures: u8) -> HashMap<String, Vec<u64>> {
        agents.iter().map(|&id| {
            (id.to_string(), self.schedule_agent(id, measures))
        }).collect()
    }

    /// Compute the swing offset for a given beat and subdivision.
    pub fn swing_offset(&self, beat: u64, subdivision: u8) -> u64 {
        let beat_start = beat * self.beat_ms;
        self.feel.subdivision_offset(self.beat_ms, subdivision) + beat_start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_straight_feel() {
        let feel = SwingFeel::straight();
        let (first, second) = feel.subdivide(1000);
        assert_eq!(first, 500);
        assert_eq!(second, 500);
    }

    #[test]
    fn test_swing_feel() {
        let feel = SwingFeel::swing();
        let (first, second) = feel.subdivide(1000);
        assert_eq!(first, 660);
        assert_eq!(second, 340);
    }

    #[test]
    fn test_hard_swing_feel() {
        let feel = SwingFeel::hard_swing();
        let (first, second) = feel.subdivide(1000);
        assert_eq!(first, 750);
        assert_eq!(second, 250);
    }

    #[test]
    fn test_swing_amount() {
        let straight = SwingFeel::straight();
        assert!((straight.swing_amount() - 0.0).abs() < f64::EPSILON);

        let hard = SwingFeel::hard_swing();
        assert!(hard.swing_amount() > 0.5);
    }

    #[test]
    fn test_feel_clamping() {
        let too_low = SwingFeel::new(0.1);
        assert!((too_low.ratio - 0.5).abs() < f64::EPSILON);

        let too_high = SwingFeel::new(1.5);
        assert!((too_high.ratio - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_subdivision_offset() {
        let feel = SwingFeel::swing();
        assert_eq!(feel.subdivision_offset(1000, 0), 0);
        assert_eq!(feel.subdivision_offset(1000, 1), 660);
    }

    #[test]
    fn test_groove_four_on_floor() {
        let groove = GroovePattern::four_on_the_floor();
        let feel = SwingFeel::straight();
        let onsets = groove.onsets(500, &feel);
        assert_eq!(onsets, vec![0, 500, 1000, 1500]);
    }

    #[test]
    fn test_groove_classic_swing() {
        let groove = GroovePattern::classic_swing();
        let feel = SwingFeel::swing();
        let onsets = groove.onsets(500, &feel);
        // Beat 0: down only → 0
        // Beat 1: down + up → 500, 830
        // Beat 2: down only → 1000
        // Beat 3: down + up → 1500, 1830
        assert_eq!(onsets.len(), 6);
        assert_eq!(onsets[0], 0);
        assert_eq!(onsets[1], 500);
        assert_eq!(onsets[2], 830); // 500 + 330
        assert_eq!(onsets[3], 1000);
        assert_eq!(onsets[4], 1500);
        assert_eq!(onsets[5], 1830);
    }

    #[test]
    fn test_measure_duration() {
        let groove = GroovePattern::four_on_the_floor();
        assert_eq!(groove.measure_duration(500), 2000);
    }

    #[test]
    fn test_onset_count() {
        let groove = GroovePattern::four_on_the_floor();
        assert_eq!(groove.onset_count(), 4);

        let swing = GroovePattern::classic_swing();
        assert_eq!(swing.onset_count(), 6);
    }

    #[test]
    fn test_syncopation_map() {
        let mut map = SyncopationMap::new();
        map.set("agent-a", false);
        map.set("agent-b", true);

        assert!(!map.syncopates("agent-a"));
        assert!(map.syncopates("agent-b"));
        assert!(!map.syncopates("unknown")); // default false
    }

    #[test]
    fn test_syncopation_apply() {
        let mut map = SyncopationMap::new();
        map.set("sync-agent", true);
        let feel = SwingFeel::swing();

        let normal = map.apply("sync-agent", 1000, &feel, 1000);
        let shifted = 1000 + 660;
        assert_eq!(normal, shifted);

        map.set("straight-agent", false);
        let no_shift = map.apply("straight-agent", 1000, &feel, 1000);
        assert_eq!(no_shift, 1000);
    }

    #[test]
    fn test_swing_clock_straight() {
        let feel = SwingFeel::straight();
        let mut clock = SwingClock::new(1000, feel);
        let times = clock.schedule(4);

        // Straight: 0, 500, 1000, 1500
        assert_eq!(times[0], 0);
        assert_eq!(times[1], 500);
        assert_eq!(times[2], 1000);
        assert_eq!(times[3], 1500);
    }

    #[test]
    fn test_swing_clock_swing() {
        let feel = SwingFeel::swing();
        let mut clock = SwingClock::new(1000, feel);
        let times = clock.schedule(4);

        // Swing: 0, 660, 1000, 1660
        assert_eq!(times[0], 0);
        assert_eq!(times[1], 660);
        assert_eq!(times[2], 1000);
        assert_eq!(times[3], 1660);
    }

    #[test]
    fn test_swing_clock_elapsed() {
        let feel = SwingFeel::straight();
        let mut clock = SwingClock::new(1000, feel);
        clock.schedule(2); // 500 + 500 = 1000 elapsed
        assert_eq!(clock.elapsed_ms(), 1000);
    }

    #[test]
    fn test_swing_clock_reset() {
        let feel = SwingFeel::swing();
        let mut clock = SwingClock::new(1000, feel);
        clock.schedule(4);
        assert!(clock.elapsed_ms() > 0);

        clock.reset();
        assert_eq!(clock.elapsed_ms(), 0);
        assert_eq!(clock.current_beat(), 0);
    }

    #[test]
    fn test_swing_scheduler_basic() {
        let feel = SwingFeel::swing();
        let scheduler = SwingScheduler::new(feel, 500);
        let times = scheduler.schedule_agent("agent-a", 1);

        // 1 measure of classic swing = 6 onsets
        assert_eq!(times.len(), 6);
        assert_eq!(times[0], 0);
    }

    #[test]
    fn test_swing_scheduler_with_syncopation() {
        let feel = SwingFeel::swing();
        let mut scheduler = SwingScheduler::new(feel, 500);
        scheduler.syncopation.set("agent-a", true);

        let normal = {
            let s = SwingScheduler::new(SwingFeel::swing(), 500);
            s.schedule_agent("agent-a", 1)
        };
        let syncopated = scheduler.schedule_agent("agent-a", 1);

        // Syncopated times should be shifted
        assert_ne!(normal, syncopated);
    }

    #[test]
    fn test_swing_scheduler_multi_agent() {
        let feel = SwingFeel::swing();
        let mut scheduler = SwingScheduler::new(feel, 500);
        scheduler.syncopation.set("agent-a", false);
        scheduler.syncopation.set("agent-b", true);

        let schedule = scheduler.schedule_all(&["agent-a", "agent-b"], 1);
        assert!(schedule.contains_key("agent-a"));
        assert!(schedule.contains_key("agent-b"));
        assert_ne!(schedule["agent-a"], schedule["agent-b"]);
    }

    #[test]
    fn test_swing_offset_computation() {
        let scheduler = SwingScheduler::new(SwingFeel::swing(), 1000);
        let offset_0 = scheduler.swing_offset(0, 0);
        assert_eq!(offset_0, 0);

        let offset_1 = scheduler.swing_offset(0, 1);
        assert_eq!(offset_1, 660);

        let offset_beat2 = scheduler.swing_offset(2, 0);
        assert_eq!(offset_beat2, 2000);
    }

    #[test]
    fn test_feel_parameter_effect() {
        let beat = 1000u64;
        let straight = SwingFeel::straight();
        let swing = SwingFeel::swing();
        let hard = SwingFeel::hard_swing();

        let (s1, s2) = straight.subdivide(beat);
        let (sw1, sw2) = swing.subdivide(beat);
        let (h1, h2) = hard.subdivide(beat);

        // As swing increases, first subdivision gets longer, second gets shorter
        assert!(s1 < sw1);
        assert!(sw1 < h1);
        assert!(s2 > sw2);
        assert!(sw2 > h2);

        // They always sum to the beat
        assert_eq!(s1 + s2, beat);
        assert_eq!(sw1 + sw2, beat);
        assert_eq!(h1 + h2, beat);
    }

    #[test]
    fn test_groove_repetition() {
        let groove = GroovePattern::classic_swing();
        let feel = SwingFeel::swing();
        let beat = 500u64;

        let onsets_m1 = groove.onsets(beat, &feel);
        let measure_ms = groove.measure_duration(beat);

        // Second measure onsets should be shifted by measure_ms
        let mut onsets_m2: Vec<u64> = onsets_m1.iter().map(|&o| o + measure_ms).collect();

        let all: Vec<u64> = onsets_m1.into_iter().chain(onsets_m2.drain(..)).collect();
        assert_eq!(all.len(), 12); // 6 per measure × 2
    }
}
