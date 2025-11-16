mod cli;
mod merge;

use anyhow::Result;
use clap::Parser;
use cli::Args;

fn main() -> Result<()> {
    let args = Args::parse();
    merge::run(args)
}
