use super::predictor::Predictor;
use crate::predictors::table::TwoBitBranchPredictionTable;
use crate::tracer::TraceEntry;

// const functions used for branch prediction hints in hot if statement paths
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

pub struct GShare<const N: usize> {
    mispredicts: u64,
    table: TwoBitBranchPredictionTable<N>,
    ghr: u64, // shift register of last log2(N) outcomes
    mask: u64,
}

impl<const N: usize> GShare<N> {
    pub fn new() -> Self {
        Self {
            mispredicts: 0,
            table: TwoBitBranchPredictionTable::new(),
            ghr: 0,
            mask: (1u64 << N.trailing_zeros()) - 1,
        }
    }
}

impl<const N: usize> Predictor for GShare<N> {
    fn predict(&mut self, line: &TraceEntry) {
        let index = (line.pc_addr ^ self.ghr) & ((N as u64) - 1);

        let taken = self.table.table[index as usize].predict(line.taken);

        if unlikely(taken != line.taken) {
            self.mispredicts += 1;
        }

        self.ghr = ((self.ghr << 1) | line.taken as u64) & self.mask;
    }

    fn get_mispredicts(&self) -> u64 {
        self.mispredicts
    }
}
