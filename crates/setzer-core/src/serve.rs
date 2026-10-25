//! `--serve`: a small preview server for the written site, standard
//! library only. It serves files as a static host would: `/blog/` is
//! `blog/index.html`, `/blog` redirects to `/blog/`, and a missing path gets
//! the closest `404.html` up the tree, as on Cloudflare Pages. For local
//! use only: it listens on 127.0.0.1.
//!
//! Live reload: every HTML page it sends gets a small script (the files in
//! the output folder stay untouched). The script listens on
//! `/__setzer/reload`, where the server announces its build id. `cargo watch`
//! restarts the server after a change; the script reconnects, sees a new id
//! and reloads the page.

use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const RELOAD_PATH: &str = "/__setzer/reload";

/// Added before `</body>` of every HTML page the server sends.
const RELOAD_SCRIPT: &str = r#"<script>(() => {
  let build;
  new EventSource("/__setzer/reload").onmessage = (e) => {
    if (build && build !== e.data) location.reload();
    build = e.data;
  };
})();</script>"#;

pub(crate) fn run(root: &Path, port: u16) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .with_context(|| format!("port {port} is in use; try --serve <another port>"))?;
    println!("setzer: serving http://127.0.0.1:{port}/  (Ctrl-C to stop; pages reload on rebuild)");
    let build = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    for stream in listener.incoming().flatten() {
        let root = root.to_path_buf();
        std::thread::spawn(move || {
            let _ = handle(stream, &root, build);
        });
    }
    Ok(())
}

fn handle(mut stream: TcpStream, root: &Path, build: u128) -> std::io::Result<()> {
    let mut request = String::new();
    BufReader::new(&stream).read_line(&mut request)?;
    let mut parts = request.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/"));
    if method != "GET" && method != "HEAD" {
        return respond(
            &mut stream,
            "405 Method Not Allowed",
            "text/plain",
            b"",
            method == "HEAD",
        );
    }
    let path = decode(target.split(['?', '#']).next().unwrap_or("/"));
    if path == RELOAD_PATH {
        return announce(&mut stream, build);
    }
    if !path.starts_with('/') || path.split('/').any(|s| s == "..") {
        return respond(&mut stream, "400 Bad Request", "text/plain", b"", false);
    }
    let file = root.join(path.trim_start_matches('/'));
    if file.is_dir() && !path.ends_with('/') {
        let head = format!("HTTP/1.1 301 Moved Permanently\r\nLocation: {path}/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        return stream.write_all(head.as_bytes());
    }
    let file = if file.is_dir() {
        file.join("index.html")
    } else {
        file
    };
    let head_only = method == "HEAD";
    match std::fs::read(&file) {
        Ok(bytes) => respond(
            &mut stream,
            "200 OK",
            content_type(&file),
            &bytes,
            head_only,
        ),
        Err(_) => match not_found_page(root, &file) {
            Some(page) => respond(
                &mut stream,
                "404 Not Found",
                "text/html; charset=utf-8",
                &std::fs::read(page)?,
                head_only,
            ),
            None => respond(
                &mut stream,
                "404 Not Found",
                "text/plain",
                b"404 Not Found",
                head_only,
            ),
        },
    }
}

/// The `404.html` closest to `file`, looking up to the site root.
fn not_found_page(root: &Path, file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .take_while(|dir| dir.starts_with(root))
        .map(|dir| dir.join("404.html"))
        .find(|page| page.is_file())
}

/// Server-sent events: the build id once, then a keep-alive comment every
/// 15 seconds until the page closes or this server stops.
fn announce(stream: &mut TcpStream, build: u128) -> std::io::Result<()> {
    let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: keep-alive\r\n\r\n";
    stream.write_all(head.as_bytes())?;
    // `retry`: reconnect 300 ms after the server went away, i.e. during a rebuild.
    stream.write_all(format!("retry: 300\ndata: {build}\n\n").as_bytes())?;
    loop {
        std::thread::sleep(Duration::from_secs(15));
        stream.write_all(b": keep-alive\n\n")?;
    }
}

fn respond(
    stream: &mut TcpStream,
    status: &str,
    kind: &str,
    body: &[u8],
    head_only: bool,
) -> std::io::Result<()> {
    let with_script;
    let body = if kind.starts_with("text/html") {
        let html = String::from_utf8_lossy(body);
        with_script = match html.rfind("</body>") {
            Some(i) => format!("{}{RELOAD_SCRIPT}{}", &html[..i], &html[i..]),
            None => format!("{html}{RELOAD_SCRIPT}"),
        };
        with_script.as_bytes()
    } else {
        body
    };
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    if !head_only {
        stream.write_all(body)?;
    }
    Ok(())
}

fn content_type(file: &Path) -> &'static str {
    match file.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "json" => "application/json",
        "txt" => "text/plain; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "pdf" => "application/pdf",
        "woff2" => "font/woff2",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

/// `%C3%BC` → `ü`.
fn decode(path: &str) -> String {
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = |b: u8| (b as char).to_digit(16).map(|d| d as u8);
        match (
            bytes[i],
            bytes.get(i + 1).and_then(|&b| hex(b)),
            bytes.get(i + 2).and_then(|&b| hex(b)),
        ) {
            (b'%', Some(hi), Some(lo)) => {
                out.push(hi * 16 + lo);
                i += 3;
            }
            (b, _, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
