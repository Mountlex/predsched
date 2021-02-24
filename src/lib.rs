#![feature(drain_filter)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod instance;
mod pred;
mod algs;

mod tables;
mod rtnorm;

pub use instance::*;
pub use pred::*;
pub use algs::*;


pub trait Opt {
    fn opt(&self) -> f64;
}


pub trait Gen<P> {
    fn generate(params: &P) -> Self;
}