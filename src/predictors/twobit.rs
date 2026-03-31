use super::predictor::Predictor;
use crate::tracer::TraceEntry;

#[derive(Clone, Copy)]
pub struct Counter {
    pub predict_taken: bool,
    pub prev_predict_taken: bool,
}

impl Counter {
    fn predict(&mut self, real_taken: bool) -> bool {
        let taken = self.predict_taken;

        (self.predict_taken, self.prev_predict_taken) =
            match (self.predict_taken, self.prev_predict_taken, real_taken) {
                (false, false, false) => (false, false),
                (false, false, true) => (false, true),
                (false, true, false) => (false, false),
                (false, true, true) => (true, true),
                (true, false, false) => (false, false),
                (true, false, true) => (true, true),
                (true, true, false) => (true, false),
                (true, true, true) => (true, true),
            };

        taken
    }
}

pub struct BranchPredictionTable<const N: usize> {
    table: [Counter; N],
}

impl<const N: usize> BranchPredictionTable<N> {
    const fn new() -> Self {
        Self {
            table: [Counter {
                predict_taken: false,
                prev_predict_taken: false,
            }; N],
        }
    }
}

pub struct TwoBit<const N: usize> {
    mispredicts: u64,
    table: BranchPredictionTable<N>,
}

impl<const N: usize> TwoBit<N> {
    pub fn new() -> Self {
        Self {
            mispredicts: 0,
            table: BranchPredictionTable::<N>::new(),
        }
    }
}

impl<const N: usize> Predictor for TwoBit<N> {
    fn predict(&mut self, line: &TraceEntry) {
        let index = line.pc_addr % N as u64;

        let taken = self.table.table[index as usize].predict(line.taken);

        if taken != line.taken {
            self.mispredicts += 1;
        }
    }

    fn get_mispredicts(&self) -> u64 {
        self.mispredicts
    }
}
