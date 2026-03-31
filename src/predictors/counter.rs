#[derive(Clone, Copy)]
pub struct TwoBitCounter {
    pub predict_taken: bool,
    pub prev_predict_taken: bool,
}

impl TwoBitCounter {
    pub fn predict(&mut self, real_taken: bool) -> bool {
        let taken = self.predict_taken;

        (self.predict_taken, self.prev_predict_taken) =
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
