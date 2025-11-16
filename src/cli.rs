use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Header styles
#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug)]
pub enum HeaderStyle {
    Plain,
    Hash,
    Underline,
}

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(author, version, about = "Concatenate all text files in a directory tree.")]
pub struct Args {
    /// Root directory to process (must be a directory)
    pub path: PathBuf,

    /// Output file name; defaults to <dirname>.txt
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Glob patterns to include (repeatable)
    #[arg(short = 'i', long = "include")]
    pub include: Vec<String>,

    /// Glob patterns to exclude (repeatable)
    #[arg(short = 'x', long = "exclude")]
    pub exclude: Vec<String>,

    /// Only include files with these extensions
    #[arg(short = 'e', long = "ext")]
    pub ext: Vec<String>,

    /// Disable default excludes
    #[arg(long = "all-files")]
    pub all_files: bool,

    /// Line count after which to split output (never splits inside a file)
    #[arg(long = "split-every")]
    pub split_every: Option<usize>,

    /// Header style for file separators
    #[arg(long = "header-style", value_enum, default_value = "hash")]
    pub header_style: HeaderStyle,

    /// Dry-run mode (no files written)
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Skip confirmation prompts
    #[arg(long = "no-confirm")]
    pub no_confirm: bool,

    /// Follow symlinked directories
    #[arg(long = "follow-symlinks")]
    pub follow_symlinks: bool,

    /// Verbose logging
    #[arg(long = "verbose")]
    pub verbose: bool,
}
