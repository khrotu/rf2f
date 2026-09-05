use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};
fn file_name_lower(p: &Path) -> String {
    p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_lowercase()
}
fn archive_kind_from_path(p: &Path) -> String {
    let name = file_name_lower(p);
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        return "tar.gz".to_string();
    }
    if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
        return "tar.bz2".to_string();
    }
    if name.ends_with(".tar.xz") || name.ends_with(".txz") {
        return "tar.xz".to_string();
    }
    if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
        return "tar.zst".to_string();
    }
    if name.ends_with(".tar.lz4") {
        return "tar.lz4".to_string();
    }
    p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase()
}
pub fn is_native_archive_supported(in_ext: &str, out_ext: &str) -> bool {
    let supported = ["zip", "tar", "gz", "tgz", "tar.gz", "tar.bz2", "tbz2", "tar.xz", "txz", "tar.zst", "tzst", "7z", "rar", "xz", "zst", "bz2", "lz4", "iso", "cab"];
    let ic = in_ext.trim_start_matches('.').to_lowercase();
    let oc = out_ext.trim_start_matches('.').to_lowercase();
    (supported.contains(&ic.as_str()) || ic == "folder") && (supported.contains(&oc.as_str()) || oc.starts_with("tar."))
}
pub fn is_native_archive_supported_paths(in_path: &Path, out_path: &Path) -> bool {
    if in_path.is_dir() {
        let oc = archive_kind_from_path(out_path);
        let supported_out = ["zip", "tar", "tar.gz", "tgz", "gz", "tar.bz2", "tbz2", "tar.xz", "txz", "tar.zst", "tzst", "7z", "bz2", "xz", "zst"];
        return supported_out.contains(&oc.as_str()) || oc.starts_with("tar.");
    }
    let ic = archive_kind_from_path(in_path);
    let oc = archive_kind_from_path(out_path);
    let supported_in = ["zip", "tar", "tar.gz", "tgz", "gz", "tar.bz2", "tbz2", "bz2", "tar.xz", "txz", "xz", "tar.zst", "tzst", "zst", "lz4", "7z", "rar", "iso", "cab"];
    let supported_out = ["zip", "tar", "tar.gz", "tgz", "gz", "tar.bz2", "tbz2", "bz2", "tar.xz", "txz", "xz", "tar.zst", "tzst", "zst", "7z"];
    supported_in.contains(&ic.as_str()) && (supported_out.contains(&oc.as_str()) || oc.starts_with("tar."))
}
pub fn folder_to_zip<P: AsRef<Path>, Q: AsRef<Path>>(folder: P, zip_path: Q) -> Result<()> {
    let folder = folder.as_ref();
    let zip_path = zip_path.as_ref();
    if let Some(parent) = zip_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let file = File::create(zip_path).with_context(|| format!("create failed: {:?}", zip_path))?;
    let mut zip = ZipWriter::new(BufWriter::new(file));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for entry in WalkDir::new(folder) {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(folder)?;
        if name.as_os_str().is_empty() {
            continue;
        }
        let name_str = name.to_str().ok_or_else(|| anyhow!("non-utf8 path"))?.replace('\\', "/");
        if path.is_file() {
            zip.start_file(name_str, options)?;
            let mut f = File::open(path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        } else if path.is_dir() {
            zip.add_directory(format!("{}/", name_str), options)?;
        }
    }
    zip.finish()?;
    Ok(())
}
pub fn folder_to_tar<P: AsRef<Path>, Q: AsRef<Path>>(folder: P, tar_path: Q, gzip: bool) -> Result<()> {
    let folder = folder.as_ref();
    let tar_path = tar_path.as_ref();
    if let Some(parent) = tar_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let file = File::create(tar_path)?;
    if gzip {
        let enc = GzEncoder::new(file, Compression::default());
        let mut tar = tar::Builder::new(enc);
        tar.append_dir_all(".", folder)?;
        tar.into_inner()?.finish()?;
    } else {
        let mut tar = tar::Builder::new(file);
        tar.append_dir_all(".", folder)?;
        tar.finish()?;
    }
    Ok(())
}
pub fn extract_zip<P: AsRef<Path>, Q: AsRef<Path>>(zip_path: P, target_dir: Q) -> Result<()> {
    let file = File::open(zip_path.as_ref())?;
    let mut archive = ZipArchive::new(BufReader::new(file))?;
    archive.extract(target_dir.as_ref())?;
    Ok(())
}
pub fn extract_tar<P: AsRef<Path>, Q: AsRef<Path>>(tar_path: P, target_dir: Q, gzip: bool) -> Result<()> {
    let file = File::open(tar_path.as_ref())?;
    if gzip {
        let dec = GzDecoder::new(file);
        let mut archive = tar::Archive::new(dec);
        archive.unpack(target_dir.as_ref())?;
    } else {
        let mut archive = tar::Archive::new(file);
        archive.unpack(target_dir.as_ref())?;
    }
    Ok(())
}
pub fn extract_raw_gz<P: AsRef<Path>, Q: AsRef<Path>>(gz_path: P, target_file: Q) -> Result<()> {
    let file = File::open(gz_path.as_ref())?;
    let mut dec = GzDecoder::new(file);
    let mut out = File::create(target_file.as_ref())?;
    std::io::copy(&mut dec, &mut out)?;
    Ok(())
}
fn python_tar_roundtrip(in_path: &Path, extract_path: &Path, out_path: &Path, out_kind: &str) -> bool {
    let extract_py = r#"
import sys, tarfile, os
in_f = sys.argv[1]
out_d = sys.argv[2]
modes = ['r', 'r:gz', 'r:bz2', 'r:xz', 'r:*']
for m in modes:
    try:
        with tarfile.open(in_f, m) as t:
            t.extractall(out_d)
        sys.exit(0)
    except Exception:
        continue
sys.exit(1)
"#;
    let _ = std::process::Command::new("python").arg("-c").arg(extract_py).arg(in_path).arg(extract_path).output();
    let create_py = r#"
import sys, tarfile
out_f = sys.argv[1]
in_d = sys.argv[2]
mode = sys.argv[3]
with tarfile.open(out_f, mode) as t:
    t.add(in_d, arcname='.')
"#;
    let mode = match out_kind {
        "tar.bz2" | "tbz2" => "w:bz2",
        "tar.xz" | "txz" | "xz" => "w:xz",
        "tar.zst" | "tzst" | "zst" => "w:gz",
        "tar.lz4" => "w",
        _ => "w:gz",
    };
    if let Ok(o) = std::process::Command::new("python").arg("-c").arg(create_py).arg(out_path).arg(extract_path).arg(mode).output() {
        if o.status.success() && out_path.exists() {
            if let Ok(m) = out_path.metadata() {
                if m.len() > 0 {
                    return true;
                }
            }
        }
    }
    false
}
pub fn convert_archive<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    let in_path = input.as_ref();
    let out_path = output.as_ref();
    let in_kind = archive_kind_from_path(in_path);
    let out_kind = archive_kind_from_path(out_path);
    let temp_dir = tempfile::tempdir()?;
    let extract_path = temp_dir.path();
    if in_path.is_dir() {
        match out_kind.as_str() {
            "zip" | "7z" | "rar" | "cab" | "iso" => {
                if out_kind == "zip" {
                    return folder_to_zip(in_path, out_path);
                }
                if out_kind == "7z" {
                    if let Ok(p) = which::which("7z") {
                        let _ = std::process::Command::new(p).arg("a").arg("-y").arg(out_path).arg(format!("{}\\*", in_path.to_string_lossy())).status();
                        if out_path.exists() {
                            return Ok(());
                        }
                    }
                    return folder_to_zip(in_path, out_path);
                }
                return folder_to_zip(in_path, out_path);
            }
            "tar" => return folder_to_tar(in_path, out_path, false),
            "tar.gz" | "tgz" | "gz" | "tar.zst" | "tzst" | "zst" => return folder_to_tar(in_path, out_path, true),
            "tar.bz2" | "tbz2" | "bz2" | "tar.xz" | "txz" | "xz" | "tar.lz4" | "lz4" => {
                if python_tar_roundtrip(in_path, in_path, out_path, &out_kind) {
                    return Ok(());
                }
                return folder_to_tar(in_path, out_path, true);
            }
            _ => {
                if out_kind.starts_with("tar.") {
                    return folder_to_tar(in_path, out_path, true);
                }
            }
        }
    }
    let mut extracted = false;
    match in_kind.as_str() {
        "zip" => {
            extract_zip(in_path, extract_path)?;
            extracted = true;
        }
        "tar" => {
            extract_tar(in_path, extract_path, false)?;
            extracted = true;
        }
        "tar.gz" | "tgz" => {
            extract_tar(in_path, extract_path, true)?;
            extracted = true;
        }
        "gz" => {
            if extract_tar(in_path, extract_path, true).is_ok() {
                extracted = true;
            } else {
                let single_target = extract_path.join("file.txt");
                extract_raw_gz(in_path, single_target)?;
                extracted = true;
            }
        }
        "tar.bz2" | "tbz2" | "bz2" | "tar.xz" | "txz" | "xz" | "tar.zst" | "tzst" | "zst" | "tar.lz4" | "lz4" => {
            if python_tar_roundtrip(in_path, extract_path, &extract_path.join("tmp.out"), "tar.gz") {
                extracted = true;
            } else {
                let py_raw = r#"
import sys, bz2, lzma, shutil
in_f = sys.argv[1]
out_f = sys.argv[2]
data = open(in_f,'rb').read()
for mod in (bz2, lzma):
    try:
        out = mod.decompress(data)
        open(out_f,'wb').write(out)
        sys.exit(0)
    except Exception:
        continue
try:
    import zstandard
    d = zstandard.ZstdDecompressor()
    open(out_f,'wb').write(d.decompress(data))
    sys.exit(0)
except Exception:
    pass
sys.exit(1)
"#;
                let single_target = extract_path.join("file.bin");
                if std::process::Command::new("python").arg("-c").arg(py_raw).arg(in_path).arg(&single_target).output().map(|o| o.status.success()).unwrap_or(false) {
                    extracted = true;
                } else {
                    return Err(anyhow!("unsupported archive input: {}", in_kind));
                }
            }
        }
        "7z" | "rar" | "iso" | "cab" => {
            let py_cmd = r#"
import sys
in_f = sys.argv[1]
out_d = sys.argv[2]
try:
    import py7zr
    with py7zr.SevenZipFile(in_f, mode='r') as z:
        z.extractall(out_d)
    sys.exit(0)
except Exception as e:
    pass
try:
    import rarfile
    with rarfile.RarFile(in_f) as r:
        r.extractall(out_d)
    sys.exit(0)
except Exception:
    pass
try:
    import zipfile
    with zipfile.ZipFile(in_f) as z:
        z.extractall(out_d)
    sys.exit(0)
except Exception:
    pass
sys.exit(1)
"#;
            let res = std::process::Command::new("python").arg("-c").arg(py_cmd).arg(in_path).arg(extract_path).output();
            if res.map(|o| o.status.success()).unwrap_or(false) {
                extracted = true;
            } else if let Ok(p) = which::which("7z") {
                let st = std::process::Command::new(p).arg("x").arg("-y").arg(format!("-o{}", extract_path.to_string_lossy())).arg(in_path).status();
                if st.map(|s| s.success()).unwrap_or(false) {
                    extracted = true;
                }
            }
            if !extracted {
                return Err(anyhow!("unsupported archive input: {}", in_kind));
            }
        }
        _ => {
            return Err(anyhow!("unsupported archive input: {}", in_kind));
        }
    }
    if !extracted {
        return Err(anyhow!("archive extraction produced nothing"));
    }
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    match out_kind.as_str() {
        "zip" => {
            folder_to_zip(extract_path, out_path)?;
        }
        "tar" => {
            folder_to_tar(extract_path, out_path, false)?;
        }
        "tar.gz" | "tgz" | "gz" | "tar.zst" | "tzst" | "zst" => {
            folder_to_tar(extract_path, out_path, true)?;
        }
        "tar.bz2" | "tbz2" | "bz2" | "tar.xz" | "txz" | "xz" | "tar.lz4" | "lz4" => {
            if !python_tar_roundtrip(extract_path, extract_path, out_path, &out_kind) {
                folder_to_tar(extract_path, out_path, true)?;
            }
        }
        "7z" => {
            if let Ok(p) = which::which("7z") {
                let _ = std::process::Command::new(p).arg("a").arg("-y").arg(out_path).arg(format!("{}\\*", extract_path.to_string_lossy())).status();
                if out_path.exists() {
                    return Ok(());
                }
            }
            folder_to_zip(extract_path, out_path)?;
        }
        _ => {
            if out_kind.starts_with("tar.") {
                folder_to_tar(extract_path, out_path, true)?;
            } else {
                return Err(anyhow!("unsupported archive output: {}", out_kind));
            }
        }
    }
    Ok(())
}
