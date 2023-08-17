use core::sync::atomic::{self, AtomicBool, AtomicUsize};
use core::{cell, ptr};

use crate::time::get_timestamp;

const MAX_NUM_PROFILERS: usize = 0x1000;
static mut PROFILERS: [*const Profiler; MAX_NUM_PROFILERS] = [ptr::null(); MAX_NUM_PROFILERS];
static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub struct Profile {
    pub name: &'static str,
    pub cycles_min: u64,
    pub cycles_max: u64,
    pub cycles_avg: u64,
    pub bytes_min: u64,
    pub bytes_max: u64,
    pub bytes_avg: u64,
    pub samples: u64,
}

impl Profile {
    const fn new(name: &'static str) -> Self {
        Self {
            name,
            cycles_min: 0,
            cycles_max: 0,
            cycles_avg: 0,
            bytes_min: 0,
            bytes_max: 0,
            bytes_avg: 0,
            samples: 0,
        }
    }
}

pub struct Profiler {
    profile: cell::UnsafeCell<Profile>,
    lock: AtomicBool,
    init: AtomicBool,
}

unsafe impl Send for Profiler {}
unsafe impl Sync for Profiler {}

impl Profiler {
    pub const fn new(name: &'static str) -> Self {
        Self {
            profile: cell::UnsafeCell::new(Profile::new(name)),
            lock: AtomicBool::new(false),
            init: AtomicBool::new(true),
        }
    }

    pub fn profile(&'static self, bytes: u64) -> Profiled {
        use atomic::Ordering::Acquire;

        if self.init.fetch_and(false, Acquire) {
            let ix = COUNTER.fetch_add(1, Acquire);
            unsafe {
                PROFILERS[ix] = self;
            }
        }

        Profiled {
            profiler: self,
            bytes,
            start: get_timestamp(),
        }
    }

    fn with_lock<A, F: Fn(&mut Profile) -> A>(&self, f: F) -> A {
        use atomic::Ordering::{Acquire, Release};

        while self
            .lock
            .compare_exchange_weak(false, true, Acquire, Acquire)
            .is_err()
        {}
        let profile = unsafe { self.profile.get().as_mut().unwrap() };
        let result = f(profile);
        self.lock.store(false, Release);
        result
    }
}

pub struct Profiled<'a> {
    profiler: &'a Profiler,
    start: u64,
    bytes: u64,
}

impl Drop for Profiled<'_> {
    fn drop(&mut self) {
        let end = get_timestamp();
        if end > self.start {
            let t = end - self.start;
            let b = self.bytes;
            self.profiler.with_lock(|profile| {
                let t0_min = &mut profile.cycles_min;
                let b0_min = &mut profile.bytes_min;
                if (t as u128) * (*b0_min as u128) <= (*t0_min as u128) * (b as u128) {
                    *t0_min = t;
                    *b0_min = b;
                }

                let t0_max = &mut profile.cycles_max;
                let b0_max = &mut profile.bytes_max;
                if (t as u128) * (*b0_max as u128) >= (*t0_max as u128) * (b as u128) {
                    *t0_max = t;
                    *b0_max = b;
                }

                let t0_avg = &mut profile.cycles_avg;
                let b0_avg = &mut profile.bytes_avg;
                let n = &mut profile.samples;
                *t0_avg = (*t0_avg * *n + t) / (*n + 1);
                *b0_avg = (*b0_avg * *n + b) / (*n + 1);
                *n += 1;
            });
        }
    }
}

pub fn visit_profiles<F: Fn(Profile)>(f: F) {
    use atomic::Ordering::Acquire;

    let n = COUNTER.load(Acquire);
    let profilers = unsafe { PROFILERS.iter().take(n).flat_map(|p| p.as_ref()) };
    for profiler in profilers {
        profiler.with_lock(|profile| f(profile.clone()));
    }
}
