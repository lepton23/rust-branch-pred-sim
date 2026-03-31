use super::counter::TwoBitCounter;

pub struct TwoBitBranchPredictionTable<const N: usize> {
    pub table: [TwoBitCounter; N],
}

impl<const N: usize> TwoBitBranchPredictionTable<N> {
    pub const fn new() -> Self {
        Self {
            table: [TwoBitCounter {
                predict_taken: false,
                prev_predict_taken: false,
            }; N],
        }
    }
}
