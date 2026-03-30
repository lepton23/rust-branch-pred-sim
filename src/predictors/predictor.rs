use crate::tracer::TraceEntry;

pub trait Predictor {
    fn predict(&mut self, entry: &TraceEntry);
    fn get_mispredicts(&self) -> u64;
}
