use super::meta::Meta;
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

pub struct GShare {
    mispredicts: u64,
}

impl GShare {
    fn new(meta: &Meta) -> Self {
        Self { mispredicts: 0 }
    }
}

impl Predictor for GShare {
    fn predict(&mut self, line: &TraceEntry) {
        if !line.taken {
            self.mispredicts += 1;
        }
    }

    fn get_mispredicts(&self) -> u64 {
        self.mispredicts
    }
}
