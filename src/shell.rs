use crate::formats::{find_format, get_suggested_conversions, is_video_resolution_target, scale_target_for, FormatCategory, VIDEO_RESOLUTIONS, FORMAT_DATABASE};
use anyhow::{Context, Result};
use std::env;
use winreg::enums::*;
use winreg::RegKey;
pub fn add_exe_to_path() -> Result<()> {
    let exe_path = env::current_exe().context("failed to get current exe path")?;
    let exe_dir = exe_path.parent().unwrap_or(exe_path.as_path());
    let dir_str = exe_dir.to_str().context("non-utf8 path")?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (env_key, _) = hkcu.create_subkey("Environment")?;
    let current_path: String = env_key.get_value("Path").unwrap_or_default();
    let mut entries: Vec<String> = current_path.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    if !entries.iter().any(|e| e.eq_ignore_ascii_case(dir_str)) {
        entries.push(dir_str.to_string());
        let new_path = entries.join(";");
        env_key.set_value("Path", &new_path)?;
        notify_environment_change();
    }
    Ok(())
}
pub fn remove_exe_from_path() -> Result<()> {
    let exe_path = env::current_exe().context("failed to get current exe path")?;
    let exe_dir = exe_path.parent().unwrap_or(exe_path.as_path());
    let dir_str = exe_dir.to_str().context("non-utf8 path")?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (env_key, _) = hkcu.create_subkey("Environment")?;
    let current_path: String = env_key.get_value("Path").unwrap_or_default();
    let mut entries: Vec<String> = current_path.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    if entries.iter().any(|e| e.eq_ignore_ascii_case(dir_str)) {
        entries.retain(|e| !e.eq_ignore_ascii_case(dir_str));
        let new_path = entries.join(";");
        env_key.set_value("Path", &new_path)?;
        notify_environment_change();
    }
    Ok(())
}
fn notify_environment_change() {
    unsafe {
        let env_wide: Vec<u16> = "Environment\0".encode_utf16().collect();
        let mut result: usize = 0;
        windows_sys::Win32::UI::WindowsAndMessaging::SendMessageTimeoutW(
            windows_sys::Win32::UI::WindowsAndMessaging::HWND_BROADCAST as _,
            windows_sys::Win32::UI::WindowsAndMessaging::WM_SETTINGCHANGE,
            0,
            env_wide.as_ptr() as _,
            windows_sys::Win32::UI::WindowsAndMessaging::SMTO_ABORTIFHUNG,
            1000,
            &mut result,
        );
    }
}
fn sanitize_ext(ext: &str) -> String {
    ext.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>()
}
fn context_menu_id(ext: &str) -> String {
    format!("rf2f_{}", sanitize_ext(ext))
}
fn resolve_icon_path(exe_path: &std::path::Path, exe_str: &str) -> String {
    let mut icon_path = format!("{},0", exe_str);
    if let Some(parent) = exe_path.parent() {
        let direct_ico = parent.join("logo.ico");
        let parent_assets_ico = parent.join("assets").join("logo.ico");
        let root_assets_ico = parent.parent().and_then(|p| p.parent()).map(|p| p.join("assets").join("logo.ico"));
        if direct_ico.exists() {
            icon_path = direct_ico.to_string_lossy().to_string();
        } else if parent_assets_ico.exists() {
            icon_path = parent_assets_ico.to_string_lossy().to_string();
        } else if let Some(r) = root_assets_ico {
            if r.exists() {
                icon_path = r.to_string_lossy().to_string();
            }
        }
    }
    icon_path
}
pub fn register_context_menu() -> Result<()> {
    add_exe_to_path()?;
    let exe_path = env::current_exe().context("failed to get current exe path")?;
    let exe_str = exe_path.to_str().context("non-utf8 path")?;
    let icon_path = resolve_icon_path(&exe_path, exe_str);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let _ = hkcu.delete_subkey_all(r"Software\Classes\*\shell\rf2f");
    let _ = hkcu.delete_subkey_all(r"Software\Classes\Directory\ContextMenus\rf2f");
    for info in FORMAT_DATABASE {
        let ext_lower = info.ext.to_lowercase();
        let menu_id = context_menu_id(&ext_lower);
        let scale_menu_id = format!("{}_scale", menu_id);
        let _ = hkcu.delete_subkey_all(&format!(r"Software\Classes\Directory\ContextMenus\{}", menu_id));
        let _ = hkcu.delete_subkey_all(&format!(r"Software\Classes\Directory\ContextMenus\{}", scale_menu_id));
        for (res_label, _dims) in VIDEO_RESOLUTIONS {
            let sub_id = format!("{}_{}", menu_id, sanitize_ext(res_label));
            let _ = hkcu.delete_subkey_all(&format!(r"Software\Classes\Directory\ContextMenus\{}", sub_id));
        }
        let ctx_shell_path = format!(r"Software\Classes\Directory\ContextMenus\{}\shell", menu_id);
        let (ctx_shell_key, _) = hkcu.create_subkey(&ctx_shell_path)?;
        let src_cat = find_format(&ext_lower).map(|f| f.category);
        let scale_all = matches!(src_cat, Some(FormatCategory::Image));
        let scale_filtered = matches!(src_cat, Some(FormatCategory::Video));
        let mut order: u32 = 1;
        let mut scale_targets: Vec<String> = Vec::new();
        for target in get_suggested_conversions(&ext_lower) {
            let target_clean = target.trim_start_matches('.').to_lowercase();
            if target_clean.is_empty() || target_clean == ext_lower {
                continue;
            }
            let label = format!("Convert to {}", target_clean.to_uppercase());
            let id = format!("{:02}_to_{}", order, sanitize_ext(&target_clean));
            order += 1;
            let (preset_key, _) = ctx_shell_key.create_subkey(&id)?;
            preset_key.set_value("", &label)?;
            preset_key.set_value("MUIVerb", &label)?;
            preset_key.set_value("Icon", &icon_path)?;
            let (cmd_key, _) = preset_key.create_subkey("command")?;
            cmd_key.set_value("", &format!("\"{}\" convert \"%1\" --format {}", exe_str, target_clean))?;
            let scale_ok = scale_all || (scale_filtered && is_video_resolution_target(&target_clean));
            if scale_ok && !scale_targets.contains(&target_clean) {
                scale_targets.push(target_clean);
            }
        }
        if (scale_all || scale_filtered) && !scale_targets.is_empty() {
            let scale_shell_path = format!(r"Software\Classes\Directory\ContextMenus\{}\shell", scale_menu_id);
            let (scale_shell_key, _) = hkcu.create_subkey(&scale_shell_path)?;
            let scale_format = scale_target_for(&ext_lower);
            for (i, (res_label, _dims)) in VIDEO_RESOLUTIONS.iter().enumerate() {
                let scale_label = format!("Scale {}", res_label);
                let scale_id = format!("{:02}_scale_{}", i as u32 + 1, sanitize_ext(res_label));
                let (scale_key, _) = scale_shell_key.create_subkey(&scale_id)?;
                scale_key.set_value("", &scale_label)?;
                scale_key.set_value("MUIVerb", &scale_label)?;
                scale_key.set_value("Icon", &icon_path)?;
                let (scale_cmd, _) = scale_key.create_subkey("command")?;
                scale_cmd.set_value("", &format!("\"{}\" convert \"%1\" --format {} --resolution {}", exe_str, scale_format, res_label))?;
            }
            let scale_assoc_path = format!(r"Software\Classes\SystemFileAssociations\.{}\shell\rf2f_scale", ext_lower);
            let (scale_assoc_key, _) = hkcu.create_subkey(&scale_assoc_path)?;
            scale_assoc_key.set_value("", &"Scale with rf2f")?;
            scale_assoc_key.set_value("MUIVerb", &"Scale with rf2f")?;
            scale_assoc_key.set_value("Icon", &icon_path)?;
            scale_assoc_key.set_value("ExtendedSubCommandsKey", &format!(r"Directory\ContextMenus\{}", scale_menu_id))?;
            let _ = scale_assoc_key.delete_value("SubCommands");
            let _ = hkcu.delete_subkey_all(&format!(r"Software\Classes\SystemFileAssociations\.{}\shell\rf2f_scale\shell", ext_lower));
        }
        let assoc_path = format!(r"Software\Classes\SystemFileAssociations\.{}\shell\rf2f", ext_lower);
        let (assoc_key, _) = hkcu.create_subkey(&assoc_path)?;
        assoc_key.set_value("", &"Convert with rf2f")?;
        assoc_key.set_value("MUIVerb", &"Convert with rf2f")?;
        assoc_key.set_value("Icon", &icon_path)?;
        assoc_key.set_value("ExtendedSubCommandsKey", &format!(r"Directory\ContextMenus\{}", menu_id))?;
        let _ = assoc_key.delete_value("SubCommands");
        let _ = hkcu.delete_subkey_all(&format!(r"Software\Classes\SystemFileAssociations\.{}\shell\rf2f\shell", ext_lower));
    }
    let dir_classes_path = r"Software\Classes\Directory\shell\rf2f";
    let (dir_key, _) = hkcu.create_subkey(dir_classes_path)?;
    dir_key.set_value("", &"Compress folder with rf2f (ZIP)")?;
    dir_key.set_value("MUIVerb", &"Compress folder with rf2f (ZIP)")?;
    dir_key.set_value("Icon", &icon_path)?;
    let (dir_cmd_key, _) = dir_key.create_subkey("command")?;
    dir_cmd_key.set_value("", &format!("\"{}\" convert \"%1\" --format zip", exe_str))?;
    unsafe {
        windows_sys::Win32::UI::Shell::SHChangeNotify(
            windows_sys::Win32::UI::Shell::SHCNE_ASSOCCHANGED as i32,
            windows_sys::Win32::UI::Shell::SHCNF_IDLIST,
            std::ptr::null(),
            std::ptr::null(),
        );
    }
    Ok(())
}
pub fn unregister_context_menu() -> Result<()> {
    remove_exe_from_path()?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let _ = hkcu.delete_subkey_all(r"Software\Classes\*\shell\rf2f");
    let _ = hkcu.delete_subkey_all(r"Software\Classes\Directory\ContextMenus\rf2f");
    let _ = hkcu.delete_subkey_all(r"Software\Classes\Directory\shell\rf2f");
    for info in FORMAT_DATABASE {
        let ext_lower = info.ext.to_lowercase();
        let menu_id = context_menu_id(&ext_lower);
        let _ = hkcu.delete_subkey_all(&format!(r"Software\Classes\SystemFileAssociations\.{}\shell\rf2f", ext_lower));
        let _ = hkcu.delete_subkey_all(&format!(r"Software\Classes\SystemFileAssociations\.{}\shell\rf2f_scale", ext_lower));
        let _ = hkcu.delete_subkey_all(&format!(r"Software\Classes\Directory\ContextMenus\{}", menu_id));
        let _ = hkcu.delete_subkey_all(&format!(r"Software\Classes\Directory\ContextMenus\{}_scale", menu_id));
        for (res_label, _dims) in VIDEO_RESOLUTIONS {
            let sub_id = format!("{}_{}", menu_id, sanitize_ext(res_label));
            let _ = hkcu.delete_subkey_all(&format!(r"Software\Classes\Directory\ContextMenus\{}", sub_id));
        }
    }
    unsafe {
        windows_sys::Win32::UI::Shell::SHChangeNotify(
            windows_sys::Win32::UI::Shell::SHCNE_ASSOCCHANGED as i32,
            windows_sys::Win32::UI::Shell::SHCNF_IDLIST,
            std::ptr::null(),
            std::ptr::null(),
        );
    }
    Ok(())
}
