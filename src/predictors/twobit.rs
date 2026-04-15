use super::predictor::Predictor;
use super::table::{TwoBitTable, SaturatingTable};
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
    pht: TwoBitTable<N>,
    index_mask: u64,
}

impl<const N: usize> TwoBit<N> {
    pub fn new() -> Self {
        Self {
            mispredicts: 0,
            pht: TwoBitTable::<N>::new(),
            index_mask: (N as u64) - 1,
        }
    }
}

impl<const N: usize> Predictor for TwoBit<N> {
    fn predict(&mut self, line: &TraceEntry) {
        let index: u64 = line.pc_addr & self.index_mask;

        let prediction: bool = self.pht.table[index as usize].predict(line.taken);

        if unlikely(prediction != line.taken) {
            self.mispredicts += 1;
        }
    }

    fn get_mispredicts(&self) -> u64 {
        self.mispredicts
    }
}

// TwoBit predictor using a saturating counter table instead of the state-machine counter.
pub struct TwoBitSat<const N: usize> {
    mispredicts: u64,
    pht: SaturatingTable<N>,
    index_mask: u64,
}

impl<const N: usize> TwoBitSat<N> {
    pub fn new() -> Self {
        Self {
            mispredicts: 0,
            pht: SaturatingTable::<N>::new(),
            index_mask: (N as u64) - 1,
        }
    }
}

impl<const N: usize> Predictor for TwoBitSat<N> {
    fn predict(&mut self, line: &TraceEntry) {
        let index: u64 = line.pc_addr & self.index_mask;

        let prediction: bool = self.pht.table[index as usize].predict(line.taken);

        if unlikely(prediction != line.taken) {
            self.mispredicts += 1;
        }
    }

    fn get_mispredicts(&self) -> u64 {
        self.mispredicts
    }
}
