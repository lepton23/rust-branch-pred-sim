use super::predictor::Predictor;
use super::table::SaturatingTable;
use crate::tracer::{TraceEntry, process_trace};

use rustc_hash::FxHashMap;

// const functions used for branch prediction hints in cold if statement paths
#[inline]
#[cold]
const fn cold() {}

#[inline]
const fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
}

/*
    === Possible Approaches ===
    - Pre-warmed GShare: initialise branch history table from profiled run, so predictor starts "warm"
    - Profile-based hybrid selection: profile data identifies which branches are highly biased (better for static prediction), vs branches better for dynamic prediction.
                                      uses a meta predictor or compile-time selector to choose which predictor to use per branch
    - Path profile seeding: use Ball-Larus path profiling to capture correlations across multiple branches along a path. This will seed a path-based predictor (e.g. path-indexed PHT)

    Agbias predictor:
    - GHR Filtering: Split profiled branches into strongly biased (easy), which have the same outcome 95% of the time, and not strongly biased (hard) sub-streams.
                     Only hard sub-stream branches will affect the GHR, saving about 70% space. This helps to reduce destructive aliasing between branches.

    - Hard branches use normal dynamic predictor (use agree predictor),
      easy branches use separate smaller dynamic predictor so that they
      do not affect hard branches but are not fully dependent on profile data


    === Papers to mention in report ===
    Branch Prediction Using Profile Data - Alex Ramirez, Josep L. Larriba-Pey, and Mateo Valero                        - Details the `agbias` predictor and GHR Filtering
    Fisher & Freudenberger, ASPLOS 1992 — "Predicting Conditional Branch Directions From Previous Runs of a Program" - Foundational paper for profile-based static prediction
    Calder et al., TOPLAS 1997 — "Evidence-Based Static Branch Prediction Using Machine Learning"                    - Extends profiling with feature-based learning
    Ball & Larus, MICRO 1996 — "Efficient Path Profiling"                                                            - The standard algorithm for collecting path profiles
    Chang, Mahlke, Hwu, MICRO 1992 — "Using Profile Information to Assist Classic Code Optimizations"                - Profile use in compiler-directed prediction
    McFarling, WRL Tech Note TN-36, 1993 — "Combining Branch Predictors"                                             - Background on hybrid predictors
    Eric Sprangle et al., 1997. The agree predictor: a mechanism for reducing negative branch history interference. (May 1997) https://doi.org/10.1145/384286.264210
*/

pub struct Profiled<const N: usize> {
    mispredicts: u64,
    gshare: SaturatingTable<N>,
    easy_table: SaturatingTable<N>,
    ghr: u64, // shift register of last log2(N) outcomes
    index_mask: u64,
    biases: FxHashMap<u64, (bool, bool)>,
}

// TODO: possibly add separate GHR mask which is smaller than index_mask
// assuming here that we are using the most GHR possible - i.e. the same amount of GHR bits as the PHT index
impl<const N: usize> Profiled<N> {
    pub fn new(profile: Option<String>) -> Self {
        Self {
            mispredicts: 0,
            gshare: SaturatingTable::new(),
            easy_table: SaturatingTable::new(),
            ghr: 0,
            index_mask: (N as u64) - 1,
            biases: Self::profile_run(profile.unwrap()),
        }
    }

    fn profile_run(profile_path: String) -> FxHashMap<u64, (bool, bool)> {
        let mut totals: FxHashMap<u64, (u64, u64)> = FxHashMap::default();
        let _ = process_trace(&profile_path, |entry| {
            let count = totals.entry(entry.pc_addr).or_default();
            if entry.taken { 
                count.0 += 1;
            }
            count.1 += 1;
        });

        totals.into_iter()
            .map(|(count , (taken, total))| 
                (count, 
                (taken > (total / 2), 
                (taken as f64 / total as f64) < 0.95 
                    && (taken as f64 / total as f64) > 0.05)))
            .collect()
    }

    fn predict_branch(&mut self, pc_addr: u64, line_taken: bool, hard_branch: bool) -> bool {
        // xor the pc_addr and ghr to index into BP table
        if hard_branch {
            let index: u64 = (pc_addr ^ self.ghr) & self.index_mask;
            let taken: bool = self.gshare.table[index as usize].predict(line_taken);
            // store the last log2(N) bits of taken/not taken results
            self.ghr = ((self.ghr << 1) | line_taken as u64) & self.index_mask;
            taken
        } else {
            let index: u64 = pc_addr & self.index_mask;
            let taken: bool = self.easy_table.table[index as usize].predict(line_taken);
            taken
        }
    }
} // impl Profiled<N>

impl<const N: usize> Predictor for Profiled<N> {
    fn predict(&mut self, line: &TraceEntry) {
        let (bias, hard) = self.biases.get(&line.pc_addr).copied().unwrap();
        let agree = if line.taken {
            bias
        } else {
            !bias
        };

        let taken = if self.predict_branch(line.pc_addr, agree, hard) {
            bias
        } else {
            !bias
        };
        if unlikely(taken != line.taken) {
            self.mispredicts += 1;
        }

    }

    fn get_mispredicts(&self) -> u64 {
        self.mispredicts
    }
}
