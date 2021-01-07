use crate::{instance::Instance, pred::Prediction};
use log::{trace, info};

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
        self.remaining == 0.0
    }
}

pub fn adaptive_round_robin(instance: &Instance, pred: &Prediction, lambda: f64) -> f64 {
    let mut jobs: Vec<Job> = instance.jobs.iter().zip(pred.jobs.iter()).enumerate().map(|(idx, (real, pred))| Job::new(idx, *real, *pred)).collect();
    jobs.sort_by(|a,b| a.predicted_length.partial_cmp(&b.predicted_length).unwrap());

    let mut env = ArrEnv {
        remaining_jobs: jobs,
        rr: vec![],
        time: 0.0,
        active_to_process: None,
        cost: 0.0,
        lambda
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
}

impl ArrEnv {
    fn simulate(&mut self) -> bool {
        let lambda = self.lambda;
        if !self.remaining_jobs.is_empty() {
            let active_job = self.remaining_jobs.first_mut().unwrap();
            let active_alg = self
                .active_to_process
                .unwrap_or_else(|| (1.0 - lambda) * active_job.predicted_length);

            let mut remaining_lengths: Vec<f64> = self.rr.iter().map(|j| j.remaining).collect();
            remaining_lengths.push(active_alg.min(active_job.total_length));
            let num_processing_jobs = remaining_lengths.len();
            let stepsize = remaining_lengths
                .into_iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            println!("Stepsize: {}", stepsize);
            self.time += stepsize * num_processing_jobs as f64; // divide rr equally

            active_job.process(stepsize);
            self.rr.iter_mut().for_each(|j| j.process(stepsize));
            let finished = self
                .rr
                .drain_filter(|j| if j.is_finished() { true } else { false });
            self.cost += finished.count() as f64 * self.time;

            if active_job.is_finished() {
                // active job finishes
                self.cost += self.time;
                println!("Finished job {} at time {}.", active_job.id, self.time);
                drop(active_job);
                self.remaining_jobs.remove(0);
                self.active_to_process = None;
            } else if stepsize == active_alg {
                // alg decides to move the job to rr
                let j = self.remaining_jobs.remove(0);
                self.rr.push(j);
                self.active_to_process = None;
            } else {
                // some rr finished
                self.active_to_process = Some(active_alg - stepsize);
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
                let finished = self
                    .rr
                    .drain_filter(|j| if j.is_finished() { true } else { false });
                self.cost += finished.count() as f64 * self.time;
                return true;
            } else {
                return false;
            }
        }
    }
}



#[cfg(test)]
mod test_algs {
    use crate::Opt;

    use super::*;

    #[test]
    fn test_rr() {
        let instance: Instance = vec![1.0, 2.0, 2.0].into(); 
        let pred = instance.clone();
        let alg = adaptive_round_robin(&instance, &pred, 1.0);
        assert_eq!(13.0, alg);
    }

    #[test]
    fn test_ftp() {
        let instance: Instance = vec![1.0, 2.0, 3.0].into(); 
        let pred = instance.clone();
        let alg = adaptive_round_robin(&instance, &pred, 0.0);
        assert_eq!(10.0, alg);
    }
  
    #[test]
    fn test_arr() {
        let instance: Instance = vec![2.0, 4.0].into(); 
        let pred = instance.clone();
        let alg = adaptive_round_robin(&instance, &pred, 0.5);
        assert_eq!(9.0, alg);
    }

    #[quickcheck]
    fn check_consistency(jobs: Vec<u64>) -> bool {
        let instance: Instance = jobs.into_iter().map(|j| j as f64).collect::<Vec<f64>>().into(); 
        let pred = instance.clone();
        instance.opt() == adaptive_round_robin(&instance, &pred, 0.0)
    }

  
}