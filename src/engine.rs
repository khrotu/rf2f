use crate::converters::document::convert_document;
use crate::converters::ffmpeg::{convert_with_ffmpeg_res, is_ffmpeg_available};
use crate::converters::font::{convert_font, is_font_supported};
use crate::converters::magick::{convert_with_magick_res, is_magick_available};
use crate::converters::model3d::{convert_model3d, is_model3d_supported};
use crate::converters::native_archive::{convert_archive, is_native_archive_supported_paths};
use crate::converters::native_data::{convert_data, is_native_data_supported};
use crate::converters::native_image::{convert_image_res, is_native_image_supported};
use crate::converters::native_subtitles::{convert_subtitles, is_native_subtitle_supported};
use crate::formats::{detect_format_from_path, FormatCategory};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::time::Instant;
#[derive(Debug, Clone)]
pub struct ConversionJob {
    pub input_path: PathBuf,
    pub target_format: String,
    pub output_path: Option<PathBuf>,
    pub resolution: Option<String>,
}
#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub duration_ms: u128,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub backend: &'static str,
}
pub struct ConversionEngine;
impl ConversionEngine {
    pub fn new() -> Self {
        Self
    }
    pub fn execute(&self, job: &ConversionJob) -> Result<ConversionResult> {
        let in_path = &job.input_path;
        if !in_path.exists() {
            return Err(anyhow!("file not found: {:?}", in_path));
        }
        let in_ext = in_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        let target_ext = job.target_format.trim_start_matches('.').to_lowercase();
        let out_path = if let Some(ref p) = job.output_path {
            if p.is_dir() {
                let file_stem = in_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
                let mut file_name = format!("{}.{}", file_stem, target_ext);
                if let Some(ref res) = job.resolution {
                    if !res.trim().is_empty() {
                        let safe_res = res.trim().to_lowercase().replace(':', "x").replace(' ', "");
                        file_name = format!("{}_{}.{}", file_stem, safe_res, target_ext);
                    }
                }
                p.join(file_name)
            } else if p.extension().is_none() && !target_ext.is_empty() {
                let mut pb = p.clone();
                let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("output");
                pb.set_file_name(format!("{}.{}", fname, target_ext));
                pb
            } else {
                p.clone()
            }
        } else {
            let file_stem = in_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
            let parent = in_path.parent().unwrap_or_else(|| Path::new("."));
            if let Some(ref res) = job.resolution {
                if !res.trim().is_empty() {
                    let safe_res = res.trim().to_lowercase().replace(':', "x").replace(' ', "");
                    parent.join(format!("{}_{}.{}", file_stem, safe_res, target_ext))
                } else {
                    parent.join(format!("{}.{}", file_stem, target_ext))
                }
            } else {
                parent.join(format!("{}.{}", file_stem, target_ext))
            }
        };
        let input_bytes = if in_path.is_file() { std::fs::metadata(in_path)?.len() } else { 0 };
        let start = Instant::now();
        let backend = self.dispatch_conversion(in_path, &out_path, &in_ext, &target_ext, job.resolution.as_deref())?;
        let duration_ms = start.elapsed().as_millis();
        let output_bytes = std::fs::metadata(&out_path)?.len();
        Ok(ConversionResult {
            input_path: in_path.clone(),
            output_path: out_path,
            duration_ms,
            input_bytes,
            output_bytes,
            backend,
        })
    }
    fn dispatch_conversion(&self, in_path: &Path, out_path: &Path, in_ext: &str, out_ext: &str, resolution: Option<&str>) -> Result<&'static str> {
        let in_cat = detect_format_from_path(in_path).map(|f| f.category).unwrap_or(FormatCategory::Unknown);
        let out_cat = crate::formats::find_format(out_ext).map(|f| f.category).unwrap_or(FormatCategory::Unknown);
        let animated_in = matches!(in_ext, "gif" | "apng" | "webp");
        let animated_out = matches!(out_ext, "gif" | "apng" | "webm" | "mp4" | "mov" | "mkv");
        if animated_in && animated_out && is_ffmpeg_available() {
            if convert_with_ffmpeg_res(in_path, out_path, resolution).is_ok() {
                return Ok("ffmpeg");
            }
        }
        if is_native_image_supported(in_ext, out_ext) {
            if convert_image_res(in_path, out_path, resolution).is_ok() {
                return Ok("native-image");
            }
        }
        if is_native_data_supported(in_ext, out_ext) {
            if convert_data(in_path, out_path).is_ok() {
                return Ok("native-data");
            }
        }
        if is_native_archive_supported_paths(in_path, out_path) {
            if convert_archive(in_path, out_path).is_ok() {
                return Ok("native-archive");
            }
        }
        if is_native_subtitle_supported(in_ext, out_ext) {
            if convert_subtitles(in_path, out_path).is_ok() {
                return Ok("native-subtitles");
            }
        }
        if is_font_supported(in_ext, out_ext) {
            if convert_font(in_path, out_path).is_ok() {
                return Ok("font");
            }
        }
        if is_model3d_supported(in_ext, out_ext) {
            if convert_model3d(in_path, out_path).is_ok() {
                return Ok("model3d");
            }
        }
        let category = in_cat;
        match category {
            FormatCategory::Audio | FormatCategory::Video => {
                if is_ffmpeg_available() {
                    convert_with_ffmpeg_res(in_path, out_path, resolution)?;
                    return Ok("ffmpeg");
                }
            }
            FormatCategory::Document => {
                convert_document(in_path, out_path)?;
                return Ok("document");
            }
            FormatCategory::Image => {
                if is_magick_available() {
                    convert_with_magick_res(in_path, out_path, resolution)?;
                    return Ok("imagemagick");
                }
            }
            FormatCategory::Font => {
                convert_font(in_path, out_path)?;
                return Ok("font");
            }
            FormatCategory::Model3D => {
                convert_model3d(in_path, out_path)?;
                return Ok("model3d");
            }
            FormatCategory::Subtitle => {
                convert_subtitles(in_path, out_path)?;
                return Ok("native-subtitles");
            }
            FormatCategory::Data => {
                if convert_data(in_path, out_path).is_ok() {
                    return Ok("native-data");
                }
                if matches!(out_ext, "xlsx" | "xls" | "ods" | "pdf") {
                    convert_document(in_path, out_path)?;
                    return Ok("document");
                }
                return Err(anyhow!("no conversion route for .{} -> .{}", in_ext, out_ext));
            }
            FormatCategory::Archive => {
                convert_archive(in_path, out_path)?;
                return Ok("native-archive");
            }
            FormatCategory::Unknown => {}
        }
        let magick_tried = category == FormatCategory::Image && is_magick_available();
        if !magick_tried && is_magick_available() && matches!(category, FormatCategory::Image | FormatCategory::Document | FormatCategory::Unknown) {
            if convert_with_magick_res(in_path, out_path, resolution).is_ok() {
                return Ok("imagemagick");
            }
        }
        let ffmpeg_tried = matches!(category, FormatCategory::Audio | FormatCategory::Video) && is_ffmpeg_available();
        if !ffmpeg_tried && is_ffmpeg_available() {
            let media_in = matches!(category, FormatCategory::Audio | FormatCategory::Video | FormatCategory::Image | FormatCategory::Unknown);
            let media_out = matches!(out_cat, FormatCategory::Audio | FormatCategory::Video | FormatCategory::Image | FormatCategory::Unknown);
            if media_in && media_out {
                if convert_with_ffmpeg_res(in_path, out_path, resolution).is_ok() {
                    return Ok("ffmpeg");
                }
            }
        }
        if category != FormatCategory::Document && matches!(category, FormatCategory::Document | FormatCategory::Image | FormatCategory::Unknown) {
            if convert_document(in_path, out_path).is_ok() {
                return Ok("document");
            }
        }
        if !is_native_subtitle_supported(in_ext, out_ext) {
            if convert_subtitles(in_path, out_path).is_ok() {
                return Ok("native-subtitles");
            }
        }
        if !is_native_data_supported(in_ext, out_ext) {
            if convert_data(in_path, out_path).is_ok() {
                return Ok("native-data");
            }
        }
        Err(anyhow!("no conversion route for .{} -> .{}", in_ext, out_ext))
    }
}
