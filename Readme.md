# treemerge

*Caveat emptor: The code of this project is 100% clanker-generated.*

`treemerge` recursively scans a directory tree, identifies plain-text files, and concatenates them into one or more output files. Each file is preceded by a header showing its relative path. The tool is designed for building reproducible text corpora, audits, and archival bundles — especially for supplying contextual text to LLMs.

## Features

* Safe and fast directory pre-scan using Rayon  
* Progress bars for merging  
* Dry-run mode (`--dry-run`)  
* Optional file splitting (`--split-every N`) without breaking files  
* Header styles (`plain`, `hash`, `underline`)  
* Glob-based inclusion (`--include`) and exclusion (`--exclude`)  
* Smart defaults: ignores `.git/`, lockfiles, license files, build outputs, caches, etc.  
* `--all-files` to disable default ignore rules  
* Extension filtering (`--ext`)  
* Output size estimation + safety prompts  
* Reproducible build via Nix flake  

## Usage

```bash
treemerge [OPTIONS] <PATH>
```

### Common options

```
-o, --output <FILE>       Output file (default: treemerge.txt)
-i, --include <GLOB>      Force-include paths (repeatable)
-x, --exclude <GLOB>      Exclude paths (repeatable)
-e, --ext <EXT>           Only include files with these extensions
--split-every <LINES>     Split output every N lines (never splits inside a file)
--header-style <STYLE>    plain | hash | underline
--all-files               Disable default excludes (.git/, lockfiles, LICENSE, etc.)
--dry-run                 Show what would happen, no output written
--no-confirm              Skip safety confirmation prompts
--follow-symlinks         Follow symlinks during traversal
--verbose                 Log processed files
```

## Default excludes

`treemerge` automatically ignores these unless `--all-files` is provided:

* Version control metadata: `.git/`, `.svn/`, `.hg/`
* Build/dist directories: `target/`, `build/`, `dist/`, `out/`
* Caches/environments: `__pycache__/`, `.venv/`, `.cache/`, `.mypy_cache/`, `.pytest_cache/`, `.idea/`, `.vscode/`, `node_modules/`
* Documentation builds: `_site/`, `_book/`, `docs/_build/`
* Licenses and legal boilerplate: `LICENSE`, `LICENSE.*`, `COPYING`, `NOTICE`
* Lockfiles: `*.lock`, `Pipfile.lock`, `poetry.lock`, `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`
* Common compiled/binary artifacts: `*.pyc`, `*.pyo`, `*.o`, `*.so`, `*.dylib`, `*.dll`, `*.exe`

Use:
- `--include` to force-include a pattern (overrides all excludes)
- `--exclude` to add additional exclusions
- `--all-files` to disable defaults entirely

## Examples

Merge all text files:

```bash
treemerge ./data
```

Include `.lock` files even though they’re ignored by default:

```bash
treemerge . --include "*.lock"
```

Split output every 50,000 lines and exclude logs:

```bash
treemerge ./src --split-every 50000 --exclude "*.log"
```

Disable all default excludes and include absolutely everything:

```bash
treemerge . --all-files
```

Dry-run without writing output:

```bash
treemerge ./corpus --dry-run
```

## Installation

### Prebuilt binaries

Precompiled binaries for Linux and macOS are available on the  
**GitHub Releases page**:  
<https://github.com/b-rodrigues/treemerge/releases>

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
