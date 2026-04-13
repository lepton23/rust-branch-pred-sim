use crate::tracer::TraceEntry;

pub trait Predictor {
    fn predict(&mut self, entry: &TraceEntry);
    fn get_mispredicts(&self) -> u64;
    /// Returns per-PC bias data for profiled predictors: (pc_addr, taken_pct, is_hard).
    /// Non-profiled predictors return None.
    fn get_bias_data(&self) -> Option<Vec<(u64, f64, bool)>> {
        None
    }
}
