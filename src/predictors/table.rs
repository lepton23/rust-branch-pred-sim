use super::counter::{TwoBitCounter, SaturatingCounter};

pub struct TwoBitTable<const N: usize> {
    pub table: [TwoBitCounter; N],
}

impl<const N: usize> TwoBitTable<N> {
    pub const fn new() -> Self {
        Self {
            table: [TwoBitCounter {
                predict_taken: false,
                prev_predict_correct: false,
            }; N],
        }
    }
}

pub struct SaturatingTable<const N: usize> {
    pub table: [SaturatingCounter; N],
}

impl<const N: usize> SaturatingTable<N> {
    pub const fn new() -> Self {
        Self {
            table: [SaturatingCounter {
                value: 1,
            }; N],
        }
    }
}
