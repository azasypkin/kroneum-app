//! Deterministic timer state. Time is supplied by the host, never read here.

use crate::dimming::{DimConfig, DimPlan, Dimmer, DEFAULT_MAX_DIM_PERCENT};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Timer {
    Work,
    #[serde(rename = "break")]
    EyeBreak,
}

impl Timer {
    pub fn next(self) -> Self {
        match self {
            Self::Work => Self::EyeBreak,
            Self::EyeBreak => Self::Work,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Sound {
    Chime,
    Glass,
    Ping,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct Settings {
    pub work_seconds: u32,
    pub break_seconds: u32,
    pub snooze_seconds: u32,
    pub idle_seconds: u32,
    pub volume: f32,
    pub sound: Sound,
    pub auto_continue: bool,
    pub dim_after_work: bool,
    pub fade_seconds: u32,
    pub max_dim_percent: u8,
    pub always_dim_break: bool,
    pub hide_on_timer_start: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work_seconds: 20 * 60,
            break_seconds: 30,
            snooze_seconds: 5 * 60,
            idle_seconds: 5 * 60,
            volume: 0.45,
            sound: Sound::Chime,
            auto_continue: false,
            dim_after_work: false,
            fade_seconds: 30,
            max_dim_percent: DEFAULT_MAX_DIM_PERCENT,
            always_dim_break: false,
            hide_on_timer_start: false,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        for (name, value, max) in [
            ("Work timer", self.work_seconds, 4 * 60 * 60),
            ("Eye break", self.break_seconds, 60 * 60),
            ("Snooze", self.snooze_seconds, 60 * 60),
            ("Idle reminder", self.idle_seconds, 60 * 60),
            ("Fade duration", self.fade_seconds, 600),
        ] {
            if value == 0 || value > max {
                return Err(format!("{name} must be between 1 and {max} seconds."));
            }
        }
        if !self.volume.is_finite() || !(0.05..=1.0).contains(&self.volume) {
            return Err("Volume must be between 5% and 100%.".into());
        }
        if !(1..=100).contains(&self.max_dim_percent) {
            return Err("Maximum dimming must be between 1% and 100%.".into());
        }
        Ok(())
    }

    fn dim_config(&self) -> DimConfig {
        DimConfig {
            after_work: self.dim_after_work,
            during_break: self.always_dim_break,
            fade_seconds: self.fade_seconds,
            max_dim_percent: self.max_dim_percent,
        }
    }

    pub fn duration(&self, timer: Timer) -> u32 {
        match timer {
            Timer::Work => self.work_seconds,
            Timer::EyeBreak => self.break_seconds,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Alert {
    Finished { timer: Timer },
    Idle,
}

#[derive(Clone, Debug, PartialEq)]
enum Phase {
    Idle {
        since: u64,
    },
    Running {
        timer: Timer,
        deadline: u64,
        total: u32,
    },
    Paused {
        timer: Timer,
        remaining_ms: u64,
        total: u32,
    },
    Alert(Alert),
    Snoozed {
        deadline: u64,
        alert: Alert,
        total: u32,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    StartWork,
    StartBreak,
    Pause,
    Resume,
    Stop,
    Acknowledge,
    Snooze,
    CancelSnooze,
    RestoreScreens,
}

/// Domain effects of a successful action, independent of the window host.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActionEffect {
    pub timer_started: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub phase: &'static str,
    pub timer: Option<Timer>,
    pub remaining_seconds: u32,
    pub total_seconds: u32,
    pub alert: Option<Alert>,
    pub next_timer: Timer,
    pub completed_breaks: u32,
    pub settings: Settings,
    pub dimming: Option<DimPlan>,
}

pub struct Engine {
    pub settings: Settings,
    phase: Phase,
    completed_breaks: u32,
    next_timer: Timer,
    dimmer: Dimmer,
}

fn seconds(ms: u64) -> u32 {
    ms.div_ceil(1_000).min(u32::MAX as u64) as u32
}

impl Engine {
    pub fn new(settings: Settings, now: u64) -> Self {
        Self {
            settings,
            phase: Phase::Idle { since: now },
            completed_breaks: 0,
            next_timer: Timer::Work,
            dimmer: Dimmer::default(),
        }
    }

    pub fn tick(&mut self, now: u64) {
        match self.phase {
            Phase::Running {
                timer, deadline, ..
            } if now >= deadline => {
                if timer == Timer::EyeBreak {
                    self.completed_breaks = self.completed_breaks.saturating_add(1);
                    self.dimmer.break_finished();
                } else {
                    self.dimmer.work_finished(self.settings.dim_config(), now);
                }
                self.next_timer = timer.next();
                self.phase = Phase::Alert(Alert::Finished { timer });
            }
            Phase::Idle { since }
                if now.saturating_sub(since) >= u64::from(self.settings.idle_seconds) * 1_000 =>
            {
                self.phase = Phase::Alert(Alert::Idle);
            }
            Phase::Snoozed {
                deadline, alert, ..
            } if now >= deadline => {
                self.phase = Phase::Alert(alert);
            }
            _ => {}
        }
    }

    fn start(&mut self, timer: Timer, now: u64) {
        if timer == Timer::Work {
            self.dimmer.work_started();
        } else {
            self.dimmer.break_started(self.settings.dim_config(), now);
        }
        let total = self.settings.duration(timer);
        self.next_timer = timer.next();
        self.phase = Phase::Running {
            timer,
            deadline: now + u64::from(total) * 1_000,
            total,
        };
    }

    pub fn act(&mut self, action: Action, now: u64) -> Result<ActionEffect, String> {
        // Expiration wins over pause at the deadline, even between host ticks.
        self.tick(now);
        match (action, self.phase.clone()) {
            (Action::StartWork, _) => self.start(Timer::Work, now),
            (Action::StartBreak, _) => self.start(Timer::EyeBreak, now),
            (
                Action::Pause,
                Phase::Running {
                    timer,
                    deadline,
                    total,
                },
            ) => {
                self.phase = Phase::Paused {
                    timer,
                    remaining_ms: deadline.saturating_sub(now),
                    total,
                };
            }
            (
                Action::Resume,
                Phase::Paused {
                    timer,
                    remaining_ms,
                    total,
                },
            ) => {
                self.phase = Phase::Running {
                    timer,
                    deadline: now + remaining_ms,
                    total,
                };
            }
            (Action::Stop, Phase::Running { .. } | Phase::Paused { .. }) => {
                self.dimmer.break_stopped(self.settings.dim_config(), now);
                self.phase = Phase::Idle { since: now }
            }
            (Action::Acknowledge, Phase::Alert(Alert::Finished { timer })) => {
                if self.settings.auto_continue {
                    self.start(timer.next(), now);
                } else {
                    self.phase = Phase::Idle { since: now };
                }
            }
            (Action::Snooze, Phase::Alert(alert)) => {
                let total = self.settings.snooze_seconds;
                self.phase = Phase::Snoozed {
                    deadline: now + u64::from(total) * 1_000,
                    alert,
                    total,
                };
            }
            (Action::CancelSnooze, Phase::Snoozed { alert, .. }) => {
                self.phase = Phase::Alert(alert)
            }
            (Action::RestoreScreens, _) => self.dimmer.restore(),
            (Action::Acknowledge, Phase::Alert(Alert::Idle)) => {
                return Err("Pick a timer or snooze the idle reminder.".into());
            }
            _ => return Err("This action is not available in the current timer state.".into()),
        }
        Ok(ActionEffect {
            timer_started: matches!(action, Action::StartWork | Action::StartBreak)
                || (matches!(action, Action::Acknowledge) && self.settings.auto_continue),
        })
    }

    pub fn update_settings(&mut self, settings: Settings, now: u64) {
        self.tick(now);
        self.dimmer
            .settings_changed(self.settings.dim_config(), settings.dim_config(), now);
        self.settings = settings;
        self.tick(now);
    }

    /// The clock host wakes at deadlines, not only at the next display tick.
    pub fn next_wake_ms(&self, now: u64) -> u64 {
        let deadline = match self.phase {
            Phase::Running { deadline, .. } | Phase::Snoozed { deadline, .. } => deadline,
            Phase::Idle { since } => since + u64::from(self.settings.idle_seconds) * 1_000,
            _ => return 1_000,
        };
        deadline.saturating_sub(now).clamp(1, 1_000)
    }

    pub fn snapshot(&self, now: u64) -> Snapshot {
        let (phase, timer, remaining_seconds, total_seconds, alert) = match self.phase {
            Phase::Idle { since } => (
                "idle",
                None,
                seconds(
                    (u64::from(self.settings.idle_seconds) * 1_000)
                        .saturating_sub(now.saturating_sub(since)),
                ),
                self.settings.idle_seconds,
                None,
            ),
            Phase::Running {
                timer,
                deadline,
                total,
            } => (
                "running",
                Some(timer),
                seconds(deadline.saturating_sub(now)),
                total,
                None,
            ),
            Phase::Paused {
                timer,
                remaining_ms,
                total,
            } => ("paused", Some(timer), seconds(remaining_ms), total, None),
            Phase::Alert(alert) => ("alert", None, 0, 0, Some(alert)),
            Phase::Snoozed {
                deadline,
                alert,
                total,
            } => (
                "snoozed",
                None,
                seconds(deadline.saturating_sub(now)),
                total,
                Some(alert),
            ),
        };
        Snapshot {
            phase,
            timer,
            remaining_seconds,
            total_seconds,
            alert,
            next_timer: self.next_timer,
            completed_breaks: self.completed_breaks,
            settings: self.settings.clone(),
            dimming: self.dimmer.plan(match self.phase {
                Phase::Running {
                    timer: Timer::EyeBreak,
                    deadline,
                    ..
                } => Some(deadline),
                _ => None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> Engine {
        Engine::new(Settings::default(), 0)
    }

    #[test]
    fn defaults_match_the_requested_routine() {
        let s = Settings::default();
        assert_eq!(
            (
                s.work_seconds,
                s.break_seconds,
                s.snooze_seconds,
                s.idle_seconds
            ),
            (1200, 30, 300, 300)
        );
        assert!(s.validate().is_ok());
    }

    #[test]
    fn serialization_matches_the_typescript_boundary() {
        assert_eq!(
            serde_json::to_value(Action::StartWork).unwrap(),
            "startWork"
        );
        assert_eq!(serde_json::to_value(Timer::EyeBreak).unwrap(), "break");
        let json = serde_json::to_value(engine().snapshot(0)).unwrap();
        assert_eq!(json["settings"]["workSeconds"], 1200);
        assert_eq!(json["remainingSeconds"], 300);
        assert_eq!(
            serde_json::to_value(Alert::Finished {
                timer: Timer::EyeBreak
            })
            .unwrap(),
            serde_json::json!({ "kind": "finished", "timer": "break" })
        );
    }

    #[test]
    fn expiration_is_exact_and_alarm_is_persistent() {
        let mut e = engine();
        e.act(Action::StartWork, 0).unwrap();
        e.tick(1_199_999);
        assert_eq!(e.snapshot(1_199_999).remaining_seconds, 1);
        e.tick(1_200_000);
        assert_eq!(
            e.snapshot(1_200_000).alert,
            Some(Alert::Finished { timer: Timer::Work })
        );
        e.tick(90_000_000);
        assert_eq!(e.snapshot(90_000_000).phase, "alert");
    }

    #[test]
    fn idle_cannot_be_dismissed_without_a_timer() {
        let mut e = engine();
        e.tick(299_999);
        assert_eq!(e.snapshot(299_999).phase, "idle");
        e.tick(300_000);
        assert!(e.act(Action::Acknowledge, 300_000).is_err());
        assert!(e.act(Action::Stop, 300_000).is_err());
        e.act(Action::StartBreak, 300_000).unwrap();
        assert_eq!(e.snapshot(300_000).phase, "running");
    }

    #[test]
    fn snooze_returns_the_original_alarm_without_recounting_a_break() {
        let mut e = engine();
        e.act(Action::StartBreak, 0).unwrap();
        e.tick(30_000);
        e.act(Action::Snooze, 30_000).unwrap();
        e.tick(329_999);
        assert_eq!(e.snapshot(329_999).phase, "snoozed");
        e.tick(330_000);
        assert_eq!(
            e.snapshot(330_000).alert,
            Some(Alert::Finished {
                timer: Timer::EyeBreak
            })
        );
        assert_eq!(e.snapshot(330_000).completed_breaks, 1);
        e.act(Action::Snooze, 330_000).unwrap();
        e.tick(630_000);
        assert_eq!(e.snapshot(630_000).completed_breaks, 1);
    }

    #[test]
    fn idle_snooze_suppresses_the_idle_watchdog() {
        let mut e = engine();
        e.tick(300_000);
        e.act(Action::Snooze, 300_000).unwrap();
        e.tick(599_999);
        assert_eq!(e.snapshot(599_999).phase, "snoozed");
        e.tick(600_000);
        assert_eq!(e.snapshot(600_000).alert, Some(Alert::Idle));
    }

    #[test]
    fn pause_preserves_fractional_seconds_and_suspends_idle_reminders() {
        let mut e = engine();
        e.act(Action::StartBreak, 0).unwrap();
        e.act(Action::Pause, 1_501).unwrap();
        e.tick(9_000_000);
        assert_eq!(e.snapshot(9_000_000).phase, "paused");
        e.act(Action::Resume, 9_000_000).unwrap();
        e.tick(9_028_498);
        assert_eq!(e.snapshot(9_028_498).phase, "running");
        e.tick(9_028_499);
        assert_eq!(e.snapshot(9_028_499).phase, "alert");
    }

    #[test]
    fn automatic_cycle_still_requires_acknowledgment() {
        let mut e = Engine::new(
            Settings {
                auto_continue: true,
                ..Settings::default()
            },
            0,
        );
        e.act(Action::StartWork, 0).unwrap();
        e.tick(1_200_000);
        assert_eq!(e.snapshot(1_200_000).phase, "alert");
        e.act(Action::Acknowledge, 1_201_000).unwrap();
        assert_eq!(e.snapshot(1_201_000).timer, Some(Timer::EyeBreak));
        assert_eq!(e.snapshot(1_201_000).remaining_seconds, 30);
    }

    #[test]
    fn starting_any_timer_cancels_an_alarm_or_snooze() {
        let mut e = engine();
        e.tick(300_000);
        e.act(Action::Snooze, 300_000).unwrap();
        e.act(Action::StartWork, 300_001).unwrap();
        e.tick(600_000);
        assert_eq!(e.snapshot(600_000).phase, "running");
        assert!(e.snapshot(600_000).alert.is_none());
    }

    #[test]
    fn settings_changes_do_not_resize_an_active_timer_or_snooze() {
        let mut e = engine();
        e.act(Action::StartBreak, 0).unwrap();
        e.update_settings(
            Settings {
                break_seconds: 60,
                ..Settings::default()
            },
            1_000,
        );
        assert_eq!(e.snapshot(1_000).remaining_seconds, 29);
        e.tick(30_000);
        e.act(Action::Snooze, 30_000).unwrap();
        e.update_settings(
            Settings {
                snooze_seconds: 60,
                ..Settings::default()
            },
            31_000,
        );
        assert_eq!(e.snapshot(31_000).remaining_seconds, 299);
    }

    #[test]
    fn stop_and_acknowledge_restart_the_idle_deadline() {
        let mut e = engine();
        e.act(Action::StartBreak, 0).unwrap();
        e.act(Action::Acknowledge, 30_000).unwrap();
        e.tick(329_999);
        assert_eq!(e.snapshot(329_999).phase, "idle");
        e.tick(330_000);
        assert_eq!(e.snapshot(330_000).phase, "alert");
    }

    #[test]
    fn invalid_settings_are_rejected() {
        for value in [0, 14_401, u32::MAX] {
            assert!(Settings {
                work_seconds: value,
                ..Settings::default()
            }
            .validate()
            .is_err());
        }
        for volume in [f32::NAN, f32::INFINITY, 0.0, 1.01] {
            assert!(Settings {
                volume,
                ..Settings::default()
            }
            .validate()
            .is_err());
        }
    }

    #[test]
    fn late_tick_after_sleep_produces_one_alarm_not_a_chain_of_cycles() {
        let mut e = engine();
        e.act(Action::StartBreak, 0).unwrap();
        e.tick(86_400_000);
        assert_eq!(e.snapshot(86_400_000).phase, "alert");
        assert_eq!(e.snapshot(86_400_000).completed_breaks, 1);
    }

    #[test]
    fn pause_at_expiration_cannot_hide_an_alarm() {
        let mut e = engine();
        e.act(Action::StartBreak, 0).unwrap();
        assert!(e.act(Action::Pause, 30_000).is_err());
        assert_eq!(e.snapshot(30_000).phase, "alert");
    }

    fn dimming_engine() -> Engine {
        Engine::new(
            Settings {
                dim_after_work: true,
                work_seconds: 10,
                break_seconds: 5,
                ..Settings::default()
            },
            0,
        )
    }

    #[test]
    fn dimming_is_opt_in_and_legacy_settings_get_safe_defaults() {
        let mut e = engine();
        e.act(Action::StartWork, 0).unwrap();
        e.tick(1_200_000);
        assert!(e.snapshot(1_200_000).dimming.is_none());
        let old: Settings =
            serde_json::from_value(serde_json::json!({ "workSeconds": 1500, "breakSeconds": 40 }))
                .unwrap();
        assert_eq!(old.work_seconds, 1500);
        assert!(!old.dim_after_work);
        assert_eq!(old.fade_seconds, 30);
        assert_eq!(old.max_dim_percent, 85);
        assert!(!old.always_dim_break);
        assert!(!old.hide_on_timer_start);
    }

    #[test]
    fn dimming_persists_through_the_break_and_clears_before_acceptance() {
        let mut e = dimming_engine();
        e.act(Action::StartWork, 0).unwrap();
        e.tick(9_999);
        assert!(e.snapshot(9_999).dimming.is_none());
        e.tick(10_000);
        assert_eq!(e.snapshot(10_000).dimming.unwrap().started_at_ms, 10_000);
        e.act(Action::StartBreak, 11_000).unwrap();
        assert_eq!(
            e.snapshot(11_000).dimming.unwrap().clear_at_ms,
            Some(16_000)
        );
        e.tick(15_999);
        assert!(e.snapshot(15_999).dimming.is_some());
        e.tick(16_000);
        assert!(e.snapshot(16_000).dimming.is_none());
        assert_eq!(e.snapshot(16_000).phase, "alert");
    }

    #[test]
    fn snooze_and_dismiss_do_not_cancel_fading() {
        let mut e = dimming_engine();
        e.act(Action::StartWork, 0).unwrap();
        e.act(Action::Snooze, 10_000).unwrap();
        assert!(e.snapshot(10_000).dimming.is_some());
        e.act(Action::CancelSnooze, 11_000).unwrap();
        e.act(Action::Acknowledge, 11_000).unwrap();
        assert!(e.snapshot(11_000).dimming.is_some());
    }

    #[test]
    fn starting_work_restores_screens_until_the_new_work_timer_finishes() {
        let mut e = dimming_engine();
        e.act(Action::StartWork, 0).unwrap();
        e.tick(10_000);
        assert!(e.snapshot(10_000).dimming.is_some());
        e.act(Action::StartWork, 11_000).unwrap();
        assert!(e.snapshot(11_000).dimming.is_none());
        assert_eq!(e.snapshot(11_000).timer, Some(Timer::Work));
        assert_eq!(e.snapshot(11_000).completed_breaks, 0);
        e.update_settings(e.settings.clone(), 12_000);
        assert!(e.snapshot(12_000).dimming.is_none());
        e.tick(20_999);
        assert!(e.snapshot(20_999).dimming.is_none());
        e.tick(21_000);
        assert_eq!(e.snapshot(21_000).dimming.unwrap().started_at_ms, 21_000);
    }

    #[test]
    fn switching_from_an_unfinished_break_to_work_cancels_the_old_fade_deadline() {
        let mut e = dimming_engine();
        e.act(Action::StartWork, 0).unwrap();
        e.act(Action::StartBreak, 10_000).unwrap();
        assert_eq!(
            e.snapshot(10_000).dimming.unwrap().clear_at_ms,
            Some(15_000)
        );
        e.act(Action::StartWork, 11_000).unwrap();
        assert!(e.snapshot(11_000).dimming.is_none());
        e.tick(15_000);
        assert!(e.snapshot(15_000).dimming.is_none());
        assert_eq!(e.snapshot(15_000).phase, "running");
        assert_eq!(e.snapshot(15_000).completed_breaks, 0);
    }

    #[test]
    fn disabling_or_restoring_clears_dimming_without_changing_the_timer() {
        let mut e = dimming_engine();
        e.act(Action::StartWork, 0).unwrap();
        e.act(Action::StartBreak, 10_000).unwrap();
        e.act(Action::RestoreScreens, 11_000).unwrap();
        assert!(e.snapshot(11_000).dimming.is_none());
        assert_eq!(e.snapshot(11_000).timer, Some(Timer::EyeBreak));
        e.update_settings(
            Settings {
                dim_after_work: false,
                ..e.settings.clone()
            },
            11_500,
        );
        assert!(e.snapshot(11_500).dimming.is_none());
        e.update_settings(
            Settings {
                dim_after_work: true,
                ..e.settings.clone()
            },
            12_000,
        );
        assert!(e.snapshot(12_000).dimming.is_some());
        e.update_settings(
            Settings {
                dim_after_work: false,
                ..e.settings.clone()
            },
            12_001,
        );
        assert!(e.snapshot(12_001).dimming.is_none());
    }

    #[test]
    fn pause_and_resume_change_only_the_native_restore_deadline() {
        let mut e = dimming_engine();
        e.act(Action::StartWork, 0).unwrap();
        e.act(Action::StartBreak, 10_000).unwrap();
        e.act(Action::Pause, 11_000).unwrap();
        assert_eq!(e.snapshot(11_000).dimming.unwrap().clear_at_ms, None);
        e.act(Action::Resume, 20_000).unwrap();
        assert_eq!(
            e.snapshot(20_000).dimming.unwrap().clear_at_ms,
            Some(24_000)
        );
        e.tick(24_000);
        assert!(e.snapshot(24_000).dimming.is_none());
    }

    #[test]
    fn late_work_expiration_starts_a_fresh_fade_after_wake() {
        let mut e = dimming_engine();
        e.act(Action::StartWork, 0).unwrap();
        e.tick(500_000);
        assert_eq!(
            e.snapshot(500_000).dimming.unwrap().opacity_at(500_000),
            0.0
        );
    }

    #[test]
    fn the_scheduler_wakes_exactly_at_a_near_deadline() {
        let mut e = dimming_engine();
        e.act(Action::StartBreak, 0).unwrap();
        assert_eq!(e.next_wake_ms(4_900), 100);
        assert_eq!(e.next_wake_ms(1_000), 1_000);
        for fade_seconds in [0, 601] {
            assert!(Settings {
                fade_seconds,
                ..Settings::default()
            }
            .validate()
            .is_err());
        }
    }

    #[test]
    fn always_dim_applies_to_a_manual_break_and_clears_before_acceptance() {
        let mut e = Engine::new(
            Settings {
                always_dim_break: true,
                break_seconds: 5,
                max_dim_percent: 40,
                fade_seconds: 2,
                ..Settings::default()
            },
            0,
        );
        e.act(Action::StartBreak, 1_000).unwrap();
        let plan = e.snapshot(1_000).dimming.unwrap();
        assert_eq!(plan.clear_at_ms, Some(6_000));
        assert_eq!(plan.max_opacity(), 0.4);
        assert_eq!(plan.opacity_at(4_000), 0.4);
        e.tick(6_000);
        assert!(e.snapshot(6_000).dimming.is_none());
        assert_eq!(e.snapshot(6_000).phase, "alert");
    }

    #[test]
    fn all_fading_mode_combinations_preserve_work_start_restoration() {
        for dim_after_work in [false, true] {
            for always_dim_break in [false, true] {
                let mut e = Engine::new(
                    Settings {
                        dim_after_work,
                        always_dim_break,
                        work_seconds: 10,
                        break_seconds: 5,
                        ..Settings::default()
                    },
                    0,
                );
                e.act(Action::StartWork, 0).unwrap();
                e.tick(10_000);
                assert_eq!(e.snapshot(10_000).dimming.is_some(), dim_after_work);
                e.act(Action::StartBreak, 11_000).unwrap();
                assert_eq!(
                    e.snapshot(11_000).dimming.is_some(),
                    dim_after_work || always_dim_break
                );
                e.act(Action::StartWork, 12_000).unwrap();
                e.update_settings(e.settings.clone(), 12_100);
                assert!(e.snapshot(12_100).dimming.is_none());
            }
        }
    }

    #[test]
    fn maximum_dimming_is_checked_and_updates_an_existing_fade_on_save() {
        for max_dim_percent in [0, 101] {
            assert!(Settings {
                max_dim_percent,
                ..Settings::default()
            }
            .validate()
            .is_err());
        }
        for max_dim_percent in [1, 100] {
            assert!(Settings {
                max_dim_percent,
                ..Settings::default()
            }
            .validate()
            .is_ok());
        }
        let mut e = dimming_engine();
        e.act(Action::StartWork, 0).unwrap();
        e.tick(10_000);
        e.update_settings(
            Settings {
                max_dim_percent: 40,
                fade_seconds: 60,
                ..e.settings.clone()
            },
            12_000,
        );
        let plan = e.snapshot(12_000).dimming.unwrap();
        assert_eq!(plan.started_at_ms, 10_000);
        assert_eq!(plan.fade_seconds, 30);
        assert_eq!(plan.max_dim_percent, 40);
        assert_eq!(plan.opacity_at(50_000), 0.4);
    }

    #[test]
    fn enabling_always_dim_during_a_break_does_not_restart_the_timer() {
        let mut e = Engine::new(
            Settings {
                break_seconds: 5,
                ..Settings::default()
            },
            0,
        );
        e.act(Action::StartBreak, 0).unwrap();
        e.update_settings(
            Settings {
                always_dim_break: true,
                ..e.settings.clone()
            },
            1_000,
        );
        let plan = e.snapshot(1_000).dimming.unwrap();
        assert_eq!(plan.started_at_ms, 1_000);
        assert_eq!(plan.clear_at_ms, Some(5_000));
        assert_eq!(e.snapshot(1_000).remaining_seconds, 4);
    }

    #[test]
    fn stopping_a_break_only_fade_restores_screens_and_pause_reschedules_restoration() {
        let mut e = Engine::new(
            Settings {
                always_dim_break: true,
                break_seconds: 5,
                ..Settings::default()
            },
            0,
        );
        e.act(Action::StartBreak, 0).unwrap();
        e.act(Action::Pause, 1_000).unwrap();
        assert_eq!(e.snapshot(1_000).dimming.unwrap().clear_at_ms, None);
        e.act(Action::Resume, 10_000).unwrap();
        assert_eq!(
            e.snapshot(10_000).dimming.unwrap().clear_at_ms,
            Some(14_000)
        );
        e.act(Action::Stop, 11_000).unwrap();
        e.update_settings(e.settings.clone(), 11_100);
        assert!(e.snapshot(11_100).dimming.is_none());
    }

    #[test]
    fn always_dim_respects_manual_restore_until_an_explicit_new_break() {
        let mut e = Engine::new(
            Settings {
                always_dim_break: true,
                ..Settings::default()
            },
            0,
        );
        e.act(Action::StartBreak, 0).unwrap();
        e.act(Action::RestoreScreens, 1_000).unwrap();
        e.update_settings(e.settings.clone(), 2_000);
        assert!(e.snapshot(2_000).dimming.is_none());
        e.act(Action::StartBreak, 3_000).unwrap();
        assert!(e.snapshot(3_000).dimming.is_some());
    }

    #[test]
    fn only_a_new_timer_reports_a_start_effect() {
        let mut e = engine();
        assert!(e.act(Action::StartWork, 0).unwrap().timer_started);
        assert!(!e.act(Action::Pause, 100).unwrap().timer_started);
        assert!(!e.act(Action::Resume, 200).unwrap().timer_started);
        assert!(!e.act(Action::RestoreScreens, 300).unwrap().timer_started);
        assert!(!e.act(Action::Stop, 400).unwrap().timer_started);
        assert!(e.act(Action::StartBreak, 500).unwrap().timer_started);
        assert!(!e.act(Action::Snooze, 30_500).unwrap().timer_started);
        assert!(!e.act(Action::CancelSnooze, 30_600).unwrap().timer_started);
        assert!(!e.act(Action::Acknowledge, 30_700).unwrap().timer_started);
        assert!(e.act(Action::Acknowledge, 30_800).is_err());
    }

    #[test]
    fn accepting_a_cycle_completion_reports_the_actual_timer_start() {
        let mut e = Engine::new(
            Settings {
                auto_continue: true,
                hide_on_timer_start: true,
                work_seconds: 1,
                break_seconds: 1,
                ..Settings::default()
            },
            0,
        );
        e.act(Action::StartWork, 0).unwrap();
        let effect = e.act(Action::Acknowledge, 1_000).unwrap();
        assert!(effect.timer_started);
        assert_eq!(e.snapshot(1_000).timer, Some(Timer::EyeBreak));
        assert!(e.act(Action::Acknowledge, 2_000).unwrap().timer_started);
        assert_eq!(e.snapshot(2_000).timer, Some(Timer::Work));
    }
}
