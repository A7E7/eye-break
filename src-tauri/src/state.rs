use crate::settings::Settings;
use serde::Serialize;

/// Where we are in the 20-20-20 cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    /// Counting down to the next break.
    Working,
    /// Reminder shown; waiting for the user to confirm they're looking away.
    AwaitingConfirm,
    /// The break countdown is running.
    Break,
    /// Reminders suspended (timed or until the user resumes).
    Paused,
}

/// The single source of truth, owned behind a `Mutex` and mutated by the
/// background timer thread, the tray, and the frontend commands.
pub struct AppState {
    pub settings: Settings,
    pub phase: Phase,
    /// Seconds remaining in the current phase (Working / Break / timed Paused).
    pub remaining: i64,
    /// True when a Paused phase has no scheduled end.
    pub pause_indefinite: bool,
    /// Seconds spent in AwaitingConfirm, used for the re-notify / skip grace.
    pub awaiting_elapsed: i64,
    /// Whether the grace re-notify has already fired this reminder.
    pub renotified: bool,
    /// True while the Working countdown is frozen because the user is idle.
    pub idle_paused: bool,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        let work = settings.work_secs();
        Self {
            settings,
            phase: Phase::Working,
            remaining: work,
            pause_indefinite: false,
            awaiting_elapsed: 0,
            renotified: false,
            idle_paused: false,
        }
    }

    /// Reset to a fresh Working countdown.
    pub fn start_working(&mut self) {
        self.phase = Phase::Working;
        self.remaining = self.settings.work_secs();
        self.awaiting_elapsed = 0;
        self.renotified = false;
        self.idle_paused = false;
        self.pause_indefinite = false;
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            phase: self.phase,
            remaining: self.remaining,
            idle_paused: self.idle_paused,
            settings: self.settings.clone(),
        }
    }
}

/// Serializable view emitted to the frontend on every tick.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub phase: Phase,
    pub remaining: i64,
    pub idle_paused: bool,
    pub settings: Settings,
}
