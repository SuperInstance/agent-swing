# agent-swing

**Swing rhythm for agent scheduling — off-beat execution for better flow.**

Straight scheduling runs everything on the grid: exactly every N milliseconds, perfectly even. But groove isn't about perfection — it's about feel. In music, swing shifts the off-beat slightly later, creating a lilting, propulsive rhythm. `agent-swing` applies this to agent scheduling: instead of rigid grid execution, agents run on a swung timeline that produces more natural, efficient flow.

## Core Concepts

### SwingFeel

The triplet ratio that defines the groove:

| Ratio | Feel | Character |
|-------|------|-----------|
| 0.50 | Straight | Even subdivision, no swing |
| 0.66 | Swing | Classic jazz/swing feel |
| 0.75 | Hard Swing | Dotted-note, pushed feel |

Given a beat of 1000ms:
- **Straight** → subdivide into 500ms + 500ms
- **Swing** → subdivide into 660ms + 340ms
- **Hard Swing** → subdivide into 750ms + 250ms

The first subdivision (downbeat) gets longer; the second (upbeat) gets shorter. That's swing.

### GroovePattern

A repeating rhythmic pattern defined per beat:
- `(true, false)` — downbeat only
- `(true, true)` — downbeat + upbeat
- `(false, true)` — upbeat only (syncopated)

Built-in patterns:
- **four_on_the_floor** — every downbeat, no upbeats
- **classic_swing** — alternating down+up beats

Patterns generate onset times when combined with a SwingFeel and beat duration.

### SyncopationMap

Tracks which agents syncopate (execute on the off-beat). Syncopating agents are shifted by the swing subdivision offset, landing them on the upbeat instead of the downbeat. This creates polyrhythmic interplay between agents.

### SwingClock

A time source with swing feel. Instead of uniform ticks, it produces swung ticks following the subdivision pattern. Each tick alternates between downbeat and upbeat timing.

### SwingScheduler

The full scheduling engine combining feel, groove, and syncopation:
- Schedule agents across measures
- Apply syncopation per-agent
- Multi-agent scheduling with different rhythmic roles

## Usage

```rust
use agent_swing::*;

// Create a swing scheduler
let feel = SwingFeel::swing();
let mut scheduler = SwingScheduler::new(feel, 500); // 500ms beats

// Assign syncopation roles
scheduler.syncopation.set("worker-a", false); // on the beat
scheduler.syncopation.set("worker-b", true);  // off the beat

// Schedule 4 measures for each agent
let schedule = scheduler.schedule_all(&["worker-a", "worker-b"], 4);
for (agent, times) in &schedule {
    println!("{}: {} executions at {:?}", agent, times.len(), times);
}

// Or use the clock directly
let mut clock = SwingClock::new(1000, SwingFeel::hard_swing());
let ticks = clock.schedule(8);
for t in &ticks {
    println!("tick at {}ms", t);
}
```

## Why Swing for Scheduling?

Straight scheduling has problems:
- **Thundering herd** — all agents fire at once, causing resource spikes
- **Predictable load** — even if load is bursty, you get rigid patterns
- **No groove** — everything feels mechanical

Swing scheduling:
- **Spreads load naturally** — upbeat agents fire slightly later, smoothing peaks
- **Creates rhythmic patterns** — agents develop complementary timing
- **Feels better** — and in practice, slightly off-grid execution often performs better because it avoids synchronized contention

The 66% swing ratio (classic jazz feel) works well for most scheduling: the upbeat lands at ~2/3 of the beat, giving the system time to process the downbeat before the upbeat fires.

## Groove Patterns as Coordination

Think of a 4-agent fleet:
- Agent A: downbeat every beat (heartbeat)
- Agent B: upbeat on beats 2 and 4 (syncopated)
- Agent C: downbeat only on beat 1 (anchor)
- Agent D: every upbeat (responder)

This creates a musical coordination pattern where agents complement rather than compete. The GroovePattern encodes these roles.

## Testing

22 tests covering all three swing feels, feel clamping, subdivision offsets, both groove patterns (four-on-floor and classic swing), measure duration, onset counting, syncopation map, syncopation application, swing clock (straight, swing, elapsed time, reset), scheduler basics, multi-agent scheduling, syncopated scheduling, swing offset computation, feel parameter progression, and groove repetition.

```bash
cargo test
```

## License

MIT
