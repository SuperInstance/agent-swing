//! Straight rhythms vs swung rhythms. Shows how swing timing creates groove.
//! Prints timing grids for multiple feels and patterns.

use agent_swing::*;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          SWING DEMO — Timing Is Everything                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let bpm = 120;
    let ms_per_beat = 60_000 / bpm;
    println!("Tempo: {} BPM ({}/beat)\n", bpm, ms_per_beat);

    // Compare straight vs swing vs hard swing timing grids
    let feels = [
        (SwingFeel::straight(), "Straight (0.50)"),
        (SwingFeel::swing(),    "Swing (0.66)"),
        (SwingFeel::hard_swing(),"Hard Swing (0.75)"),
    ];

    println!("━━━ Timing Grid: 8th Notes ━━━");
    println!("  Each beat shown as: downbeat + upbeat ( swung )\n");

    for (feel, name) in &feels {
        println!("  {} — groove factor: {:.2}", name, feel.groove_factor());
        print!("    ");
        for beat in 0..8 {
            let down = beat as f64;
            let up = beat as f64 + feel.ratio;
            print!("│{:>5.2} {:>5.2}", down, up);
        }
        println!("│");
    }
    println!();

    // Groove patterns
    println!("━━━ Groove Patterns ━━━\n");
    let patterns = [
        ("Swing Basic", GroovePattern::swing_basic()),
        ("Jazz Ride",   GroovePattern::jazz_ride()),
        ("Funk",        GroovePattern::funk()),
        ("Bossa Nova",  GroovePattern::bossa_nova()),
    ];

    for (name, groove) in &patterns {
        println!("  {} [{} steps]", name, groove.len());
        print!("    Pattern: ");
        let mut pattern_display = Vec::new();
        for &t in &groove.pattern {
            let sym = match t {
                 1 => "▸ PUSH  ",
                 0 => "· GHOST ",
                -1 => "◂ PULL  ",
                _  => "  ???   ",
            };
            pattern_display.push(sym);
        }
        println!("{}", pattern_display.join("|"));
        println!("    Density: {:.0}%  Syncopation: {:.2}",
            groove.density() * 100.0, groove.syncopation());
        println!();
    }

    // Schedule playback for each pattern
    println!("━━━ Scheduled Timeline (Swing Feel, 120 BPM) ━━━\n");

    for (name, groove) in &patterns {
        println!("  {}:", name);
        let mut sched = SwingScheduler::new(bpm, groove.clone());
        sched.feel = SwingFeel::swing();
        let timeline = sched.schedule(8);

        let mut abs_time = 0u64;
        for (i, (action, offset)) in timeline.iter().enumerate() {
            abs_time += offset;
            let icon = match action {
                TritAction::Push     => "▸",
                TritAction::GhostNote => "·",
                TritAction::PullBack => "◂",
            };
            let label = match action {
                TritAction::Push     => "PUSH ",
                TritAction::GhostNote => "GHOST",
                TritAction::PullBack => "PULL ",
            };
            println!("    beat {:>2}: {} {}  offset={:>4}ms  abs_time={:>5}ms",
                i, icon, label, offset, abs_time);
        }
        println!();
    }

    // Swing clock comparison
    println!("━━━ Swing Clock — Beat Timings ━━━\n");
    for (feel, name) in &feels {
        println!("  {}:", name);
        let mut clock = SwingClock::new(bpm);
        clock.feel = *feel;
        print!("    Beats:  ");
        for _ in 0..8 {
            let ms = clock.tick();
            print!("{:>5}ms ", ms);
        }
        println!("  total: {}ms", clock.elapsed());
    }
    println!();

    // Syncopation analysis
    println!("━━━ Syncopation Analysis ━━━\n");
    let detector = SyncopationDetector::new(8);

    let sequences: [(&str, Vec<TritAction>); 4] = [
        ("Straight 8ths", vec![TritAction::Push; 8]),
        ("Swing Basic",   (0..8).map(|i| if i % 2 == 0 { TritAction::Push } else { TritAction::GhostNote }).collect()),
        ("Reggae",        vec![
            TritAction::GhostNote, TritAction::Push,
            TritAction::GhostNote, TritAction::Push,
            TritAction::PullBack, TritAction::Push,
            TritAction::GhostNote, TritAction::Push,
        ]),
        ("Max Syncopation", vec![
            TritAction::GhostNote, TritAction::Push,
            TritAction::PullBack, TritAction::Push,
            TritAction::GhostNote, TritAction::Push,
            TritAction::PullBack, TritAction::Push,
        ]),
    ];

    for (name, actions) in &sequences {
        let sync = detector.analyze(actions);
        let pocket = detector.in_the_pocket(actions);
        let weak = detector.has_weak_beat_activity(actions);

        let bar_len = (sync * 30.0) as usize;
        let bar: String = "█".repeat(bar_len);

        let pocket_icon = if pocket { " 🎯 IN THE POCKET" } else { "" };
        println!("  {:<18} sync={:.2} {} weak-beats={}{}", name, sync, bar, weak, pocket_icon);
    }
}
