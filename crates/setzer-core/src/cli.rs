//! The one command-line option of a site program: `--out <dir>`.
//!
//! Default: `<target>/site`, next to cargo's own build output. `<target>`
//! is `CARGO_TARGET_DIR` if set, else the nearest folder above the running
//! program that holds cargo's `CACHEDIR.TAG` marker (the workspace's
//! `target/`), else `./target`.

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub(crate) fn out_dir() -> Result<PathBuf> {
    let mut out = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => {
                out = Some(
                    args.next()
                        .map(PathBuf::from)
                        .ok_or_else(|| anyhow::anyhow!("--out needs a directory"))?,
                )
            }
            other => bail!("unknown argument {other}; known: --out <dir>"),
        }
    }
    Ok(out.unwrap_or_else(|| {
        target_dir()
            .unwrap_or_else(|| PathBuf::from("target"))
            .join("site")
    }))
}

/// Cargo's target directory, if this program runs from one.
pub(crate) fn target_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("CARGO_TARGET_DIR") {
        return Some(PathBuf::from(dir));
    }
    std::env::current_exe().ok().and_then(|exe| {
        exe.ancestors()
            .find(|d| d.join("CACHEDIR.TAG").is_file())
            .map(Path::to_path_buf)
    })
}
