use super::predictor::Predictor;
use crate::tracer::TraceEntry;

// const functions used for branch prediction hints in hot if statement paths
#[inline]
#[cold]
const fn cold() {}

#[inline]
const fn likely(b: bool) -> bool {
    if !b {
        cold()
    }
    b
}

pub struct Profiled<const N: usize> {
    mispredicts: u64,
}

impl<const N: usize> Profiled<N> {
    pub fn new(profile: Option<String>) -> Self {
        Self { mispredicts: 0 }
    }
}

impl<const N: usize> Predictor for Profiled<N> {
    fn predict(&mut self, line: &TraceEntry) {
        if !line.taken {
            self.mispredicts += 1;
        }
    }

    fn get_mispredicts(&self) -> u64 {
        self.mispredicts
    }
}
