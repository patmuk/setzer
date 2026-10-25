//! The command-line options of a site program:
//!
//! * `--out <dir>`: where to write the site. Default: `<target>/site`, next
//!   to cargo's own build output. `<target>` is `CARGO_TARGET_DIR` if set,
//!   else the nearest folder above the running program that holds cargo's
//!   `CACHEDIR.TAG` marker (the workspace's `target/`), else `./target`.
//! * `--serve [port]`: afterwards, serve the site at `http://127.0.0.1:<port>/`
//!   (default 1111) until Ctrl-C.

use anyhow::{anyhow, bail, Result};
use std::path::{Path, PathBuf};

pub(crate) struct Options {
    pub out: PathBuf,
    pub serve: Option<u16>,
}

pub(crate) const DEFAULT_PORT: u16 = 1111;

pub(crate) fn options() -> Result<Options> {
    let mut out = None;
    let mut serve = None;
    let mut args = std::env::args().skip(1).peekable();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => {
                out = Some(
                    args.next()
                        .map(PathBuf::from)
                        .ok_or_else(|| anyhow!("--out needs a directory"))?,
                )
            }
            "--serve" => {
                let port = match args.peek().and_then(|p| p.parse::<u16>().ok()) {
                    Some(port) => {
                        args.next();
                        port
                    }
                    None => DEFAULT_PORT,
                };
                serve = Some(port);
            }
            other => bail!("unknown argument {other}; known: --out <dir>, --serve [port]"),
        }
    }
    let out = out.unwrap_or_else(|| {
        target_dir()
            .unwrap_or_else(|| PathBuf::from("target"))
            .join("site")
    });
    Ok(Options { out, serve })
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
