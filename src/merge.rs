use crate::cli::{Args, HeaderStyle};
use anyhow::{anyhow, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

/// Build a GlobSet from patterns
fn compile_globs(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        builder.add(Glob::new(p).context("Invalid glob pattern")?);
    }
    Ok(builder.build()?)
}

/// Check if a file looks like text using infer + UTF-8 heuristic
fn is_text_file(path: &Path, allowed_exts: &[String]) -> Result<bool> {
    // extension allowlist (fast path)
    if !allowed_exts.is_empty() {
        if let Some(ext) = path.extension().and_then(|x| x.to_str()) {
            return Ok(allowed_exts.iter().any(|e| e.eq_ignore_ascii_case(ext)));
        } else {
            return Ok(false);
        }
    }

    // content detection
    const BUF_SIZE: usize = 8192;
    let mut file = File::open(path)?;
    let mut buf = [0u8; BUF_SIZE];
    let n = file.read(&mut buf)?;

    if n == 0 {
        return Ok(false);
    }

    if let Some(kind) = infer::get(&buf[..n]) {
        if kind.mime_type().starts_with("text/") {
            return Ok(true);
        }
    }

    Ok(std::str::from_utf8(&buf[..n]).is_ok())
}

/// List of built-in excludes
fn default_excludes() -> Vec<String> {
    vec![
        // VCS
        ".git/**".into(),
        ".svn/**".into(),
        ".hg/**".into(),
        // build dirs
        "target/**".into(),
        "dist/**".into(),
        "build/**".into(),
        "out/**".into(),
        // caches
        "__pycache__/**".into(),
        ".cache/**".into(),
        ".mypy_cache/**".into(),
        ".pytest_cache/**".into(),
        ".venv/**".into(),
        ".idea/**".into(),
        ".vscode/**".into(),
        "node_modules/**".into(),
        // docs output
        "_site/**".into(),
        "_book/**".into(),
        "docs/_build/**".into(),
        // boilerplate
        "LICENSE".into(),
        "LICENSE.*".into(),
        "COPYING".into(),
        "NOTICE".into(),
        // lockfiles
        "*.lock".into(),
        "package-lock.json".into(),
        "poetry.lock".into(),
        "Pipfile.lock".into(),
        "pnpm-lock.yaml".into(),
        "yarn.lock".into(),
        // binaries
        "*.pyc".into(),
        "*.pyo".into(),
        "*.o".into(),
        "*.so".into(),
        "*.dll".into(),
        "*.exe".into(),
    ]
}

/// Determine whether a given path should be included
fn should_include(
    path: &Path,
    includes: &GlobSet,
    excludes: &GlobSet,
    builtin_excludes: &GlobSet,
    all_files: bool,
) -> bool {
    let s = path.to_string_lossy();

    if includes.is_match(&*s) {
        return true;
    }

    if excludes.is_match(&*s) {
        return false;
    }

    if !all_files && builtin_excludes.is_match(&*s) {
        return false;
    }

    true
}

/// Format header for each file
fn write_header<W: Write>(w: &mut W, style: HeaderStyle, path: &Path) -> Result<()> {
    let s = path.to_string_lossy();

    match style {
        HeaderStyle::Plain => {
            writeln!(w, ">>> {}", s)?;
        }
        HeaderStyle::Hash => {
            writeln!(w, "########## {}", s)?;
        }
        HeaderStyle::Underline => {
            writeln!(w, "{}", s)?;
            writeln!(w, "{}", "=".repeat(s.len()))?;
        }
    }

    writeln!(w)?;
    Ok(())
}

pub fn run(args: Args) -> Result<()> {
    let root = &args.path;

    // Only directories allowed
    if !root.is_dir() {
        return Err(anyhow!(
            "treemerge only operates on directories: {}",
            root.display()
        ));
    }

    // Determine default output
    let output_base = if let Some(o) = &args.output {
        o.clone()
    } else {
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("treemerge");
        PathBuf::from(format!("{}.txt", name))
    };

    // Compile glob sets
    let include_globs = compile_globs(&args.include)?;
    let exclude_globs = compile_globs(&args.exclude)?;
    let builtin_globs = if args.all_files {
        compile_globs(&[])? // empty
    } else {
        compile_globs(&default_excludes())?
    };

    // Scan directory tree
    let walker = WalkDir::new(root).follow_links(args.follow_symlinks);
    let entries: Vec<DirEntry> = walker.into_iter().filter_map(|e| e.ok()).collect();

    let files: Vec<PathBuf> = entries
        .par_iter()
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            should_include(
                &entry.path(),
                &include_globs,
                &exclude_globs,
                &builtin_globs,
                args.all_files,
            )
        })
        .filter_map(|entry| {
            let path = entry.path();
            match is_text_file(path, &args.ext) {
                Ok(true) => Some(path.to_owned()),
                _ => None,
            }
        })
        .collect();

    if files.is_empty() {
        return Err(anyhow!("No text files matched criteria."));
    }

    // Estimate output size
    let estimated: u64 = files
        .par_iter()
        .map(|p| fs::metadata(p).map(|m| m.len()).unwrap_or(0))
        .sum();

    if !args.no_confirm && estimated > 500 * 1024 * 1024 {
        return Err(anyhow!(
            "estimated output exceeds 500MB; use --no-confirm to bypass."
        ));
    }

    if args.dry_run {
        println!("Dry-run. Would merge {} files:", files.len());
        for f in &files {
            println!("{}", f.display());
        }
        return Ok(());
    }

    // Progress bar
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("[{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    let mut current_lines = 0usize;
    let mut file_index = 0usize;

    let mut out = BufWriter::new(File::create(&output_base)?);

    for file in &files {
        pb.inc(1);
        pb.set_message(format!("{}", file.display()));

        write_header(&mut out, args.header_style, file)?;

        let mut reader = BufReader::new(File::open(file)?);

        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line)?;
            if n == 0 {
                break;
            }

            out.write_all(line.as_bytes())?;
            current_lines += 1;
        }

        // Check splitting
        if let Some(limit) = args.split_every {
            if current_lines >= limit {
                out.flush()?;
                file_index += 1;
                current_lines = 0;
                let next_name = output_base
                    .with_file_name(format!("{}.part{}", output_base.display(), file_index));
                out = BufWriter::new(File::create(next_name)?);
            }
        }
    }

    pb.finish_with_message("done");

    Ok(())
}
