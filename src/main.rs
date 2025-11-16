mod cli;
mod merge;

use anyhow::Result;
use clap::Parser;
use cli::CliArgs;

fn main() -> Result<()> {
    let args = CliArgs::parse();
    merge::run(args)
}
 
