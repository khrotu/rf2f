use crate::converters::find_binary;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
static MAGICK_BIN: OnceLock<Option<PathBuf>> = OnceLock::new();
pub fn find_magick() -> Option<PathBuf> {
    MAGICK_BIN.get_or_init(|| {
        if let Ok(entries) = std::fs::read_dir(r"C:\Program Files") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("ImageMagick") {
                    let candidate = entry.path().join("magick.exe");
                    if candidate.exists() {
                        return Some(candidate);
                    }
                }
            }
        }
        find_binary("magick", &[
            r"C:\Program Files\ImageMagick\magick.exe",
            r"C:\tools\imagemagick\magick.exe",
        ])
    }).clone()
}
pub fn is_magick_available() -> bool {
    find_magick().is_some()
}
pub fn convert_with_magick<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    convert_with_magick_res(input, output, None)
}
pub fn magick_geometry_for_resolution(res: &str) -> Option<String> {
    let r = res.trim().to_lowercase();
    if r.is_empty() || r == "original" || r == "same" || r == "none" {
        return None;
    }
    if r.contains('x') {
        let parts: Vec<&str> = r.split('x').collect();
        if parts.len() == 2 {
            if let (Ok(w), Ok(h)) = (parts[0].trim().parse::<u32>(), parts[1].trim().parse::<u32>()) {
                if w > 0 && h > 0 {
                    return Some(format!("{}x{}", w, h));
                }
            }
        }
        return None;
    }
    if let Some(num) = r.strip_suffix('p') {
        if let Ok(h) = num.parse::<u32>() {
            if h >= 144 && h <= 4320 {
                return Some(format!("x{}", h));
            }
        }
    }
    None
}
pub fn convert_with_magick_res<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q, resolution: Option<&str>) -> Result<()> {
    let in_path = input.as_ref();
    let out_path = output.as_ref();
    if crate::converters::native_image::convert_image_res(in_path, out_path, resolution).is_ok() {
        return Ok(());
    }
    let magick = find_magick().ok_or_else(|| anyhow!("magick not found"))?;
    let in_ext = in_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let in_stem = in_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let out_dir = out_path.parent().unwrap_or_else(|| Path::new("."));
    let out_ext = out_path.extension().and_then(|s| s.to_str()).unwrap_or("png").to_lowercase();
    if !out_dir.as_os_str().is_empty() {
        std::fs::create_dir_all(out_dir)?;
    }
    let mut cmd = Command::new(magick);
    if ["psd", "psb", "ai", "xcf", "pdf", "gif", "ico", "cur", "heic", "avif"].contains(&in_ext.as_str()) {
        cmd.arg(format!("{}[0]", in_path.to_string_lossy()));
    } else {
        cmd.arg(in_path);
    }
    if let Some(res) = resolution {
        if let Some(geom) = magick_geometry_for_resolution(res) {
            cmd.arg("-thumbnail").arg(geom);
        }
    }
    cmd.arg(out_path);
    let output_res = cmd.output().with_context(|| "failed to execute magick")?;
    if out_path.exists() && out_path.metadata()?.len() > 0 {
        return Ok(());
    }
    let subframe_candidate = out_dir.join(format!("{}-0.{}", in_stem, out_ext));
    if subframe_candidate.exists() && subframe_candidate.metadata()?.len() > 0 {
        let _ = std::fs::remove_file(out_path);
        std::fs::rename(&subframe_candidate, out_path)?;
        return Ok(());
    }
    let py_cmd = r#"
import sys
from PIL import Image
try:
    img = Image.open(sys.argv[1])
    target = sys.argv[3].lower()
    if target in ('jpg', 'jpeg', 'jfif'):
        img.convert('RGB').save(sys.argv[2], 'JPEG')
    elif target == 'dng':
        img.save(sys.argv[2], 'TIFF')
    else:
        img.save(sys.argv[2])
    sys.exit(0)
except Exception:
    sys.exit(1)
"#;
    let res = Command::new("python").arg("-c").arg(py_cmd).arg(in_path).arg(out_path).arg(&out_ext).output();
    if let Ok(o) = res {
        if o.status.success() && out_path.exists() {
            if let Ok(meta) = out_path.metadata() {
                if meta.len() > 0 {
                    return Ok(());
                }
            }
        }
    }
    if !output_res.status.success() {
        let err_str = String::from_utf8_lossy(&output_res.stderr);
        return Err(anyhow!("magick failed: {}", err_str.trim()));
    }
    Err(anyhow!("magick produced no output file"))
}
