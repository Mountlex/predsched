use crate::Opt;


#[derive(Clone, Debug, Default, PartialEq)]
pub struct Instance {
    pub jobs: Vec<f64>
}

impl Instance {
    pub fn job_len(&self, idx: usize) -> f64 {
        self.jobs[idx]
    }

    pub fn num_jobs(&self) -> usize {
        self.jobs.len()
    }
}

impl Opt for Instance {
    fn opt(&self) -> f64 {
        let mut jobs = self.jobs.clone();
        jobs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = jobs.len();
        jobs.into_iter().enumerate().fold(0.0, |acc, (idx, job)| {
            acc + (n - idx) as f64  * job
        })
    }
}