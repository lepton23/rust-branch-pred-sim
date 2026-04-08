#[derive(Clone, Copy)]
pub struct TwoBitCounter {
    pub predict_taken: bool,
    pub prev_predict_correct: bool,
}

impl TwoBitCounter {
    pub fn predict(&mut self, real_taken: bool) -> bool {
        let taken: bool = self.predict_taken;

        (self.predict_taken, self.prev_predict_correct) =
            // ideally this allows compiler to optimise this into a lookup table
            match (self.predict_taken, self.prev_predict_correct, real_taken) {
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

#[derive(Clone, Copy)]
pub struct SaturatingCounter {
    pub value: u8,
}

impl SaturatingCounter {
    pub fn predict(&mut self, real_taken: bool) -> bool {
        let prediction: bool = self.value >= 2;

        if real_taken{
            if self.value < 3 {
                self.value += 1; 
            }
        } else {
            if self.value > 0 {
                self.value -= 1;
            }
        }

        prediction
    }
}
