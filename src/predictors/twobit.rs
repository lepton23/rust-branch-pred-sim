use super::predictor::Predictor;
use super::table::BranchPredictionTable;
use crate::tracer::TraceEntry;

// const functions used for branch prediction hints in cold if statement paths
#[inline]
#[cold]
const fn cold() {}

#[inline]
const fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
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
        let index: u64 = line.pc_addr % N as u64;

        let taken: bool = self.table.table[index as usize].predict(line.taken);

        if unlikely(taken != line.taken) {
            self.mispredicts += 1;
        }
    }

    fn get_mispredicts(&self) -> u64 {
        self.mispredicts
    }
}
