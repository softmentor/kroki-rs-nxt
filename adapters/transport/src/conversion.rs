//! SVG-to-raster format conversion using resvg + image.
//!
//! Provides post-processing conversion of SVG output to PNG, WebP, and (future) PDF.
//! Providers always generate SVG; the transport layer converts to the target format.

use kroki_core::{DiagramError, DiagramResult, OutputFormat};
use tracing::{debug, info};

/// Convert SVG bytes to the requested output format.
///
/// If the target format is SVG, this is a no-op that returns the input unchanged.
/// For PNG and WebP, uses `resvg` to rasterise the SVG and `image` to encode.
/// PDF is not yet supported and returns an error.
pub fn convert_svg(svg_data: &[u8], target: &OutputFormat) -> DiagramResult<Vec<u8>> {
    match target {
        OutputFormat::Svg => Ok(svg_data.to_vec()),
        OutputFormat::Png => svg_to_png(svg_data),
        OutputFormat::WebP => svg_to_webp(svg_data),
        OutputFormat::Pdf => Err(DiagramError::UnsupportedFormat {
            format: "pdf".to_string(),
            provider: "format-converter".to_string(),
        }),
    }
}

/// Rasterise SVG to PNG using resvg + image.
fn svg_to_png(svg_data: &[u8]) -> DiagramResult<Vec<u8>> {
    let pixmap = rasterise_svg(svg_data)?;
    let (width, height) = (pixmap.width(), pixmap.height());
    let rgba_pixels = pixmap.data();

    debug!(width, height, "encoding rasterised SVG to PNG");

    // Create an image buffer from the RGBA pixel data.
    let img: image::RgbaImage = image::ImageBuffer::from_raw(width, height, rgba_pixels.to_vec())
        .ok_or_else(|| {
            DiagramError::ProcessFailed(
                "failed to create image buffer from pixel data".to_string(),
            )
        })?;

    let mut output = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut output);
    image::ImageEncoder::write_image(
        encoder,
        img.as_raw(),
        width,
        height,
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|err| DiagramError::ProcessFailed(format!("PNG encoding failed: {err}")))?;

    info!(
        output_size = output.len(),
        width, height, "SVG→PNG conversion complete"
    );
    Ok(output)
}

/// Rasterise SVG to WebP using resvg + image.
fn svg_to_webp(svg_data: &[u8]) -> DiagramResult<Vec<u8>> {
    let pixmap = rasterise_svg(svg_data)?;
    let (width, height) = (pixmap.width(), pixmap.height());
    let rgba_pixels = pixmap.data();

    debug!(width, height, "encoding rasterised SVG to WebP");

    let img: image::RgbaImage = image::ImageBuffer::from_raw(width, height, rgba_pixels.to_vec())
        .ok_or_else(|| {
            DiagramError::ProcessFailed(
                "failed to create image buffer from pixel data".to_string(),
            )
        })?;

    let mut output = Vec::new();
    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
    image::ImageEncoder::write_image(
        encoder,
        img.as_raw(),
        width,
        height,
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|err| DiagramError::ProcessFailed(format!("WebP encoding failed: {err}")))?;

    info!(
        output_size = output.len(),
        width, height, "SVG→WebP conversion complete"
    );
    Ok(output)
}

/// Shared SVG rasterisation step using resvg.
///
/// Parses the SVG data, renders it to a `tiny_skia::Pixmap`, and returns
/// the pixel buffer for downstream encoding.
fn rasterise_svg(svg_data: &[u8]) -> DiagramResult<resvg::tiny_skia::Pixmap> {
    let svg_str = std::str::from_utf8(svg_data).map_err(|err| {
        DiagramError::ValidationFailed(format!("SVG data is not valid UTF-8: {err}"))
    })?;

    let opts = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg_str, &opts)
        .map_err(|err| DiagramError::ProcessFailed(format!("SVG parsing failed: {err}")))?;

    let size = tree.size();
    let width = size.width().ceil() as u32;
    let height = size.height().ceil() as u32;

    if width == 0 || height == 0 {
        return Err(DiagramError::ProcessFailed(
            "SVG has zero dimensions, cannot rasterise".to_string(),
        ));
    }

    // Clamp maximum dimensions to prevent memory exhaustion.
    const MAX_DIM: u32 = 8192;
    if width > MAX_DIM || height > MAX_DIM {
        return Err(DiagramError::ProcessFailed(format!(
            "SVG dimensions ({width}x{height}) exceed maximum ({MAX_DIM}x{MAX_DIM})"
        )));
    }

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).ok_or_else(|| {
        DiagramError::ProcessFailed(format!("failed to allocate pixmap ({width}x{height})"))
    })?;

    debug!(width, height, "rasterising SVG");
    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

    Ok(pixmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <rect width="100" height="100" fill="red"/>
    </svg>"#;

    #[test]
    fn convert_svg_to_svg_is_noop() {
        let result = convert_svg(SIMPLE_SVG.as_bytes(), &OutputFormat::Svg).unwrap();
        assert_eq!(result, SIMPLE_SVG.as_bytes());
    }

    #[test]
    fn convert_svg_to_png_produces_valid_png() {
        let result = convert_svg(SIMPLE_SVG.as_bytes(), &OutputFormat::Png).unwrap();
        // PNG files start with the 8-byte PNG signature.
        assert!(result.len() > 8, "PNG output should be non-trivial");
        assert_eq!(&result[0..4], &[0x89, b'P', b'N', b'G']);
    }

    #[test]
    fn convert_svg_to_webp_produces_valid_webp() {
        let result = convert_svg(SIMPLE_SVG.as_bytes(), &OutputFormat::WebP).unwrap();
        // WebP files start with "RIFF" followed by file size, then "WEBP".
        assert!(result.len() > 12, "WebP output should be non-trivial");
        assert_eq!(&result[0..4], b"RIFF");
        assert_eq!(&result[8..12], b"WEBP");
    }

    #[test]
    fn convert_svg_to_pdf_returns_unsupported() {
        let result = convert_svg(SIMPLE_SVG.as_bytes(), &OutputFormat::Pdf);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("pdf"),
            "should mention PDF format: {err_msg}"
        );
    }

    #[test]
    fn convert_invalid_svg_returns_error() {
        let result = convert_svg(b"not an svg", &OutputFormat::Png);
        assert!(result.is_err());
    }

    #[test]
    fn convert_rejects_oversized_svg() {
        // SVG with dimensions exceeding the safety limit.
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10000" height="10000">
            <rect width="10000" height="10000" fill="blue"/>
        </svg>"#;
        let result = convert_svg(svg.as_bytes(), &OutputFormat::Png);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("exceed maximum"),
            "should mention dimension limit: {err_msg}"
        );
    }
}
