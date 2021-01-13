use core::f64;
use std::path::PathBuf;

use anyhow::Result;
use indicatif::ParallelProgressIterator;
use itertools_num::linspace;
use predsched::*;
use rayon::prelude::*;
use structopt::StructOpt;
use serde::Serialize;
use csv::Writer;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(short = "l", long, default_value = "10")]
    instance_length: usize,

    #[structopt(short = "n")]
    num_instances: usize,

    #[structopt(short = "p", default_value = "5")]
    num_preds: usize,

    #[structopt(long = "step-sigma", default_value = "10.0")]
    step_sigma: f64,

    #[structopt(long = "num-sigma", default_value = "10")]
    num_sigmas: usize,

    #[structopt(short, long = "alpha", default_value = "1.1")]
    alpha: f64,

    #[structopt(long = "num-lambdas", default_value = "5")]
    num_lambdas: usize,

    #[structopt(short, long)]
    equal_share: bool,

    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf
}

#[derive(Debug, Serialize)]
struct Entry {
    lambda: f64,
    sigma: f64,
    opt: f64,
    arr: f64,
    prr: f64,
    two_stage: f64
}

impl Cli {
    pub fn sample(&self) -> Result<()> {
        let instance_params = InstanceGenParams {
            length: self.instance_length,
            alpha: self.alpha
        };
        let instances: Vec<Instance> = (0..self.num_instances).map(|_| 
            Instance::generate(&instance_params)
        ).collect();
        analyse_instances(&instances);
        let results: Vec<Entry> = instances.into_par_iter().progress_count(self.num_instances as u64).flat_map(|instance| {
            let opt = instance.opt();
            (0..self.num_sigmas).flat_map(|sigma_num| {
                let sigma = self.step_sigma * sigma_num as f64;
                (0..self.num_preds).flat_map(|_| {     
                    let pred_params = PredGenParams {
                        sigma,
                        instance: &instance
                    };
                    let pred = Prediction::generate(&pred_params);
                    linspace(0.0, 1.0, self.num_lambdas).map(|lambda| {
                        let pred = pred.clone();
                        let arr = adaptive_round_robin(&instance, &pred, lambda, self.equal_share);
                        let two_stage = two_stage_schedule(&instance, &pred, lambda);
                        let prr = preferential_round_robin(&instance, &pred, lambda);

                        if lambda == 0.0 && arr != prr {
                          //  println!("instance: {:?}, pred: {:?}, arr: {}, prr: {}", instance, pred, arr, prr);
                        }

                        Entry {
                            lambda,
                            sigma,
                            opt,
                            arr,
                            prr,
                            two_stage
                        }
                    }).collect::<Vec<Entry>>()
                }).collect::<Vec<Entry>>()
            }).collect::<Vec<Entry>>()
        }).collect();

        export(&self.output, results)
    }
}

fn export(output: &PathBuf, results: Vec<Entry>) -> Result<()> {
    let mut wtr = Writer::from_path(output)?;
    for entry in results {
        wtr.serialize(entry)?;
    }
    Ok(())
}
