//! Site settings, set once by `Site::new` and read while rendering.

use crate::lang::Lang;
use std::sync::OnceLock;

pub(crate) struct Config {
    pub base_url: String,
    pub name: String,
    pub langs: Vec<Lang>,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub(crate) fn set(config: Config) {
    if CONFIG.set(config).is_err() {
        panic!("setzer::Site::new may only be called once per program");
    }
}

pub(crate) fn get() -> &'static Config {
    CONFIG
        .get()
        .expect("setzer: create the Site (Site::new) before rendering")
}

/// The site's base URL, without trailing slash.
pub fn base_url() -> &'static str {
    &get().base_url
}

/// The site's name, as given to `Site::new`.
pub fn site_name() -> &'static str {
    &get().name
}

/// Absolute URL of a root-relative path.
pub fn url(path: &str) -> String {
    format!("{}{}", base_url(), path)
}

pub(crate) fn default_lang() -> Lang {
    get().langs[0]
}

/// Root-relative path of `path` in `lang`: the default language lives at
/// `/`, the others under `/<code>/`.
pub(crate) fn in_lang(path: &str, lang: Lang) -> String {
    if lang == default_lang() {
        path.to_string()
    } else {
        format!("/{}{}", lang.code(), path)
    }
}
