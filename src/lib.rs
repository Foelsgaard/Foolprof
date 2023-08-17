#![no_std]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

pub mod profile;
mod time;

pub use profile::{visit_profiles, Profile, Profiled, Profiler};

use core::sync::atomic::{self, AtomicBool};

static mut FREQ: u64 = 1;

pub struct OnExit {
    on_exit: fn(Profile),
}

impl Drop for OnExit {
    fn drop(&mut self) {
        profile::visit_profiles(self.on_exit);
    }
}

#[macro_export]
macro_rules! profiled {
    ($bytes:expr, $name:literal $(,$names:literal)*) => {
        static NAME: &str = concat!(module_path!(), "::", $name $(,concat!("::", $names))*);
        static PROFILER: $crate::profile::Profiler = $crate::profile::Profiler::new(NAME);
        let _profiled = PROFILER.profile($bytes as u64);
    };
}

#[cfg(feature = "std")]
fn eprint_profile(profile: Profile) {
    let Profile {
        name,
        cycles_min,
        cycles_max,
        cycles_avg,
        bytes_min,
        bytes_max,
        bytes_avg,
        samples,
    } = profile;

    fn eprintln_stat(cycles: u64, bytes: u64) {
        eprint!("{cycles} (");

        let freq = unsafe { FREQ };
        let t = (cycles as f64) / (freq as f64);
        {
            let (v, u) = if t > 1.0 {
                (t, "s")
            } else if t > 1E-3 {
                (t * 1E3, "ms")
            } else if t > 1E-6 {
                (t * 1E6, "μs")
            } else {
                (t * 1E9, "ns")
            };
            eprint!("{v:.02} {u}");
        }
        eprint!(") ");

        let r = (bytes as f64) / t;
        {
            let (v, u) = if r > 1E9 {
                (r * 1E-9, "GB")
            } else if r > 1E6 {
                (r * 1E-6, "MB")
            } else if r > 1E3 {
                (r * 1E-3, "KB")
            } else {
                (r, "B")
            };
            eprint!("{v:.02} {u}/s");
        }
        eprintln!();
    }

    eprintln!("--- {name} ---");
    eprintln!("samples: {samples}");
    eprint!("min: ");
    eprintln_stat(cycles_min, bytes_min);
    eprint!("max: ");
    eprintln_stat(cycles_max, bytes_max);
    eprint!("avg: ");
    eprintln_stat(cycles_avg, bytes_avg);
    eprintln!();
}

#[cfg(feature = "std")]
pub fn init_default() -> OnExit {
    init(eprint_profile)
}

pub fn init(on_exit: fn(Profile)) -> OnExit {
    use atomic::Ordering::Acquire;
    use time::{get_nsc_timestamp, get_timestamp};

    static INIT: AtomicBool = AtomicBool::new(true);

    if INIT.fetch_and(false, Acquire) {
        unsafe {
            let nsc_begin = get_nsc_timestamp();
            let tsc_begin = get_timestamp();

            libc::usleep(10_000);

            let nsc_end = get_nsc_timestamp();
            let tsc_end = get_timestamp();

            let freq = (tsc_end - tsc_begin) * 1000000000 / (nsc_end - nsc_begin);
            FREQ = freq;
        }
    }

    OnExit { on_exit }
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! init_default {
    () => {
        let _on_exit = $crate::init_default();
    };
}

#[macro_export]
macro_rules! init {
    ($on_exit:expr) => {
        let _on_exit = $crate::init($on_exit);
    };
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "std")]
    use super::*;

    #[cfg(feature = "std")]
    fn short() {
        profiled!(0, "short");
        std::thread::sleep(std::time::Duration::from_millis(50));
        {
            profiled!(0, "short", "inner");
            std::thread::sleep(std::time::Duration::from_millis(50));
            {
                profiled!(0, "short", "inner", "inner");
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }

    #[cfg(feature = "std")]
    fn medium() {
        profiled!(0, "medium");
        std::thread::sleep(std::time::Duration::from_millis(250));
    }

    #[cfg(feature = "std")]
    fn long() {
        profiled!(1000_000_000, "long");
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    #[cfg(feature = "std")]
    #[test]
    fn it_works() {
        init_default!();
        let hs = [
            std::thread::spawn(|| long()),
            std::thread::spawn(|| long()),
            std::thread::spawn(|| long()),
            std::thread::spawn(|| long()),
            std::thread::spawn(|| long()),
            std::thread::spawn(|| long()),
            std::thread::spawn(|| long()),
            std::thread::spawn(|| long()),
            std::thread::spawn(|| long()),
            std::thread::spawn(|| long()),
            std::thread::spawn(|| long()),
            std::thread::spawn(|| long()),
            std::thread::spawn(|| long()),
            std::thread::spawn(|| long()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| short()),
            std::thread::spawn(|| medium()),
            std::thread::spawn(|| medium()),
            std::thread::spawn(|| medium()),
            std::thread::spawn(|| medium()),
            std::thread::spawn(|| medium()),
            std::thread::spawn(|| medium()),
            std::thread::spawn(|| medium()),
            std::thread::spawn(|| medium()),
            std::thread::spawn(|| medium()),
        ];

        for h in hs {
            let _ = h.join();
        }
    }
}
