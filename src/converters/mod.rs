use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
pub mod document;
pub mod ffmpeg;
pub mod font;
pub mod magick;
pub mod model3d;
pub mod native_archive;
pub mod native_data;
pub mod native_image;
pub mod native_subtitles;
static BIN_CACHE: OnceLock<Mutex<HashMap<String, Option<PathBuf>>>> = OnceLock::new();
pub fn find_binary(exe_name: &str, known_paths: &[&str]) -> Option<PathBuf> {
    let key = exe_name.to_lowercase();
    if let Some(m) = BIN_CACHE.get() {
        if let Ok(c) = m.lock() {
            if let Some(v) = c.get(&key) {
                return v.clone();
            }
        }
    }
    let found = find_binary_uncached(exe_name, known_paths);
    if let Ok(mut c) = BIN_CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock() {
        c.insert(key, found.clone());
    }
    found
}
fn find_binary_uncached(exe_name: &str, known_paths: &[&str]) -> Option<PathBuf> {
    let clean_name = exe_name.trim_end_matches(".exe");
    let with_exe = format!("{}.exe", clean_name);
    if let Ok(p) = which::which(clean_name) {
        return Some(p);
    }
    for p in known_paths {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let local_path = PathBuf::from(&local);
        let link_candidate = local_path.join("Microsoft").join("WinGet").join("Links").join(&with_exe);
        if link_candidate.exists() {
            return Some(link_candidate);
        }
        let apps_candidate = local_path.join("Microsoft").join("WindowsApps").join(&with_exe);
        if apps_candidate.exists() {
            return Some(apps_candidate);
        }
        let prog_candidate = local_path.join("Programs").join(clean_name).join(&with_exe);
        if prog_candidate.exists() {
            return Some(prog_candidate);
        }
        let pandoc_candidate = local_path.join("Pandoc").join(&with_exe);
        if pandoc_candidate.exists() {
            return Some(pandoc_candidate);
        }
        let pkg_dir = local_path.join("Microsoft").join("WinGet").join("Packages");
        if pkg_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&pkg_dir) {
                for entry in entries.flatten() {
                    let subpath = entry.path();
                    let cand1 = subpath.join(&with_exe);
                    if cand1.exists() {
                        return Some(cand1);
                    }
                    if let Ok(subentries) = std::fs::read_dir(&subpath) {
                        for subentry in subentries.flatten() {
                            let cand2 = subentry.path().join(&with_exe);
                            if cand2.exists() {
                                return Some(cand2);
                            }
                        }
                    }
                }
            }
        }
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let scoop_shim = PathBuf::from(userprofile).join("scoop").join("shims").join(&with_exe);
        if scoop_shim.exists() {
            return Some(scoop_shim);
        }
    }
    None
}
