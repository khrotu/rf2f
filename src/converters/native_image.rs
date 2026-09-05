use anyhow::{anyhow, Context, Result};
use image::{DynamicImage, ImageFormat, ImageReader};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
pub fn is_native_image_supported(in_ext: &str, out_ext: &str) -> bool {
    let in_clean = in_ext.trim_start_matches('.').to_lowercase();
    let out_clean = out_ext.trim_start_matches('.').to_lowercase();
    let core_native = ["png", "jpg", "jpeg", "jfif", "webp", "bmp", "ico", "tiff", "tif", "tga", "qoi", "pnm", "ppm", "pgm", "pbm", "gif"];
    core_native.contains(&in_clean.as_str()) && core_native.contains(&out_clean.as_str())
}
pub fn ext_to_image_format(ext: &str) -> Option<ImageFormat> {
    match ext.trim_start_matches('.').to_lowercase().as_str() {
        "png" => Some(ImageFormat::Png),
        "jpg" | "jpeg" | "jfif" => Some(ImageFormat::Jpeg),
        "webp" => Some(ImageFormat::WebP),
        "bmp" => Some(ImageFormat::Bmp),
        "ico" => Some(ImageFormat::Ico),
        "tiff" | "tif" => Some(ImageFormat::Tiff),
        "tga" => Some(ImageFormat::Tga),
        "qoi" => Some(ImageFormat::Qoi),
        "pnm" | "ppm" | "pgm" | "pbm" => Some(ImageFormat::Pnm),
        "gif" => Some(ImageFormat::Gif),
        "hdr" => Some(ImageFormat::Hdr),
        "exr" => Some(ImageFormat::OpenExr),
        "dds" => Some(ImageFormat::Dds),
        "avif" => Some(ImageFormat::Avif),
        _ => None,
    }
}
pub fn convert_image<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    convert_image_res(input, output, None)
}
pub fn target_size_for_resolution(w: u32, h: u32, res: &str) -> Option<(u32, u32)> {
    let r = res.trim().to_lowercase();
    if r.is_empty() || r == "original" || r == "same" || r == "none" {
        return None;
    }
    if r.contains('x') {
        let parts: Vec<&str> = r.split('x').collect();
        if parts.len() == 2 {
            if let (Ok(tw), Ok(th)) = (parts[0].trim().parse::<u32>(), parts[1].trim().parse::<u32>()) {
                if tw > 0 && th > 0 {
                    return Some((tw.max(1), th.max(1)));
                }
            }
        }
        return None;
    }
    if let Some(num) = r.strip_suffix('p') {
        if let Ok(th) = num.parse::<u32>() {
            if th >= 144 && th <= 4320 && w > 0 && h > 0 {
                let tw = ((w as u64 * th as u64) / h as u64).max(1).min(7680) as u32;
                return Some((tw, th));
            }
        }
    }
    None
}
pub fn convert_image_res<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q, resolution: Option<&str>) -> Result<()> {
    let in_path = input.as_ref();
    let out_path = output.as_ref();
    let out_ext = out_path.extension().and_then(|s| s.to_str()).ok_or_else(|| anyhow!("missing output extension"))?.to_lowercase();
    let out_fmt = ext_to_image_format(&out_ext).ok_or_else(|| anyhow!("unsupported image format: {}", out_ext))?;
    let mut img: DynamicImage = ImageReader::open(in_path)
        .with_context(|| format!("open failed: {:?}", in_path))?
        .with_guessed_format()
        .with_context(|| format!("format detection failed: {:?}", in_path))?
        .decode()
        .with_context(|| format!("decode failed: {:?}", in_path))?;
    if let Some(res) = resolution {
        if let Some((tw, th)) = target_size_for_resolution(img.width(), img.height(), res) {
            if tw != img.width() || th != img.height() {
                img = img.resize_exact(tw, th, image::imageops::FilterType::Triangle);
            }
        }
    }
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    if out_fmt == ImageFormat::Ico {
        let resized = if img.width() > 256 || img.height() > 256 {
            img.resize_to_fill(256, 256, image::imageops::FilterType::Triangle)
        } else {
            img
        };
        let out_file = File::create(out_path).with_context(|| format!("create failed: {:?}", out_path))?;
        let mut writer = BufWriter::new(out_file);
        resized.to_rgba8().write_to(&mut writer, ImageFormat::Ico)?;
        return Ok(());
    }
    let out_file = File::create(out_path).with_context(|| format!("create failed: {:?}", out_path))?;
    let mut writer = BufWriter::new(out_file);
    if matches!(out_fmt, ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Bmp | ImageFormat::WebP | ImageFormat::Gif) {
        use image::ColorType;
        match img.color() {
            ColorType::Rgb16 | ColorType::Rgba16 | ColorType::Rgb32F | ColorType::Rgba32F => {
                img.to_rgba8().write_to(&mut writer, out_fmt).with_context(|| format!("write failed: {:?}", out_path))?;
                return Ok(());
            }
            _ => {}
        }
    }
    img.write_to(&mut writer, out_fmt)
        .with_context(|| format!("write failed: {:?}", out_path))?;
    Ok(())
}
