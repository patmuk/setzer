//! The one script Setzer ships: light/dark handling. CSS does the default via `prefers-color-scheme`;
//! this one inline script applies a stored user override before first paint,
//! drives the toggle button and flips `<picture>` sources for themed images.
//! `csp_script_hash` pins it, and any page script (`render::script`), in the CSP.

use base64::Engine;
use sha2::{Digest, Sha256};

pub const SCRIPT: &str = r#"(function(){var k='theme',d=document.documentElement,o=['auto','light','dark'];function g(){try{var t=localStorage.getItem(k);return o.indexOf(t)>0?t:'auto'}catch(e){return'auto'}}function a(t){if(t==='auto'){delete d.dataset.theme}else{d.dataset.theme=t}var m=t==='auto'?'(prefers-color-scheme: dark)':t==='dark'?'all':'not all',s=document.querySelectorAll('picture>source[data-dark]');for(var i=0;i<s.length;i++){s[i].media=m}var b=document.getElementById('theme-toggle');if(b){b.textContent=b.dataset[t];b.setAttribute('aria-label',b.dataset.label+': '+b.dataset[t])}}a(g());addEventListener('DOMContentLoaded',function(){a(g());var b=document.getElementById('theme-toggle');if(!b)return;b.hidden=false;b.addEventListener('click',function(){var t=o[(o.indexOf(g())+1)%3];try{if(t==='auto'){localStorage.removeItem(k)}else{localStorage.setItem(k,t)}}catch(e){}a(t)})})})();"#;

/// `'sha256-…'` source expression for a Content-Security-Policy `script-src`.
pub fn csp_script_hash(js: &str) -> String {
    let digest = Sha256::digest(js.as_bytes());
    format!(
        "'sha256-{}'",
        base64::engine::general_purpose::STANDARD.encode(digest)
    )
}

/// Cloudflare Pages `_headers` file: a strict CSP that lets exactly the
/// `scripts` run (none: no script may run), then the site's own `extra` rules
/// as (path, header, value).
pub fn cloudflare_headers<'a>(
    scripts: impl IntoIterator<Item = &'a str>,
    extra: &[(String, String, String)],
) -> String {
    let hashes: Vec<String> = scripts.into_iter().map(csp_script_hash).collect();
    let script_src = if hashes.is_empty() {
        "'none'".to_string()
    } else {
        hashes.join(" ")
    };
    let extra: String = extra
        .iter()
        .map(|(path, name, value)| format!("\n{path}\n  {name}: {value}\n"))
        .collect();
    format!(
        "/*\n  Content-Security-Policy: default-src 'none'; img-src 'self'; style-src 'self'; script-src {script_src}; font-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'; upgrade-insecure-requests\n  X-Content-Type-Options: nosniff\n  X-Frame-Options: DENY\n  Referrer-Policy: strict-origin-when-cross-origin\n  Permissions-Policy: camera=(), microphone=(), geolocation=()\n  Cross-Origin-Opener-Policy: same-origin\n\n/img/*\n  Cache-Control: public, max-age=31536000, immutable\n\n/css/*\n  Cache-Control: public, max-age=86400\n{extra}",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_base64_sha256() {
        let h = csp_script_hash(SCRIPT);
        assert!(h.starts_with("'sha256-") && h.ends_with('\''));
        assert_eq!(h.len(), "'sha256-'".len() + 44);
    }
}
