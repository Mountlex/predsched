use crate::{Gen, Opt};
use rand::distributions::Distribution;
use rand_distr::Pareto;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Instance {
    pub jobs: Vec<f64>,
}

impl Instance {
    pub fn job_len(&self, idx: usize) -> f64 {
        self.jobs[idx]
    }

    pub fn num_jobs(&self) -> usize {
        self.jobs.len()
    }
}

impl From<Vec<f64>> for Instance {
    fn from(lengths: Vec<f64>) -> Self {
        Instance { jobs: lengths }
    }
}

impl Opt for Instance {
    fn opt(&self) -> f64 {
        let mut jobs = self.jobs.clone();
        jobs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = jobs.len();
        jobs.into_iter()
            .enumerate()
            .fold(0.0, |acc, (idx, job)| acc + (n - idx) as f64 * job)
    }
}

pub struct InstanceGenParams {
    pub length: usize,
    pub alpha: f64,
}

impl Gen<InstanceGenParams> for Instance {
    fn generate(params: &InstanceGenParams) -> Instance {
        let mut rng = rand::thread_rng();
        let dist = Pareto::new(1.0, params.alpha).unwrap();
        
        let jobs: Vec<f64> = dist
            .sample_iter(&mut rng)
            .take(params.length)
            .map(|j| j as f64)
            .collect();
        jobs.into()
    }
}

#[cfg(test)]
mod test_instance {
    use super::*;

    #[test]
    fn test_opt() {
        let instance: Instance = vec![1.0, 2.0, 4.0, 3.0].into();
        assert_eq!(20.0, instance.opt())
    }
}
