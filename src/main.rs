use std::env;

mod predictors;
use predictors::{
    alwaystake::AlwaysTaken, gshare::GShare, meta::Meta, predictor::Predictor, profiled::Profiled,
    twobit::TwoBit,
};

mod tracer;
use tracer::{process_trace, return_branch_no};

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
                let bp = GShare::<512>::new();
                $body(bp)
            }
            3 => {
                let bp = Profiled::<512>::new($meta.profile.clone());
                $body(bp)
            }
            4 => {
                let bp = TwoBit::<1024>::new();
                $body(bp)
            }
            5 => {
                let bp = GShare::<1024>::new();
                $body(bp)
            }
            6 => {
                let bp = Profiled::<1024>::new($meta.profile.clone());
                $body(bp)
            }
            7 => {
                let bp = TwoBit::<2048>::new();
                $body(bp)
            }
            8 => {
                let bp = GShare::<2048>::new();
                $body(bp)
            }
            9 => {
                let bp = Profiled::<2048>::new($meta.profile.clone());
                $body(bp)
            }
            10 => {
                let bp = TwoBit::<4096>::new();
                $body(bp)
            }
            11 => {
                let bp = GShare::<4096>::new();
                $body(bp)
            }
            12 => {
                let bp = Profiled::<4096>::new($meta.profile.clone());
                $body(bp)
            }
            _ => panic!("Unsupported Predictor type"),
        }
    };
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let trace_path: &String = &args[1];
    let bp_alg: &String = &args[2];

    let mut alg: u8 = {
        match bp_alg.as_str() {
            "always" => 0,
            "twobit" => 1,
            "gshare" => 2,
            "profiled" => 3,
            _ => panic!("Unsupported BP Algorithm"),
        }
    };

    let size: &String;
    if alg != 0 {
        size = &args[3];

        alg = {
            match size.as_str() {
                "512" => alg,
                "1024" => alg + 3,
                "2048" => alg + 6,
                "4096" => alg + 9,
                _ => panic!("Unsupported BP Table Size"),
            }
        };
    }

    let mut profile: Option<String> = None;
    if alg == 3 || alg == 6 || alg == 9 || alg == 12 {
        profile = Some(trace_path.clone());
    }

    if args.len() > 4 || args.len() < 3 {
        panic!(
            "Incorrect input, usage: <path-to-executable> <trace-path> <bp-algorithm> <bp-table-size>?"
        );
    }

    let meta = Meta { alg, profile };
    let _ = dispatch!(&meta, |mut bp| run(&mut bp, trace_path));
}

fn run<P: Predictor>(bp: &mut P, trace_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // begin running traces through BP
    run_trace(bp, trace_path);

    // calculate mispredict rate
    let mispredicts: u64 = bp.get_mispredicts();
    let total_branches: u64 = return_branch_no();
    let rate: f64 = (100 as f64) * (mispredicts as f64) / (total_branches as f64);

    println!("\n=== Mispredict Rate ===");
    println!("{mispredicts} mispredicts over {total_branches} branches.");
    println!("\n{rate}% mispredict rate \n");

    Ok(())
}

fn run_trace<P: Predictor>(bp: &mut P, trace_path: &str) {
    let _ = process_trace(trace_path, |entry| {
        bp.predict(&entry);
    });
}
