#![feature(drain_filter)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod instance;
mod pred;
mod algs;

pub trait Opt {
    fn opt(&self) -> f64;
}