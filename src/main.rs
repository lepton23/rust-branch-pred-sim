use std::env;

mod predictors;
use predictors::{AlwaysTake, GShare, Meta, Predictor, Profiled, TwoBit};

mod tracer;
use tracer::{process_trace, return_line_no};

macro_rules! dispatch {
    ($meta:expr, $body:expr) => {
        match $meta.alg {
            0 => {
                let bp = AlwaysTake::new(&$meta.clone());
                $body(bp)
            }
            1 => {
                let bp = TwoBit::<512>::new(&$meta.clone());
                $body(bp)
            }
            2 => {
                let bp = TwoBit::<1024>::new(&$meta.clone());
                $body(bp)
            }
            3 => {
                let bp = TwoBit::<2048>::new(&$meta.clone());
                $body(bp)
            }
            4 => {
                let bp = TwoBit::<4096>::new(&$meta.clone());
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

    let total_branches = return_line_no(trace_path).unwrap()?;

    // calculate mispredict rate
    let mispredicts = bp.get_mispredicts();
    let rate = (mispredicts as f64) / (total_branches as f64);

    println!("Mispredict Rate:");
    println!("{mispredicts} mispredicts over {total_branches} branches.");
    println!("\n{rate} mispredict rate.");
}

fn run_trace<P: Predictor>(bp: &mut P, trace_path: &str) {
    let _ = process_trace(trace_path, |entry| {
        bp.predict(
            entry.pc_addr,
            entry.ta_addr,
            entry.kind,
            entry.direct,
            entry.conditional,
            entry.taken,
        );
    });
}
