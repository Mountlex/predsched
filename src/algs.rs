use float_cmp::approx_eq;

use crate::{instance::Instance, pred::Prediction, Opt};

#[derive(Clone, Debug)]
pub struct Job {
    id: usize,
    total_length: f64,
    predicted_length: f64,
    remaining: f64,
}

impl Job {
    pub fn new(id: usize, len: f64, pred: f64) -> Self {
        Job {
            id,
            total_length: len,
            predicted_length: pred,
            remaining: len,
        }
    }

    pub fn process(&mut self, time: f64) {
        self.remaining -= time;
    }

    pub fn is_finished(&self) -> bool {
        self.remaining <= 0.0001
    }
}

pub fn adaptive_round_robin(instance: &Instance, pred: &Prediction, lambda: f64, equal_share: bool) -> f64 {
    let mut jobs: Vec<Job> = instance
        .jobs
        .iter()
        .zip(pred.jobs.iter())
        .enumerate()
        .map(|(idx, (real, pred))| Job::new(idx, *real, *pred))
        .collect();
    jobs.sort_by(|a, b| a.predicted_length.partial_cmp(&b.predicted_length).unwrap());

    let mut env = ArrEnv {
        remaining_jobs: jobs,
        rr: vec![],
        time: 0.0,
        active_to_process: None,
        cost: 0.0,
        lambda,
        equal_share
    };

    while env.simulate() {}

    env.cost
}

struct ArrEnv {
    /// The first job is currently active; sorted by predicted processing time.
    remaining_jobs: Vec<Job>,

    /// rr queue (idx, remaining to process)
    rr: Vec<Job>,

    /// current time
    time: f64,

    /// to process by algorithm
    active_to_process: Option<f64>,

    cost: f64,

    lambda: f64,

    equal_share: bool,
}

impl ArrEnv {
    fn simulate(&mut self) -> bool {
        let lambda = self.lambda;
        if !self.remaining_jobs.is_empty() {
            let num_unfinished_jobs = self.remaining_jobs.len() + self.rr.len();
            let num_remaining_jobs = self.remaining_jobs.len();

            let active_scale = if self.equal_share { 1.0 } else { num_unfinished_jobs as f64 / num_remaining_jobs as f64 };
            let rr_scale = if self.equal_share { 1.0 } else { num_unfinished_jobs as f64 };

            let active_job = self.remaining_jobs.first_mut().unwrap();
            let active_alg = self
                .active_to_process
                .unwrap_or_else(|| (1.0 - lambda) * active_job.predicted_length);


            let mut remaining_lengths: Vec<f64> = self.rr.iter().map(|j| j.remaining * rr_scale).collect();
            remaining_lengths.push(active_alg.min(active_job.total_length) * active_scale);
            let num_processing_jobs = remaining_lengths.len();
            
            let stepsize = remaining_lengths
                .into_iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            //println!("Stepsize: {}", stepsize);
            self.time += if self.equal_share { stepsize * num_processing_jobs as f64 } else { stepsize };
            active_job.process(stepsize / active_scale);
            self.rr.iter_mut().for_each(|j| j.process(stepsize / rr_scale));
            let finished = self.rr.drain_filter(|j| j.is_finished());
            self.cost += finished.count() as f64 * self.time;

            //println!("Time: {}", self.time);
            if active_job.is_finished() {
                // active job finishes
                self.cost += self.time;
                //println!("Finished job {} at time {}.", active_job.id, self.time);
                drop(active_job);
                self.remaining_jobs.remove(0);
                self.active_to_process = None;
            } else if approx_eq!(f64, stepsize / active_scale, active_alg, ulps = 1) {
                // alg decides to move the job to rr
                let j = self.remaining_jobs.remove(0);
                self.rr.push(j);
                self.active_to_process = None;
            } else {
                // some rr finished
                self.active_to_process = Some(active_alg - (stepsize / active_scale));
            }
            return true;
        } else {
            if !self.rr.is_empty() {
                // RR remaining jobs
                let stepsize = self
                    .rr
                    .iter()
                    .min_by(|a, b| a.remaining.partial_cmp(&b.remaining).unwrap())
                    .unwrap()
                    .remaining;
                self.time += stepsize * self.rr.len() as f64; // divide rr equally

                self.rr.iter_mut().for_each(|j| j.process(stepsize));
                let finished = self.rr.drain_filter(|j| j.is_finished());
                self.cost += finished.count() as f64 * self.time;
                return true;
            } else {
                return false;
            }
        }
    }
}

pub fn two_stage_schedule(instance: &Instance, pred: &Prediction, lambda: f64) -> f64 {
    let mut jobs: Vec<Job> = instance
        .jobs
        .iter()
        .zip(pred.jobs.iter())
        .enumerate()
        .map(|(idx, (real, pred))| Job::new(idx, *real, *pred))
        .collect();
    jobs.sort_by(|a, b| a.predicted_length.partial_cmp(&b.predicted_length).unwrap());

    let mut time = 0.0;
    let mut cost = 0.0;
    let mut misprediction_detected = false;

    let max_rr = lambda * pred.num_jobs() as f64 * pred.opt()
        / num_integer::binomial(instance.num_jobs(), 2) as f64;

    // RR until time == max_rr
    while !jobs.is_empty() && (time < max_rr || misprediction_detected) {
        let n = jobs.len() as f64;
        let mut stepsize = jobs
            .iter()
            .min_by(|a, b| a.remaining.partial_cmp(&b.remaining).unwrap())
            .unwrap()
            .remaining;
        
        //println!("Time: {}, Stepsize: {}", time, stepsize);
        if time + stepsize * n > max_rr {
            stepsize = (max_rr - time) / n;
        }
        if stepsize == 0.0 {
            break;
        }

        time += stepsize * n; // divide rr equally

        jobs.iter_mut().for_each(|j| j.process(stepsize));
        let finished = jobs.drain_filter(|j| {
            if j.is_finished() {
                if !approx_eq!(f64, j.predicted_length, j.total_length) {
                    misprediction_detected = true;
                }
                true
            } else {
                false
            }
        });
        cost += finished.count() as f64 * time;
    }
    assert!(time <= max_rr || !misprediction_detected);

    // Schedule remaining jobs in predicted order; break if a misprediction is being detected.
    while !misprediction_detected && !jobs.is_empty() {
        let job = jobs.remove(0);
        time += job.remaining;
        cost += time;
        if job.total_length != job.predicted_length {
            misprediction_detected = true;
        }
    }

    // RR until all jobs finish.
    while !jobs.is_empty() {
        let n = jobs.len() as f64;
        let stepsize = jobs
            .iter()
            .min_by(|a, b| a.remaining.partial_cmp(&b.remaining).unwrap())
            .unwrap()
            .remaining;

        time += stepsize * n; // divide rr equally

        jobs.iter_mut().for_each(|j| j.process(stepsize));
        let finished = jobs.drain_filter(|j| j.is_finished());
        cost += finished.count() as f64 * time;
    }

    cost
}

pub fn preferential_round_robin(instance: &Instance, pred: &Prediction, lambda: f64, stepsize: f64) -> f64 {
    let mut jobs: Vec<Job> = instance
        .jobs
        .iter()
        .zip(pred.jobs.iter())
        .enumerate()
        .map(|(idx, (real, pred))| Job::new(idx, *real, *pred))
        .collect();
    jobs.sort_by(|a, b| a.predicted_length.partial_cmp(&b.predicted_length).unwrap());

    let mut time: f64 = 0.0;
    let mut cost: f64 = 0.0;

    while !jobs.is_empty() {
        let num_jobs = jobs.len();
        let active = jobs.first_mut().unwrap();

        time += stepsize;
        active.process((1.0 - lambda) * stepsize);
        if active.is_finished() {
            drop(active);
            jobs.remove(0);
            cost += time;
        }
        jobs.iter_mut().for_each(|j| j.process((lambda / num_jobs as f64) * stepsize));
        let finished = jobs.drain_filter(|j| j.is_finished());
        cost += finished.count() as f64 * time;
    }

    cost 
}

#[cfg(test)]
mod test_algs {
    use crate::Opt;

    use super::*;

    #[test]
    fn test_arr_rr() {
        let instance: Instance = vec![1.0, 2.0, 2.0].into();
        let pred = instance.clone();
        let alg = adaptive_round_robin(&instance, &pred, 1.0, true);
        assert_eq!(13.0, alg);
    }

    #[test]
    fn test_arr_ftp() {
        let instance: Instance = vec![1.5, 2.5, 3.5].into();
        let pred = instance.clone();
        let alg = adaptive_round_robin(&instance, &pred, 0.0, true);
        assert_eq!(instance.opt(), alg);
    }

    #[test]
    fn test_arr() {
        let instance: Instance = vec![2.0, 4.0].into();
        let pred = instance.clone();
        let alg = adaptive_round_robin(&instance, &pred, 0.5, false);
        assert_eq!(9.0, alg);
    }

    #[quickcheck]
    fn check_arr_consistency(jobs: Vec<f64>) -> bool {
        let instance: Instance = jobs
            .into_iter()
            .map(|j| j.abs())
            .collect::<Vec<f64>>()
            .into();
        let pred = instance.clone();
        approx_eq!(f64, instance.opt(), adaptive_round_robin(&instance, &pred, 0.0, true))
    }

    #[test]
    fn test_two_stage_rr() {
        let instance: Instance = vec![1.0, 2.0, 2.0].into();
        let pred = instance.clone();
        let alg = two_stage_schedule(&instance, &pred, 1.0);
        assert_eq!(13.0, alg);
    }

    #[test]
    fn test_two_stage_ftp() {
        let instance: Instance = vec![1.0, 2.0, 3.0].into();
        let pred = instance.clone();
        let alg = two_stage_schedule(&instance, &pred, 0.0);
        assert_eq!(10.0, alg);
    }

    #[test]
    fn test_two_stage() {
        let instance: Instance = vec![2.0, 4.0].into();
        let pred = instance.clone();
        let alg = two_stage_schedule(&instance, &pred, 0.5);
        assert_eq!(10.0, alg);
    }

    #[quickcheck]
    fn check_two_stage_consistency(jobs: Vec<u64>) -> bool {
        let instance: Instance = jobs
            .into_iter()
            .map(|j| j as f64)
            .collect::<Vec<f64>>()
            .into();
        let pred = instance.clone();
        instance.opt() == two_stage_schedule(&instance, &pred, 0.0)
    }

    #[test]
    fn test_prr_rr() {
        let instance: Instance = vec![1.0, 2.0, 2.0].into();
        let pred = instance.clone();
        let alg = preferential_round_robin(&instance, &pred, 1.0, 0.1);
        assert_eq!(13.0, alg);
    }

    #[test]
    fn test_prr_ftp() {
        let instance: Instance = vec![1.5, 2.0, 3.0].into();
        let pred = instance.clone();
        let alg = preferential_round_robin(&instance, &pred, 0.0, 0.1);
        assert_eq!(11.5, alg);
    }

 
}
