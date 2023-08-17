pub fn get_nsc_timestamp() -> u64 {
    let mut t = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    unsafe {
        if libc::clock_gettime(libc::CLOCK_MONOTONIC_RAW, &mut t) == 0 {
            (t.tv_sec * 1_000_000_000 + t.tv_nsec) as u64
        } else {
            0
        }
    }
}

pub fn get_timestamp() -> u64 {
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::x86::_rdtsc()
    }
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_rdtsc()
    }
}
