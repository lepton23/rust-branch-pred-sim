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

pub struct GShare<const N: usize> {
    mispredicts: u64,
    table: BranchPredictionTable<N>,
    ghr: u64, // shift register of last log2(N) outcomes
    mask: u64,
}

impl<const N: usize> GShare<N> {
    pub fn new() -> Self {
        Self {
            mispredicts: 0,
            table: BranchPredictionTable::new(),
            ghr: 0,
            mask: (N as u64) - 1,
        }
    }
}

impl<const N: usize> Predictor for GShare<N> {
    fn predict(&mut self, line: &TraceEntry) {
        // xor the pc_addr and ghr to index into BP table
        let index: u64 = (line.pc_addr ^ self.ghr) & self.mask;

        let taken: bool = self.table.table[index as usize].predict(line.taken);

        if unlikely(taken != line.taken) {
            self.mispredicts += 1;
        }

        // store the last log2(N) bits of taken/not taken results
        self.ghr = ((self.ghr << 1) | line.taken as u64) & self.mask;
    }

    fn get_mispredicts(&self) -> u64 {
        self.mispredicts
    }
}
