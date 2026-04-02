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
    pht: BranchPredictionTable<N>,
    index_mask: u64,
}

impl<const N: usize> TwoBit<N> {
    pub fn new() -> Self {
        Self {
            mispredicts: 0,
            pht: BranchPredictionTable::<N>::new(),
            index_mask: (N as u64) - 1,
        }
    }
}

impl<const N: usize> Predictor for TwoBit<N> {
    fn predict(&mut self, line: &TraceEntry) {
        let index: u64 = line.pc_addr & self.index_mask;

        let taken: bool = self.pht.table[index as usize].predict(line.taken);

        if unlikely(taken != line.taken) {
            self.mispredicts += 1;
        }
    }

    fn get_mispredicts(&self) -> u64 {
        self.mispredicts
    }
}
