use std::env;

mod predictors;
use predictors::{
    alwaystake::AlwaysTaken,
    gshare::{GShare, GShareSat},
    meta::Meta,
    predictor::Predictor,
    profiled::{Profiled, ProfiledSat},
    twobit::{TwoBit, TwoBitSat},
};

mod tracer;
use tracer::{process_trace, return_branch_no};

macro_rules! dispatch {
    ($meta:expr, $body:expr) => {
        match $meta.alg {
            0  => { let bp = AlwaysTaken::new(); $body(bp) }
            1  => { let bp = TwoBit::<512>::new(); $body(bp) }
            2  => { let bp = TwoBit::<1024>::new(); $body(bp) }
            3  => { let bp = TwoBit::<2048>::new(); $body(bp) }
            4  => { let bp = TwoBit::<4096>::new(); $body(bp) }
            5  => { let bp = GShare::<512>::new(); $body(bp) }
            6  => { let bp = GShare::<1024>::new(); $body(bp) }
            7  => { let bp = GShare::<2048>::new(); $body(bp) }
            8  => { let bp = GShare::<4096>::new(); $body(bp) }
            9  => { let bp = Profiled::<512>::new($meta.profile.clone()); $body(bp) }
            10 => { let bp = Profiled::<1024>::new($meta.profile.clone()); $body(bp) }
            11 => { let bp = Profiled::<2048>::new($meta.profile.clone()); $body(bp) }
            12 => { let bp = Profiled::<4096>::new($meta.profile.clone()); $body(bp) }
            13 => { let bp = TwoBitSat::<512>::new(); $body(bp) }
            14 => { let bp = TwoBitSat::<1024>::new(); $body(bp) }
            15 => { let bp = TwoBitSat::<2048>::new(); $body(bp) }
            16 => { let bp = TwoBitSat::<4096>::new(); $body(bp) }
            17 => { let bp = GShareSat::<512>::new(); $body(bp) }
            18 => { let bp = GShareSat::<1024>::new(); $body(bp) }
            19 => { let bp = GShareSat::<2048>::new(); $body(bp) }
            20 => { let bp = GShareSat::<4096>::new(); $body(bp) }
            21 => { let bp = ProfiledSat::<512>::new($meta.profile.clone()); $body(bp) }
            22 => { let bp = ProfiledSat::<1024>::new($meta.profile.clone()); $body(bp) }
            23 => { let bp = ProfiledSat::<2048>::new($meta.profile.clone()); $body(bp) }
            24 => { let bp = ProfiledSat::<4096>::new($meta.profile.clone()); $body(bp) }
            _ => panic!("Unsupported Predictor type"),
        }
    };
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let trace_path: &String = &args[1];
    let bp_alg: &String = &args[2];

    let base_alg: u8 = match bp_alg.as_str() {
        "always"          => 0,
        "twobit"          => 1,
        "gshare"          => 5,
        "profiled"        => 9,
        "twobit_sat"      => 13,
        "gshare_sat"      => 17,
        "profiled_sat"    => 21,
        _ => panic!("Unsupported BP Algorithm"),
    };

    let mut alg = base_alg;

    let size: &String;
    if alg != 0 {
        size = &args[3];

        let size_offset: u8 = match size.as_str() {
            "512"  => 0,
            "1024" => 1,
            "2048" => 2,
            "4096" => 3,
            _ => panic!("Unsupported BP Table Size"),
        };

        alg = base_alg + size_offset;
    }

    let mut profile: Option<String> = None;
    if matches!(alg, 9 | 10 | 11 | 12 | 21 | 22 | 23 | 24) {
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
    println!("{mispredicts} mispredicts over {total_branches} conditional branches.");
    println!("\n{rate}% mispredict rate");

    if let Some(mut bias_data) = bp.get_bias_data() {
        bias_data.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        for (pc, pct, hard) in &bias_data {
            eprintln!("BIAS {:#018x} {:.6} {}", pc, pct, if *hard { "hard" } else { "easy" });
        }
    }
    println!();

    Ok(())
}

fn run_trace<P: Predictor>(bp: &mut P, trace_path: &str) {
    let _ = process_trace(trace_path, false, |entry| {
        bp.predict(&entry);
    });
}
