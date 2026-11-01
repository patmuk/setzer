//! Images embedded at compile time by `setzer::image!` and resized when the
//! site is written, but only those a page actually used.

use crate::render;
use anyhow::{Context as _, Result};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use std::path::Path;

/// An SVG drawn into a `size`×`size` PNG, centred, transparent around it.
/// Text must be outlined (paths): which fonts a build has differs between
/// machines, so Setzer draws no text and refuses an SVG with `<text>`.
pub(crate) fn svg_to_png(svg: &[u8], size: u32) -> Result<Vec<u8>> {
    use resvg::{tiny_skia, usvg};
    if svg.windows(5).any(|w| w == b"<text") {
        anyhow::bail!("it contains <text>; convert the text to paths (outlines) first");
    }
    let tree = usvg::Tree::from_data(svg, &usvg::Options::default())?;
    let (w, h) = (tree.size().width(), tree.size().height());
    let scale = size as f32 / w.max(h);
    let transform = tiny_skia::Transform::from_scale(scale, scale).post_translate(
        (size as f32 - w * scale) / 2.0,
        (size as f32 - h * scale) / 2.0,
    );
    let mut pixmap = tiny_skia::Pixmap::new(size, size).context("empty PNG size")?;
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    Ok(pixmap.encode_png()?)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_becomes_a_square_png_centred() {
        // 2:1, red: fills the middle band of the square, the corners stay clear.
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 10"><rect width="20" height="10" fill="#f00"/></svg>"##;
        let png = image::load_from_memory(&svg_to_png(svg, 32).unwrap())
            .unwrap()
            .to_rgba8();
        assert_eq!(png.dimensions(), (32, 32));
        assert_eq!(png.get_pixel(16, 16).0, [255, 0, 0, 255]);
        assert_eq!(png.get_pixel(0, 0).0[3], 0);
    }

    #[test]
    fn svg_with_text_is_refused() {
        let svg =
            br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><text>PM</text></svg>"#;
        assert!(
            svg_to_png(svg, 32)
                .unwrap_err()
                .to_string()
                .contains("<text>")
        );
    }
}
