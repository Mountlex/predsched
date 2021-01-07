use crate::{Gen, instance::Instance};
use rand_distr::{Distribution, Normal};

pub type Prediction = Instance;

pub struct PredGenParams<'a> {
    pub instance: &'a Instance,
    pub sigma: f64
}

impl Gen<PredGenParams<'_>> for Prediction {
    fn generate(params: &PredGenParams) -> Prediction {
        let mut rng = rand::thread_rng();
        
        let preds: Vec<f64> = params.instance.jobs.iter().map(|job| {
            let dist = Normal::new(*job, params.sigma).unwrap();
            dist.sample(&mut rng)
        }).collect();
        preds.into()
    }
}