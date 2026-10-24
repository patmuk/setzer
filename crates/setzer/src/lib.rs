//! Setzer: static sites as Rust programs. One dependency for a site:
//!
//! * [`Site`]: the builder in `main`.
//! * [`site!`] (in `main.rs`), [`page!`] (in every page's `mod.rs`),
//!   [`image!`], [`static_dir!`]: turn folders and files into typed Rust at
//!   compile time.
//! * [`components`]: ready-made building blocks.
//! * [`design`]: tokens, themes, utilities.
//!
//! `use setzer::prelude::*;` brings what a page or component needs.

pub use setzer_components as components;
pub use setzer_core::*;
pub use setzer_design as design;
pub use setzer_macros::{image, site, static_dir};

/// `Result` of `Site::write` and of `main`.
pub type Result<T = ()> = anyhow::Result<T>;

/// Everything a page or component usually needs.
pub mod prelude {
    pub use setzer_core::prelude::*;
    pub use setzer_design::prelude::*;
}
