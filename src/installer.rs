use crate::converters::document::{find_libreoffice, find_pandoc, find_typst};
use crate::converters::ffmpeg::find_ffmpeg;
use crate::converters::magick::find_magick;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};
#[derive(Clone)]
pub struct ToolInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub winget_id: &'static str,
    pub scoop_name: &'static str,
    pub choco_name: &'static str,
    pub finder: fn() -> Option<PathBuf>,
}
pub static TOOLS: &[ToolInfo] = &[
    ToolInfo {
        id: "ffmpeg",
        name: "FFmpeg",
        desc: "Audio/Video/GIF Transcoding",
        winget_id: "Gyan.FFmpeg",
        scoop_name: "ffmpeg",
        choco_name: "ffmpeg",
        finder: find_ffmpeg,
    },
    ToolInfo {
        id: "imagemagick",
        name: "ImageMagick",
        desc: "Advanced Images & RAW Conversion",
        winget_id: "ImageMagick.ImageMagick",
        scoop_name: "imagemagick",
        choco_name: "imagemagick",
        finder: find_magick,
    },
    ToolInfo {
        id: "pandoc",
        name: "Pandoc",
        desc: "Markup & Document Layouts",
        winget_id: "JohnMacFarlane.Pandoc",
        scoop_name: "pandoc",
        choco_name: "pandoc",
        finder: find_pandoc,
    },
    ToolInfo {
        id: "typst",
        name: "Typst",
        desc: "Modern Typesetting & PDF Engine",
        winget_id: "Typst.Typst",
        scoop_name: "typst",
        choco_name: "typst",
        finder: find_typst,
    },
    ToolInfo {
        id: "libreoffice",
        name: "LibreOffice",
        desc: "Office Documents (DOCX/PPTX/XLSX/ODT)",
        winget_id: "TheDocumentFoundation.LibreOffice",
        scoop_name: "libreoffice",
        choco_name: "libreoffice",
        finder: find_libreoffice,
    },
];
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Winget,
    Scoop,
    Choco,
}
pub fn detect_package_manager() -> Option<PackageManager> {
    if which::which("winget").is_ok() {
        return Some(PackageManager::Winget);
    }
    if which::which("scoop").is_ok() {
        return Some(PackageManager::Scoop);
    }
    if which::which("choco").is_ok() {
        return Some(PackageManager::Choco);
    }
    None
}
pub fn install_tool(tool: &ToolInfo, pm: PackageManager) -> Result<()> {
    println!("installing {} via {:?}...", tool.name, pm);
    let mut cmd = match pm {
        PackageManager::Winget => {
            let mut c = Command::new("winget");
            c.args([
                "install",
                "--id",
                tool.winget_id,
                "--exact",
                "--accept-source-agreements",
                "--accept-package-agreements",
            ]);
            c
        }
        PackageManager::Scoop => {
            let mut c = Command::new("powershell");
            c.args(["-NoProfile", "-Command", &format!("scoop install {}", tool.scoop_name)]);
            c
        }
        PackageManager::Choco => {
            let mut c = Command::new("choco");
            c.args(["install", tool.choco_name, "-y"]);
            c
        }
    };
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let status = cmd.status().map_err(|e| anyhow!("execution failed ({:?}): {}", pm, e))?;
    if !status.success() {
        return Err(anyhow!("install failed: {} (exit code {:?})", tool.name, status.code()));
    }
    println!("installed {}", tool.name);
    Ok(())
}
pub fn install_tools(tool_ids: Option<&[String]>, force_all: bool) -> Result<()> {
    let pm = detect_package_manager().ok_or_else(|| {
        anyhow!("no package manager found (winget, scoop, or choco)")
    })?;
    let targets: Vec<&ToolInfo> = if let Some(ids) = tool_ids {
        TOOLS.iter().filter(|t| ids.iter().any(|id| id.eq_ignore_ascii_case(t.id) || id.eq_ignore_ascii_case(t.name))).collect()
    } else if force_all {
        TOOLS.iter().collect()
    } else {
        TOOLS.iter().filter(|t| (t.finder)().is_none()).collect()
    };
    if targets.is_empty() {
        println!("all backends installed");
        return Ok(());
    }
    println!("installer: {:?} -> {}", pm, targets.iter().map(|t| t.name).collect::<Vec<_>>().join(", "));
    let mut success = 0;
    let mut failed = 0;
    for tool in targets {
        match install_tool(tool, pm) {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                eprintln!("error: {}", e);
            }
        }
    }
    println!("installed: {} ok, {} failed", success, failed);
    Ok(())
}
