#[derive(Clone, Copy)]
pub struct SaturatingCounter {
    pub predict_taken: bool,
    pub prev_predict_taken: bool,
}

impl SaturatingCounter {
    pub fn predict(&mut self, real_taken: bool) -> bool {
        let taken: bool = self.predict_taken;

        (self.predict_taken, self.prev_predict_taken) =
            // ideally this allows compiler to optimise this into a lookup table
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
