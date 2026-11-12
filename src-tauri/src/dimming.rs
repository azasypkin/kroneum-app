//! Independent work-completion and active-break triggers for native dimming.
use serde::Serialize;

pub const DEFAULT_MAX_DIM_PERCENT: u8 = 85;

#[derive(Clone, Copy, Debug)]
pub struct DimConfig {
    pub after_work: bool,
    pub during_break: bool,
    pub fade_seconds: u32,
    pub max_dim_percent: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DimPlan {
    pub started_at_ms: u64,
    pub fade_seconds: u32,
    pub max_dim_percent: u8,
    pub clear_at_ms: Option<u64>,
}

impl DimPlan {
    pub fn max_opacity(self) -> f32 {
        f32::from(self.max_dim_percent.min(100)) / 100.0
    }

    pub fn opacity_at(self, now: u64) -> f32 {
        if self.clear_at_ms.is_some_and(|deadline| now >= deadline) {
            return 0.0;
        }
        let fraction = now.saturating_sub(self.started_at_ms) as f64
            / (f64::from(self.fade_seconds.max(1)) * 1_000.0);
        self.max_opacity() * fraction.min(1.0) as f32
    }

    pub fn fade_remaining_ms(self, now: u64) -> u64 {
        (self.started_at_ms + u64::from(self.fade_seconds) * 1_000).saturating_sub(now)
    }
}

#[derive(Default)]
pub struct Dimmer {
    break_due: bool,
    break_active: bool,
    suppressed: bool,
    session: Option<DimPlan>,
}

impl Dimmer {
    pub fn work_finished(&mut self, config: DimConfig, now: u64) {
        self.break_due = true;
        self.suppressed = false;
        self.reconcile(config, now);
    }

    pub fn break_started(&mut self, config: DimConfig, now: u64) {
        self.break_active = true;
        if config.during_break {
            self.suppressed = false;
        }
        self.reconcile(config, now);
    }

    pub fn break_stopped(&mut self, config: DimConfig, now: u64) {
        self.break_active = false;
        self.reconcile(config, now);
    }

    pub fn settings_changed(&mut self, previous: DimConfig, config: DimConfig, now: u64) {
        if (config.after_work && !previous.after_work && self.break_due)
            || (config.during_break && !previous.during_break && self.break_active)
        {
            self.suppressed = false;
        }
        self.reconcile(config, now);
    }

    fn reconcile(&mut self, config: DimConfig, now: u64) {
        let wanted =
            (config.after_work && self.break_due) || (config.during_break && self.break_active);
        if !wanted {
            self.session = None;
            return;
        }
        if !self.suppressed && self.session.is_none() {
            self.session = Some(DimPlan {
                started_at_ms: now,
                fade_seconds: config.fade_seconds,
                max_dim_percent: config.max_dim_percent,
                clear_at_ms: None,
            });
        }
        if let Some(plan) = self.session.as_mut() {
            // The limit changes on save, while the fade's original duration stays fixed.
            plan.max_dim_percent = config.max_dim_percent;
        }
    }

    pub fn break_finished(&mut self) {
        *self = Self::default();
    }

    pub fn work_started(&mut self) {
        *self = Self::default();
    }

    pub fn restore(&mut self) {
        self.session = None;
        self.suppressed = true;
    }

    pub fn plan(&self, break_deadline: Option<u64>) -> Option<DimPlan> {
        self.session.map(|plan| DimPlan {
            clear_at_ms: break_deadline,
            ..plan
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> DimConfig {
        DimConfig {
            after_work: true,
            during_break: false,
            fade_seconds: 30,
            max_dim_percent: DEFAULT_MAX_DIM_PERCENT,
        }
    }

    #[test]
    fn fade_is_linear_capped_and_clears_exactly_at_the_break_deadline() {
        let plan = DimPlan {
            started_at_ms: 1_000,
            fade_seconds: 10,
            max_dim_percent: DEFAULT_MAX_DIM_PERCENT,
            clear_at_ms: Some(21_000),
        };
        assert_eq!(plan.opacity_at(500), 0.0);
        assert_eq!(plan.opacity_at(1_000), 0.0);
        assert!((plan.opacity_at(6_000) - 0.85 / 2.0).abs() < 0.001);
        assert_eq!(plan.opacity_at(20_999), 0.85);
        assert_eq!(plan.opacity_at(21_000), 0.0);
        assert_eq!(plan.fade_remaining_ms(6_000), 5_000);
    }

    #[test]
    fn a_completed_work_timer_is_required() {
        let mut dimmer = Dimmer::default();
        dimmer.settings_changed(config(), config(), 0);
        assert!(dimmer.plan(None).is_none());
        dimmer.work_finished(config(), 100);
        assert!(dimmer.plan(None).is_some());
    }

    #[test]
    fn restoring_suppresses_only_the_current_work_completion() {
        let mut dimmer = Dimmer::default();
        dimmer.work_finished(config(), 100);
        dimmer.restore();
        dimmer.settings_changed(config(), config(), 200);
        assert!(dimmer.plan(None).is_none());
        dimmer.work_finished(config(), 300);
        assert_eq!(dimmer.plan(None).unwrap().started_at_ms, 300);
    }

    #[test]
    fn disabling_clears_and_enabling_applies_to_an_outstanding_break() {
        let mut dimmer = Dimmer::default();
        let disabled = DimConfig {
            after_work: false,
            ..config()
        };
        dimmer.work_finished(disabled, 100);
        assert!(dimmer.plan(None).is_none());
        dimmer.settings_changed(disabled, config(), 200);
        assert!(dimmer.plan(None).is_some());
        dimmer.settings_changed(config(), disabled, 300);
        assert!(dimmer.plan(None).is_none());
    }

    #[test]
    fn active_fades_keep_their_duration_on_settings_updates_or_duplicate_completion() {
        let mut dimmer = Dimmer::default();
        dimmer.work_finished(config(), 100);
        let longer = DimConfig {
            fade_seconds: 60,
            ..config()
        };
        dimmer.settings_changed(config(), longer, 200);
        dimmer.work_finished(longer, 300);
        assert_eq!(dimmer.plan(None).unwrap().started_at_ms, 100);
        assert_eq!(dimmer.plan(None).unwrap().fade_seconds, 30);
        dimmer.break_finished();
        assert!(dimmer.plan(None).is_none());
    }

    #[test]
    fn a_new_work_session_clears_the_fade_and_its_pending_break() {
        let mut dimmer = Dimmer::default();
        dimmer.work_finished(config(), 100);
        dimmer.work_started();
        assert!(dimmer.plan(None).is_none());
        dimmer.settings_changed(config(), config(), 200);
        assert!(dimmer.plan(None).is_none());
        dimmer.work_finished(config(), 300);
        assert_eq!(dimmer.plan(None).unwrap().started_at_ms, 300);
    }

    #[test]
    fn always_dim_breaks_is_independent_and_stops_with_a_manual_break() {
        let mut dimmer = Dimmer::default();
        let config = DimConfig {
            after_work: false,
            during_break: true,
            ..config()
        };
        dimmer.break_started(config, 100);
        assert!(dimmer.plan(Some(5_100)).is_some());
        dimmer.break_stopped(config, 200);
        assert!(dimmer.plan(None).is_none());
    }

    #[test]
    fn one_enabled_trigger_keeps_an_existing_fade_without_restarting_it() {
        let mut dimmer = Dimmer::default();
        dimmer.work_finished(config(), 100);
        let both = DimConfig {
            during_break: true,
            ..config()
        };
        dimmer.break_started(both, 200);
        let break_only = DimConfig {
            after_work: false,
            ..both
        };
        dimmer.settings_changed(both, break_only, 300);
        assert_eq!(dimmer.plan(None).unwrap().started_at_ms, 100);
        dimmer.break_stopped(break_only, 400);
        assert!(dimmer.plan(None).is_none());
    }

    #[test]
    fn manual_restore_stays_suppressed_until_an_explicit_new_break() {
        let mut dimmer = Dimmer::default();
        let config = DimConfig {
            during_break: true,
            ..config()
        };
        dimmer.break_started(config, 100);
        dimmer.restore();
        dimmer.settings_changed(config, config, 200);
        assert!(dimmer.plan(None).is_none());
        dimmer.break_started(config, 300);
        assert_eq!(dimmer.plan(None).unwrap().started_at_ms, 300);
    }

    #[test]
    fn changing_the_maximum_updates_the_current_plan_without_restarting_its_clock() {
        let mut dimmer = Dimmer::default();
        dimmer.work_finished(config(), 100);
        let lower = DimConfig {
            max_dim_percent: 40,
            ..config()
        };
        dimmer.settings_changed(config(), lower, 200);
        let plan = dimmer.plan(None).unwrap();
        assert_eq!(plan.started_at_ms, 100);
        assert_eq!(plan.max_opacity(), 0.4);
        assert_eq!(plan.opacity_at(31_000), 0.4);
    }
}
