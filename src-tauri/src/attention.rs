//! Visual attention policy, independent of AppKit and animation timing.

pub fn should_pulse(phase: &str, reduce_motion: bool) -> bool {
    phase == "idle" && !reduce_motion
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_wait_for_a_timer_pulses() {
        assert!(should_pulse("idle", false));
        for phase in ["running", "paused", "alert", "snoozed"] {
            assert!(!should_pulse(phase, false));
        }
    }

    #[test]
    fn reduce_motion_disables_the_idle_pulse() {
        assert!(!should_pulse("idle", true));
    }
}
