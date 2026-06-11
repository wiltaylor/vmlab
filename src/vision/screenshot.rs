//! Screenshot file I/O.
//!
//! QEMU's QMP `screendump` command writes binary PPM (P6); reference images
//! are conventionally PNG. [`load_screen`] sniffs the format from file
//! content rather than trusting the extension, so either works anywhere an
//! image path is accepted.

use std::path::Path;

use anyhow::{Context, Result};
use image::{ImageFormat, RgbImage};

/// Load a screen image (PPM from QMP `screendump`, PNG, or BMP) as RGB.
///
/// The format is detected from the file's magic bytes, not its extension.
pub fn load_screen(path: &Path) -> Result<RgbImage> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("failed to read image file {}", path.display()))?;
    let format = image::guess_format(&bytes)
        .with_context(|| format!("unrecognised image format in {}", path.display()))?;
    let img = image::load_from_memory_with_format(&bytes, format)
        .with_context(|| format!("failed to decode {} as {format:?}", path.display()))?;
    Ok(img.to_rgb8())
}

/// Save an RGB image as PNG, regardless of the path's extension.
pub fn save_png(img: &RgbImage, path: &Path) -> Result<()> {
    img.save_with_format(path, ImageFormat::Png)
        .with_context(|| format!("failed to write PNG {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    /// Hand-rolled 2x2 P6 PPM decodes to the exact pixels.
    #[test]
    fn ppm_p6_decodes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("screen.ppm");
        let mut bytes = b"P6\n2 2\n255\n".to_vec();
        bytes.extend_from_slice(&[
            255, 0, 0, // (0,0) red
            0, 255, 0, // (1,0) green
            0, 0, 255, // (0,1) blue
            17, 34, 51, // (1,1) arbitrary
        ]);
        std::fs::write(&path, bytes).unwrap();

        let img = load_screen(&path).unwrap();
        assert_eq!(img.dimensions(), (2, 2));
        assert_eq!(img.get_pixel(0, 0), &Rgb([255, 0, 0]));
        assert_eq!(img.get_pixel(1, 0), &Rgb([0, 255, 0]));
        assert_eq!(img.get_pixel(0, 1), &Rgb([0, 0, 255]));
        assert_eq!(img.get_pixel(1, 1), &Rgb([17, 34, 51]));
    }

    /// save_png / load_screen round-trips pixels, sniffing PNG by content
    /// despite a misleading extension.
    #[test]
    fn png_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        // Deliberately wrong extension: content sniffing must win.
        let path = dir.path().join("shot.ppm");
        let img = RgbImage::from_fn(7, 5, |x, y| {
            Rgb([(x * 30) as u8, (y * 40) as u8, (x + y) as u8])
        });
        save_png(&img, &path).unwrap();

        let back = load_screen(&path).unwrap();
        assert_eq!(back, img);
    }

    #[test]
    fn missing_file_is_error() {
        let err = load_screen(Path::new("/nonexistent/vmlab-vision.png")).unwrap_err();
        assert!(err.to_string().contains("vmlab-vision.png"));
    }

    #[test]
    fn garbage_content_is_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("junk.png");
        std::fs::write(&path, b"not an image at all").unwrap();
        assert!(load_screen(&path).is_err());
    }
}
