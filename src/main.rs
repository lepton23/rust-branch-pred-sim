use std::env;

mod predictors;
use predictors::{
    alwaystake::AlwaysTaken, gshare::GShare, meta::Meta, predictor::Predictor, profiled::Profiled,
    twobit::TwoBit,
};

mod tracer;
use tracer::{process_trace, return_line_no};

macro_rules! dispatch {
    ($meta:expr, $body:expr) => {
        match $meta.alg {
            0 => {
                let bp = AlwaysTaken::new();
                $body(bp)
            }
            1 => {
                let bp = TwoBit::<512>::new();
                $body(bp)
            }
            2 => {
                let bp = TwoBit::<1024>::new();
                $body(bp)
            }
            3 => {
                let bp = TwoBit::<2048>::new();
                $body(bp)
            }
            4 => {
                let bp = TwoBit::<4096>::new();
                $body(bp)
            }
            5 => {
                let bp = GShare::new();
                $body(bp)
            }
            6 => {
                let bp = Profiled::new($meta.profile.clone());
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
        match bp_alg.as_str() {
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
        profile = Some(args[3].clone());
    }

    let meta = Meta { alg, profile };
    let _ = dispatch!(&meta, |mut bp| run(&mut bp, trace_path));
}

fn run<P: Predictor>(bp: &mut P, trace_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // begin running traces through BP
    run_trace(bp, trace_path);

    let total_branches: u64 = return_line_no(trace_path)?;

    // calculate mispredict rate
    let mispredicts = bp.get_mispredicts();
    let rate = (mispredicts as f64) / (total_branches as f64);

    println!("\n=== Mispredict Rate ===");
    println!("{mispredicts} mispredicts over {total_branches} branches.");
    println!("\n{rate} mispredict rate \n");

    Ok(())
}

fn run_trace<P: Predictor>(bp: &mut P, trace_path: &str) {
    let _ = process_trace(trace_path, |entry| {
        bp.predict(&entry);
    });
}
