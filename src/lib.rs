#![feature(drain_filter)]

mod instance;
mod pred;
mod algs;

pub trait Opt {
    fn opt(&self) -> f64;
}