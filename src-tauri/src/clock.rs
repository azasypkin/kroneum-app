//! A monotonic clock that includes system sleep on macOS.

#[cfg(target_os = "macos")]
pub fn now_ms() -> u64 {
    use std::sync::OnceLock;

    #[repr(C)]
    struct Timebase {
        numer: u32,
        denom: u32,
    }

    unsafe extern "C" {
        fn mach_continuous_time() -> u64;
        fn mach_timebase_info(info: *mut Timebase) -> i32;
    }

    static RATIO: OnceLock<(u32, u32)> = OnceLock::new();
    let &(numer, denom) = RATIO.get_or_init(|| {
        let mut info = Timebase { numer: 0, denom: 0 };
        // SAFETY: The pointer references a writable, ABI-compatible stack value.
        let result = unsafe { mach_timebase_info(&mut info) };
        assert!(
            result == 0 && info.denom > 0,
            "macOS timebase is unavailable"
        );
        (info.numer, info.denom)
    });
    // SAFETY: This system clock function has no arguments or preconditions.
    let ticks = unsafe { mach_continuous_time() };
    (u128::from(ticks) * u128::from(numer) / u128::from(denom) / 1_000_000) as u64
}

#[cfg(not(target_os = "macos"))]
pub fn now_ms() -> u64 {
    use std::{sync::OnceLock, time::Instant};
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}
