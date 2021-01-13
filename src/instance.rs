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

pub fn analyse_instances(instances: &Vec<Instance>) {
    let flat: Vec<f64> = instances.iter().flat_map(|instance| instance.jobs.clone()).collect();
    println!("Instance Generation Summary:");
    println!("  Mean: {}", mean(&flat).unwrap());
    println!("  StdDev: {}", std_deviation(&flat).unwrap());
    println!("  Max: {}", flat.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
}

fn mean(data: &[f64]) -> Option<f64> {
    let sum = data.iter().sum::<f64>() as f64;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f64),
        _ => None,
    }
}

fn std_deviation(data: &[f64]) -> Option<f64> {
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = data.iter().map(|value| {
                let diff = data_mean - (*value as f64);

                diff * diff
            }).sum::<f64>() / count as f64;

            Some(variance.sqrt())
        },
        _ => None
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
