use super::predictor::Predictor;
use crate::tracer::TraceEntry;

pub struct AlwaysTaken {
    mispredicts: u64,
}

impl AlwaysTaken {
    pub fn new() -> Self {
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
