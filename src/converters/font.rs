use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;
pub fn is_font_supported(in_ext: &str, out_ext: &str) -> bool {
    let in_clean = in_ext.trim_start_matches('.').to_lowercase();
    let out_clean = out_ext.trim_start_matches('.').to_lowercase();
    let supported = ["ttf", "otf", "woff", "woff2", "eot", "dfont", "pfa", "pfb"];
    supported.contains(&in_clean.as_str()) && supported.contains(&out_clean.as_str())
}
pub fn convert_font<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    let in_path = input.as_ref();
    let out_path = output.as_ref();
    let out_ext = out_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let py_script = r#"
import sys
from fontTools.ttLib import TTFont
in_file = sys.argv[1]
out_file = sys.argv[2]
out_ext = sys.argv[3].lower()
font = TTFont(in_file)
if out_ext == 'woff2':
    font.flavor = 'woff2'
    font.save(out_file)
elif out_ext == 'woff':
    font.flavor = 'woff'
    font.save(out_file)
elif out_ext in ('ttf', 'otf'):
    font.flavor = None
    font.save(out_file)
elif out_ext == 'eot':
    font.flavor = None
    font.save(out_file)
"#;
    let res = Command::new("python")
        .arg("-c")
        .arg(py_script)
        .arg(in_path)
        .arg(out_path)
        .arg(&out_ext)
        .output();
    if let Ok(output_res) = res {
        if output_res.status.success() && out_path.exists() {
            return Ok(());
        }
    }
    Err(anyhow!("font conversion failed for -> .{}", out_ext))
}
