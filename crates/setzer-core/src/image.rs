//! Images embedded at compile time by `setzer::image!` and resized when the
//! site is written, but only those a page actually used.

use crate::render;
use anyhow::{Context as _, Result};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use std::path::Path;

pub struct ImageRef {
    bytes: &'static [u8],
    name: &'static str,
    crop: Option<(u32, u32, u32, u32)>,
    widths: &'static [u32],
    width: u32,
    height: u32,
    src: &'static str,
    srcset: &'static str,
}

impl ImageRef {
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub const fn __new(
        bytes: &'static [u8],
        name: &'static str,
        crop: Option<(u32, u32, u32, u32)>,
        widths: &'static [u32],
        width: u32,
        height: u32,
        src: &'static str,
        srcset: &'static str,
    ) -> ImageRef {
        ImageRef {
            bytes,
            name,
            crop,
            widths,
            width,
            height,
            src,
            srcset,
        }
    }

    /// Path of the largest variant. Marks the image as used.
    pub fn src(&'static self) -> &'static str {
        render::record_image(self);
        self.src
    }
    /// `url 320w, url 640w, …`. Marks the image as used.
    pub fn srcset(&'static self) -> &'static str {
        render::record_image(self);
        self.srcset
    }
    /// Size of the largest variant, for `width`/`height` attributes.
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Writes `<dir>/<name>-<w>.jpg` for every width. `cache` keeps variants
    /// between runs so debug builds stay fast.
    pub(crate) fn write(&self, dir: &Path, cache: Option<&Path>) -> Result<()> {
        std::fs::create_dir_all(dir)?;
        let mut decoded = None;
        for &w in self.widths {
            let file = format!("{}-{w}.jpg", self.name);
            let target = dir.join(&file);
            if let Some(cached) = cache.map(|c| c.join(&file)).filter(|c| c.is_file()) {
                std::fs::copy(&cached, &target)?;
                continue;
            }
            if decoded.is_none() {
                let mut img = image::load_from_memory(self.bytes)
                    .with_context(|| format!("decoding image {}", self.name))?;
                if let Some((x, y, cw, ch)) = self.crop {
                    img = img.crop_imm(x, y, cw, ch);
                }
                decoded = Some(img);
            }
            let img = decoded.as_ref().unwrap();
            let resized = if w >= img.width() {
                img.clone()
            } else {
                img.resize(w, u32::MAX, FilterType::Lanczos3)
            };
            let out = std::fs::File::create(&target)?;
            JpegEncoder::new_with_quality(std::io::BufWriter::new(out), 82)
                .encode_image(&resized.to_rgb8())
                .with_context(|| format!("writing {}", target.display()))?;
            if let Some(cache) = cache {
                std::fs::create_dir_all(cache)?;
                std::fs::copy(&target, cache.join(&file))?;
            }
        }
        Ok(())
    }
}
