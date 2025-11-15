use crate::cli::{CliArgs, HeaderStyle};
use anyhow::{anyhow, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn run(args: CliArgs) -> Result<()> {
    let root = args.path;

    // Parallel scan
    let (file_count, total_size, all_files) =
        pre_scan(&root, args.verbose, args.no_confirm)?;

    //
    // Dry-run early return
    //
    if args.dry_run {
        println!("Dry-run: no files will be created.");
        println!("Detected {} files, total size {:.2} MB.",
            file_count,
            total_size as f64 / (1024.0 * 1024.0)
        );
        println!();
        println!("Files to be merged:");

        for f in &all_files {
            println!(" - {}", f.display());
        }

        println!();
        println!("Estimated output size: {:.2} MB",
            total_size as f64 / 1024.0 / 1024.0
        );

        return Ok(());
    }

    //
    // Filtering (extensions, exclude globs)
    //
    let exclude_set = build_exclude_set(&args.exclude)?;
    let follow_links = args.follow_symlinks;
    let split_every = args.split_every.unwrap_or(usize::MAX);
    let output_base =
        args.output.clone().unwrap_or_else(|| PathBuf::from("treemerge.txt"));

    // Build merging list with filtering
    let mut merge_files = Vec::new();
    for f in all_files {
        let rel = f.strip_prefix(&root).unwrap_or(&f);
        if is_excluded(rel, &exclude_set) {
            continue;
        }
        if !is_text_file(&f, &args.exts)? {
            continue;
        }
        merge_files.push(f);
    }

    //
    // Progress bar
    //
    let pb = if !args.verbose && !args.dry_run {
        let bar = make_progress_bar(merge_files.len() as u64);
        bar.set_message("Merging files");
        Some(bar)
    } else {
        None
    };

    //
    // Merge files
    //
    let mut current_lines = 0usize;
    let mut chunk_index = 1usize;
    let mut writer = open_writer(&output_base, chunk_index)?;

    for f in &merge_files {
        if let Some(pb) = &pb {
            pb.inc(1);
        }

        let rel = f.strip_prefix(&root).unwrap_or(f);

        // Count lines
        let file_line_count = count_lines(f)?;
        let header_line_count = header_line_count(&args.header_style);

        // Start new chunk if needed
        if current_lines + header_line_count + file_line_count > split_every {
            chunk_index += 1;
            writer = open_writer(&output_base, chunk_index)?;
            current_lines = 0;
        }

        // Write header
        write_header(&mut writer, rel, args.header_style)?;
        current_lines += header_line_count;

        // Write contents
        let file = File::open(f)?;
        let mut br = BufReader::new(file);
        let mut line = String::new();
        while br.read_line(&mut line)? > 0 {
            writer.write_all(line.as_bytes())?;
            current_lines += 1;
            line.clear();
        }
    }

    if let Some(pb) = pb {
        pb.finish_with_message("Done");
    }

    Ok(())
}

// ============================================================================
// Pre-scan (parallel)
// ============================================================================

fn pre_scan(path: &Path, verbose: bool, no_confirm: bool)
    -> Result<(usize, u64, Vec<PathBuf>)>
{
    const MAX_FILES: usize = 20_000;
    const MAX_FILE_SIZE: u64 = 200 * 1024 * 1024;
    const MAX_TOTAL: u64 = 4 * 1024 * 1024 * 1024;
    const MAX_DEPTH: usize = 20;
    const LARGE_OUTPUT_WARNING: u64 = 1 * 1024 * 1024 * 1024;

    let entries: Vec<_> = WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    let results = entries.par_iter().map(|entry| {
        let depth = entry.depth();

        if entry.file_type().is_file() {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            (true, size, depth)
        } else {
            (false, 0u64, depth)
        }
    }).collect::<Vec<_>>();

    let mut file_count = 0usize;
    let mut total_size = 0u64;
    let mut max_depth = 0usize;
    let mut large_files = Vec::new();

    for (is_file, size, depth) in &results {
        if *depth > max_depth {
            max_depth = *depth;
        }
        if *is_file {
            file_count += 1;
            total_size += size;
            if *size >= MAX_FILE_SIZE {
                large_files.push(*size);
            }
        }
    }

    let estimated_output = total_size + (file_count as u64 * 128);

    //
    // Warnings
    //
    let mut risky = false;

    if file_count > MAX_FILES {
        eprintln!("Warning: {} files detected.", file_count);
        risky = true;
    }

    if !large_files.is_empty() {
        eprintln!("Warning: {} files >= 200MB.", large_files.len());
        risky = true;
    }

    if total_size > MAX_TOTAL {
        eprintln!("Warning: total input size {:.2} GB.",
            total_size as f64 / 1024.0 / 1024.0 / 1024.0
        );
        risky = true;
    }

    if estimated_output > LARGE_OUTPUT_WARNING {
        eprintln!("Warning: estimated output {:.2} GB.",
            estimated_output as f64 / 1024.0 / 1024.0 / 1024.0
        );
        risky = true;
    }

    if max_depth > MAX_DEPTH {
        eprintln!("Warning: directory depth {}.", max_depth);
        risky = true;
    }

    if risky && !no_confirm {
        eprint!("Continue? [y/N]: ");
        io::stdout().flush().ok();

        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;

        let a = answer.trim().to_lowercase();
        if a != "y" && a != "yes" {
            return Err(anyhow!("Aborted by user"));
        }
    }

    // Collect files
    let mut files = Vec::new();
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }

    if verbose {
        eprintln!("Pre-scan: {} files, {:.2} MB total.",
            file_count,
            total_size as f64 / 1024.0 / 1024.0
        );
    }

    Ok((file_count, total_size, files))
}

// ============================================================================
// Helpers
// ============================================================================

fn build_exclude_set(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pat in patterns {
        builder.add(Glob::new(pat)?);
    }
    Ok(builder.build()?)
}

fn is_excluded(path: &Path, set: &GlobSet) -> bool {
    set.is_match(path)
}

fn is_text_file(path: &Path, exts: &[String]) -> Result<bool> {
    if !exts.is_empty() {
        if let Some(ext) = path.extension().and_then(|x| x.to_str()) {
            return Ok(exts.iter().any(|e| e.eq_ignore_ascii_case(ext)));
        } else {
            return Ok(false);
        }
    }

    let data = fs::read(path)?;
    Ok(infer::is_text(&data))
}

fn count_lines(path: &Path) -> Result<usize> {
    let file = File::open(path)?;
    let br = BufReader::new(file);
    Ok(br.lines().count())
}

fn open_writer(base: &Path, index: usize) -> Result<BufWriter<File>> {
    let path = if index == 1 {
        base.to_path_buf()
    } else {
        let stem = base.file_stem().unwrap().to_string_lossy();
        let ext = base.extension().map(|x| format!(".{}", x.to_string_lossy())).unwrap_or_default();
        let new = format!("{}_{}{}", stem, index, ext);
        base.with_file_name(new)
    };

    let f = File::create(path)?;
    Ok(BufWriter::new(f))
}

fn write_header(writer: &mut BufWriter<File>, path: &Path, style: HeaderStyle) -> Result<()> {
    let rel = path.to_string_lossy();

    match style {
        HeaderStyle::Plain => {
            writeln!(writer, "=== {} ===", rel)?;
        }
        HeaderStyle::Hash => {
            writeln!(writer, "## {}", rel)?;
        }
        HeaderStyle::Underline => {
            writeln!(writer, "{}", rel)?;
            writeln!(writer, "{}", "-".repeat(rel.len()))?;
        }
    }

    Ok(())
}

fn header_line_count(style: &HeaderStyle) -> usize {
    match style {
        HeaderStyle::Plain => 1,
        HeaderStyle::Hash => 1,
        HeaderStyle::Underline => 2,
    }
}

fn make_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} {pos}/{len} [{bar:40.cyan/blue}] {msg}"
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    pb
}
 
