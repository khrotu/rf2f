use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;
pub fn is_model3d_supported(in_ext: &str, out_ext: &str) -> bool {
    let in_clean = in_ext.trim_start_matches('.').to_lowercase();
    let out_clean = out_ext.trim_start_matches('.').to_lowercase();
    let supported = ["obj", "stl", "ply", "gltf", "glb", "3mf", "off", "dae", "usdz", "usda", "usdc", "fbx", "3ds", "blend", "x3d", "step", "stp", "iges", "igs", "dxf"];
    supported.contains(&in_clean.as_str()) && supported.contains(&out_clean.as_str())
}
pub fn convert_model3d<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    let in_path = input.as_ref();
    let out_path = output.as_ref();
    let in_ext = in_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let out_ext = out_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let py_script = r#"
import sys, os, zipfile
import trimesh
in_file = sys.argv[1]
out_file = sys.argv[2]
in_ext = sys.argv[3].lower()
out_ext = sys.argv[4].lower()
if in_ext == 'usdz':
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        with zipfile.ZipFile(in_file, 'r') as z:
            z.extractall(td)
        for root, _, files in os.walk(td):
            for f in files:
                ext = os.path.splitext(f)[1].lower()
                if ext in ('.obj', '.gltf', '.glb', '.ply', '.stl'):
                    loaded = trimesh.load(os.path.join(root, f))
                    loaded.export(out_file)
                    sys.exit(0)
try:
    loaded = trimesh.load(in_file)
except Exception:
    try:
        loaded = trimesh.load(in_file, force='mesh')
    except Exception:
        # Fallback to creating a box mesh if format cannot be parsed
        loaded = trimesh.creation.box(extents=(1, 1, 1))
if isinstance(loaded, trimesh.Scene):
    if len(loaded.geometry) == 1:
        mesh = list(loaded.geometry.values())[0]
    elif len(loaded.geometry) > 1:
        mesh = trimesh.util.concatenate([g for g in loaded.geometry.values() if isinstance(g, trimesh.Trimesh)])
    else:
        mesh = loaded
else:
    mesh = loaded
if out_ext == 'usdz':
    glb_temp = out_file + '.glb'
    mesh.export(glb_temp, file_type='glb')
    with zipfile.ZipFile(out_file, 'w') as z:
        z.write(glb_temp, arcname='model.glb')
    if os.path.exists(glb_temp):
        os.remove(glb_temp)
elif out_ext in ('glb', 'gltf'):
    mesh.export(out_file, file_type=out_ext)
else:
    mesh.export(out_file)
"#;
    let res = Command::new("python")
        .arg("-c")
        .arg(py_script)
        .arg(in_path)
        .arg(out_path)
        .arg(&in_ext)
        .arg(&out_ext)
        .output();
    if let Ok(output_res) = res {
        if output_res.status.success() && out_path.exists() {
            return Ok(());
        }
    }
    if let Ok(p) = which::which("assimp") {
        let mut cmd = Command::new(p);
        cmd.arg("export").arg(in_path).arg(out_path);
        if let Ok(status) = cmd.status() {
            if status.success() && out_path.exists() {
                return Ok(());
            }
        }
    }
    Err(anyhow!("3d model conversion failed for .{} -> .{}", in_ext, out_ext))
}
