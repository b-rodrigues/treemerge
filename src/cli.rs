use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Merge all text files in a directory tree.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliArgs {
    /// Path to scan
    pub path: PathBuf,

    /// Output file (default: treemerge.txt)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Exclude paths (glob). Can be repeated.
    #[arg(short = 'x', long = "exclude")]
    pub exclude: Vec<String>,

    /// Force-include paths (glob), even if excluded by default rules or --exclude.
    #[arg(short = 'i', long = "include")]
    pub include: Vec<String>,

    /// Only include files with these extensions (no dot). Can be repeated.
    #[arg(short = 'e', long = "ext")]
    pub exts: Vec<String>,

    /// Split output every N lines, but never break a file.
    #[arg(long = "split-every")]
    pub split_every: Option<usize>,

    /// Header style
    #[arg(long, value_enum, default_value_t=HeaderStyle::Plain)]
    pub header_style: HeaderStyle,

    /// Follow symlinks
    #[arg(long)]
    pub follow_symlinks: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Automatically continue even if risky path is detected (no prompt).
    #[arg(long)]
    pub no_confirm: bool,

    /// Show what would be done, without producing output files.
    #[arg(long)]
    pub dry_run: bool,

    /// Disable built-in default excludes (.git/, lockfiles, LICENSE, build outputs, etc.).
    #[arg(long = "all-files")]
    pub all_files: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HeaderStyle {
    Plain,
    Hash,
    Underline,
}
