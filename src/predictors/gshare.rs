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

pub struct GShare<const N: usize> {
    mispredicts: u64,
}

impl<const N: usize> GShare<N> {
    pub fn new() -> Self {
        Self { mispredicts: 0 }
    }
}

impl<const N: usize> Predictor for GShare<N> {
    fn predict(&mut self, line: &TraceEntry) {
        if !line.taken {
            self.mispredicts += 1;
        }
    }

    fn get_mispredicts(&self) -> u64 {
        self.mispredicts
    }
}
