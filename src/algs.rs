use core::num;

use crate::{instance::Instance, pred::Prediction};

pub fn adaptive_round_robin(instance: &Instance, pred: &Prediction, lambda: f32) {
    let mut predicted: Vec<(usize, f64)> = pred.jobs.clone().into_iter().enumerate().collect();
    predicted.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
    let mut remaining = instance.jobs.len();

    let mut time: u64 = 0;
    let mut cost: u64 = 0;

    for (idx, len) in predicted {}
}

struct ArrEnv<'a> {
    instance: &'a Instance,
    /// The first job is currently active; sorted by predicted processing time.
    remaining_jobs: Vec<(usize, f64)>,

    /// rr queue (idx, remaining to process)
    rr: Vec<(usize, f64)>,

    /// current time
    time: f64,

    /// (to process by algorithm / to process by real processing time)
    active_to_process: Option<(f64, f64)>,

    cost: f64,

    lambda: f64,
}

impl ArrEnv<'_> {
    fn simulate(&mut self) -> bool {
        if !self.remaining_jobs.is_empty() {
            let (idx, len) = self.remaining_jobs.first().unwrap();
            let real_len = self.instance.job_len(*idx);
            let (active_alg, active_real) = self
                .active_to_process
                .unwrap_or_else(|| (self.lambda * len, real_len));

            let mut processed_jobs = self.rr.clone();
            let num_processed_jobs = processed_jobs.len();
            processed_jobs.push((*idx, active_alg.min(active_real)));
            let (_, stepsize) = processed_jobs
                .into_iter()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .unwrap();
            self.time += stepsize * num_processed_jobs as f64; // divide rr equally

            for (_, mut remaining) in self.rr.iter() {
                remaining -= stepsize;
            }
            self.rr.drain_filter(|(_, remaining)| {
                if *remaining == 0.0 {
                    self.cost += self.time;
                    true
                } else {
                    false
                }
            });

            if stepsize == active_real {
                // active job finishes
                self.cost += self.time;
                self.active_to_process == None;
            } else if stepsize == active_alg {
                // alg decides to move the job to rr
                self.rr.push((*idx, active_real - stepsize));
                self.active_to_process == None;
                self.remaining_jobs.remove(0);
            } else {
                // some rr finished
                self.active_to_process = Some((active_alg - stepsize, active_real - stepsize));
            }
        } else {
            // RR remaining jobs
            let (_, stepsize) = self
                .rr
                .iter()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .unwrap();
            self.time += stepsize * self.rr.len() as f64; // divide rr equally

            for (_, mut remaining) in self.rr.iter() {
                remaining -= stepsize;
            }
            self.rr.drain_filter(|(_, remaining)| {
                if *remaining == 0.0 {
                    self.cost += self.time;
                    true
                } else {
                    false
                }
            });
        }
        false
    }
}
