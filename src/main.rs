use std::env;

mod predictors;
use predictors::{AlwaysTake}

mod tracer;
use tracer::process_trace;

#[derive(Debug, Clone)]
pub struct Meta {
    pub alg: u8,
    pub profile: Option<String>,
}

macro_rules! dispatch {
    ($meta:expr, $body:expr) => {
        match $meta.alg {
            0 => {
                let bp = AlwaysTake::new(&$meta.clone());
                $body(bp)
            }
            1 => {
                let bp = TwoBit::<512, _>::new(&$meta.clone());
                $body(bp)
            }
            2 => {
                let bp = TwoBit::<1024, _>::new(&$meta.clone());
                $body(bp)
            }
            3 => {
                let bp = TwoBit::<2048, _>::new(&$meta.clone());
                $body(bp)
            }
            4 => {
                let bp = TwoBit::<4096, _>::new(&$meta.clone());
                $body(bp)
            }
            5 => {
                let bp = GShare::new(&$meta.clone());
                $body(bp)
            }
            6 => {
                let bp = Profiled::new(&$meta.clone());
                $body(bp)
            }
            _ => panic!("Unsupported Predictor type"),
        }
    };
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let bp_alg = &args[1];
    let trace_path = &args[2];

    let alg = {
        match bp_alg.to_str() {
            "always" => 0,
            "twobit512" => 1,
            "twobit1024" => 2,
            "twobit2048" => 3,
            "twobit4096" => 4,
            "gshare" => 5,
            "profiled" => 6,
            _ => panic!("Unsupported BP Algorithm"),
        }
    };

    let mut profile: Option<String> = None;

    if args.len() == 4 {
        let profile: Option<String> = Some(args[3]); 
    } 

    let meta: Meta = Meta { alg, profile };

    let _ = dispatch!(&meta, |mut bp| run(&mut bp, trace_path));
}

fn run<P: Predictor>(bp: &mut P, trace_path: &str) {
    // begin running traces through BP
    run_trace(bp, trace_path);   
}

fn run_trace<P: Predictor>(bp: &mut P, trace_path: &str) {
    let _ = process_trace(trace_path, |entry| {
        bp.process(
            entry.pc_address,
            entry.ta_address,
            entry.kind,
            entry.direct,
            entry.conditional,
            entry.taken,
        );
    });
}
