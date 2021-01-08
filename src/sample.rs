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

    #[structopt(long = "max-job-len", default_value = "200.0")]
    max_job_len: f64,

    #[structopt(long = "num-lambdas", default_value = "5")]
    num_lambdas: usize,

    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf
}

#[derive(Debug, Serialize)]
struct Entry {
    lambda: f64,
    sigma: f64,
    opt: f64,
    arr: f64,
    two_stage: f64
}

impl Cli {
    pub fn sample(&self) -> Result<()> {
        let instance_params = InstanceGenParams {
            length: self.instance_length,
            min: 1.0,
            max: self.max_job_len,
        };
        let results: Vec<Entry> = (0..self.num_instances).into_par_iter().progress().flat_map(|_| {
            let instance = Instance::generate(&instance_params);
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
                        let arr = adaptive_round_robin(&instance, &pred, lambda);
                        let two_stage = two_stage_schedule(&instance, &pred, lambda);
                        Entry {
                            lambda,
                            sigma,
                            opt,
                            arr,
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
