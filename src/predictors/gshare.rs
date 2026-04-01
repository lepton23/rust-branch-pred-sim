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
    pht: BranchPredictionTable<N>,
    ghr: u64, // shift register of last log2(N) outcomes
    index_mask: u64,
}

// TODO: possibly add separate GHR mask which is smaller than index_mask
// assuming here that we are using the most GHR possible - i.e. the same amount of GHR bits as the PHT index
impl<const N: usize> GShare<N> {
    pub fn new() -> Self {
        Self {
            mispredicts: 0,
            pht: BranchPredictionTable::new(),
            ghr: 0,
            index_mask: (N as u64) - 1,
        }
    }
}

impl<const N: usize> Predictor for GShare<N> {
    fn predict(&mut self, line: &TraceEntry) {
        // xor the pc_addr and ghr to index into BP table
        let index: u64 = (line.pc_addr ^ self.ghr) & self.index_mask;

        let taken: bool = self.pht.table[index as usize].predict(line.taken);

        if unlikely(taken != line.taken) {
            self.mispredicts += 1;
        }

        // store the last log2(N) bits of taken/not taken results
        self.ghr = ((self.ghr << 1) | line.taken as u64) & self.index_mask;
    }

    fn get_mispredicts(&self) -> u64 {
        self.mispredicts
    }
}
