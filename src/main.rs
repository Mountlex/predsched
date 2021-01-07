use anyhow::Result;
use sample::Cli;

mod sample;


#[paw::main]
fn main(args: Cli) -> Result<()> {
    args.sample()
}