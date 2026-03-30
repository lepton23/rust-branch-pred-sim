use super::meta::Meta;
use super::predictor::Predictor;
use crate::tracer::TraceEntry;

pub struct AlwaysTaken {
    mispredicts: u64,
}

impl AlwaysTaken {
    fn new(meta: &Meta) -> Self {
        Self { mispredicts: 0 }
    }
}

impl Predictor for AlwaysTaken {
    fn predict(&mut self, line: &TraceEntry) {
        if !line.taken {
            self.mispredicts += 1;
        }
    }

    fn get_mispredicts(&self) -> u64 {
        self.mispredicts
    }
}
