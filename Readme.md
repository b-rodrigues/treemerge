# treemerge

`treemerge` recursively scans a directory tree, identifies plain-text files, and concatenates them into one or more output files. Each file is preceded by a header showing its relative path. The tool is designed for building reproducible text corpora, audits, and archival bundles.

## Features

* Safe and fast directory pre-scan using Rayon
* Progress bars for merging
* Dry-run mode (`--dry-run`)
* Optional file splitting (`--split-every N`) without breaking files
* Header styles (`plain`, `hash`, `underline`)
* Glob-based exclusion (`--exclude`)
* Extension filtering (`--ext`)
* Output size estimation + safety prompts
* Reproducible build via Nix flake

## Usage

```bash
treemerge [OPTIONS] <PATH>
```

### Common options

```
-o, --output <FILE>        Output file (default: treemerge.txt)
-x, --exclude <GLOB>       Exclude paths (repeatable)
-e, --ext <EXT>            Only include files with these extensions
--split-every <LINES>      Split output every N lines
--header-style <STYLE>     plain | hash | underline
--dry-run                  Show what would happen, no output written
--no-confirm               Skip safety confirmation prompts
--verbose                  Log processed files
```

## Example

```bash
treemerge ./data -o merged.txt --split-every 50000 \
  --exclude "*.log" --header-style underline
```

## Installation

### Nix (flake)

```bash
nix run github:b-rodrigues/treemerge
```

Or build:

```bash
nix build github:b-rodrigues/treemerge
```

### Cargo

```bash
cargo install treemerge
```

## License

GPL v3
