use super::counter::SaturatingCounter;

pub struct BranchPredictionTable<const N: usize> {
    pub table: [SaturatingCounter; N],
}

impl<const N: usize> BranchPredictionTable<N> {
    pub const fn new() -> Self {
        Self {
            table: [SaturatingCounter {
                predict_taken: false,
                prev_predict_taken: false,
            }; N],
        }
    }
}
