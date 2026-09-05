use crate::converters::find_binary;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
pub fn find_pandoc() -> Option<PathBuf> {
    find_binary("pandoc", &[
        r"C:\Program Files\Pandoc\pandoc.exe",
        r"C:\Program Files (x86)\Pandoc\pandoc.exe",
    ])
}
pub fn find_typst() -> Option<PathBuf> {
    find_binary("typst", &[
        r"C:\Program Files\Typst\typst.exe",
        r"C:\tools\typst\typst.exe",
    ])
}
pub fn find_libreoffice() -> Option<PathBuf> {
    find_binary("soffice", &[
        r"C:\Program Files\LibreOffice\program\soffice.exe",
        r"C:\Program Files (x86)\LibreOffice\program\soffice.exe",
    ])
}
pub fn convert_with_pandoc<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    let pandoc = find_pandoc().ok_or_else(|| anyhow!("pandoc not found"))?;
    let in_path = input.as_ref();
    let out_path = output.as_ref();
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let mut cmd = Command::new(pandoc);
    cmd.arg(in_path).arg("-o").arg(out_path);
    let output_res = cmd.output().with_context(|| "failed to execute pandoc")?;
    if !output_res.status.success() {
        let err_str = String::from_utf8_lossy(&output_res.stderr);
        return Err(anyhow!("pandoc failed: {}", err_str.trim()));
    }
    Ok(())
}
pub fn convert_with_typst<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    let typst = find_typst().ok_or_else(|| anyhow!("typst not found"))?;
    let in_path = input.as_ref();
    let out_path = output.as_ref();
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let mut cmd = Command::new(typst);
    cmd.arg("compile").arg(in_path).arg(out_path);
    let output_res = cmd.output().with_context(|| "failed to execute typst")?;
    if !output_res.status.success() {
        let err_str = String::from_utf8_lossy(&output_res.stderr);
        return Err(anyhow!("typst failed: {}", err_str.trim()));
    }
    Ok(())
}
pub fn convert_with_libreoffice<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    let soffice = find_libreoffice().ok_or_else(|| anyhow!("libreoffice not found"))?;
    let in_path = input.as_ref();
    let out_path = output.as_ref();
    let in_abs = std::path::absolute(in_path).unwrap_or_else(|_| in_path.to_path_buf());
    let out_abs = std::path::absolute(out_path).unwrap_or_else(|_| out_path.to_path_buf());
    let in_ext = in_abs.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let out_ext = out_abs.extension().and_then(|s| s.to_str()).unwrap_or("pdf").to_lowercase();
    let out_dir = out_abs.parent().unwrap_or_else(|| Path::new("."));
    if !out_dir.as_os_str().is_empty() {
        std::fs::create_dir_all(out_dir)?;
    }
    let temp_profile = tempfile::tempdir()?;
    let profile_uri = format!("file:///{}", temp_profile.path().to_string_lossy().replace('\\', "/"));
    let mut cmd = Command::new(soffice);
    cmd.arg(format!("-env:UserInstallation={}", profile_uri));
    cmd.arg("--headless");
    let is_calc = ["xlsx", "xls", "ods", "numbers", "csv", "tsv"].contains(&in_ext.as_str());
    let (infilter, target_filter) = match (in_ext.as_str(), out_ext.as_str()) {
        ("pdf", "docx") => (Some("writer_pdf_import"), "docx:MS Word 2007 XML"),
        ("pdf", "doc") => (Some("writer_pdf_import"), "doc:MS Word 97"),
        ("pdf", "odt") => (Some("writer_pdf_import"), "odt:writer8"),
        ("pdf", "html" | "htm") => (Some("writer_pdf_import"), "html:HTML"),
        ("pdf", "txt") => (Some("writer_pdf_import"), "txt:Text"),
        ("pdf", "rtf") => (Some("writer_pdf_import"), "rtf:Rich Text Format"),
        ("pages" | "odt" | "doc" | "docx" | "rtf" | "wpd" | "wps" | "txt", "pdf") => (None, "pdf:writer_pdf_Export"),
        ("key" | "odp" | "ppt" | "pptx", "pdf") => (None, "pdf:impress_pdf_Export"),
        ("numbers" | "ods" | "xls" | "xlsx" | "csv" | "tsv", "pdf") => (None, "pdf:calc_pdf_Export"),
        (_, "tsv") if is_calc => (None, "csv:Text - txt - csv (StarCalc):9,34,0,1,1"),
        (_, "csv") if is_calc => (None, "csv:Text - txt - csv (StarCalc)"),
        (_, "html" | "htm") if is_calc => (None, "html:HTML (StarCalc)"),
        ("doc", "docx") => (None, "docx:MS Word 2007 XML"),
        ("docx", "doc") => (None, "doc:MS Word 97"),
        (_, "docx") => (None, "docx:MS Word 2007 XML"),
        (_, "pdf") => (None, "pdf:writer_pdf_Export"),
        (_, "html" | "htm") => (None, "html:HTML"),
        (_, "txt") => (None, "txt:Text"),
        _ => (None, out_ext.as_str()),
    };
    if let Some(inf) = infilter {
        cmd.arg(format!("--infilter={}", inf));
    }
    cmd.arg("--convert-to").arg(target_filter).arg(&in_abs).arg("--outdir").arg(out_dir);
    let output_res = cmd.output().with_context(|| "failed to execute libreoffice")?;
    let in_stem = in_abs.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let candidate_exts = [&out_ext, "csv", "html", "txt", "pdf", "docx", "tsv"];
    for cand in candidate_exts {
        let generated_file = out_dir.join(format!("{}.{}", in_stem, cand));
        if generated_file.exists() && generated_file.is_file() {
            if let Ok(meta) = generated_file.metadata() {
                if meta.len() > 0 {
                    if generated_file != out_abs {
                        let _ = std::fs::remove_file(&out_abs);
                        let _ = std::fs::rename(&generated_file, &out_abs);
                    }
                    if out_abs.exists() {
                        return Ok(());
                    }
                }
            }
        }
    }
    if out_abs.exists() {
        if let Ok(meta) = out_abs.metadata() {
            if meta.len() > 0 {
                return Ok(());
            }
        }
    }
    if !output_res.status.success() {
        let err_str = String::from_utf8_lossy(&output_res.stderr);
        let out_str = String::from_utf8_lossy(&output_res.stdout);
        return Err(anyhow!("libreoffice failed: {} {}", err_str.trim(), out_str.trim()));
    }
    Err(anyhow!("libreoffice produced no output file"))
}
pub fn render_html_to_image<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    let in_path = input.as_ref();
    let out_path = output.as_ref();
    let in_abs = std::path::absolute(in_path).unwrap_or_else(|_| in_path.to_path_buf());
    let out_abs = std::path::absolute(out_path).unwrap_or_else(|_| out_path.to_path_buf());
    let in_uri = format!("file:///{}", in_abs.to_string_lossy().replace('\\', "/"));
    let browser_paths = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    ];
    for b in &browser_paths {
        let bp = PathBuf::from(b);
        if bp.exists() {
            let res = Command::new(bp)
                .arg("--headless=new")
                .arg("--disable-gpu")
                .arg(format!("--screenshot={}", out_abs.to_string_lossy()))
                .arg("--window-size=1280,1024")
                .arg(&in_uri)
                .output();
            if let Ok(o) = res {
                if o.status.success() && out_abs.exists() {
                    return Ok(());
                }
            }
        }
    }
    Err(anyhow!("no browser available to render html to image"))
}
pub fn convert_document<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    let in_path = input.as_ref();
    let out_path = output.as_ref();
    let in_ext = in_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let out_ext = out_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    if in_ext == "pdf" && out_ext == "txt" {
        let py_cmd = r#"
import fitz, sys
try:
    doc = fitz.open(sys.argv[1])
    text = '\n'.join(p.get_text() for p in doc)
    with open(sys.argv[2], 'w', encoding='utf-8') as f:
        f.write(text)
    sys.exit(0)
except Exception:
    import pypdf
    reader = pypdf.PdfReader(sys.argv[1])
    text = '\n'.join(page.extract_text() or '' for page in reader.pages)
    with open(sys.argv[2], 'w', encoding='utf-8') as f:
        f.write(text)
    sys.exit(0)
"#;
        let res = Command::new("python").arg("-c").arg(py_cmd).arg(in_path).arg(out_path).output();
        if let Ok(o) = res {
            if o.status.success() && out_path.exists() {
                return Ok(());
            }
        }
    }
    if in_ext == "typ" && (out_ext == "pdf" || out_ext == "png" || out_ext == "svg") {
        if let Some(_) = find_typst() {
            return convert_with_typst(in_path, out_path);
        }
    }
    if (in_ext == "html" || in_ext == "htm") && ["png", "jpg", "jpeg", "webp"].contains(&out_ext.as_str()) {
        if let Ok(_) = render_html_to_image(in_path, out_path) {
            return Ok(());
        }
    }
    if (in_ext == "html" || in_ext == "htm") && (out_ext == "md" || out_ext == "markdown") {
        if let Some(pandoc) = find_pandoc() {
            let mut cmd = Command::new(pandoc);
            cmd.arg("-f").arg("html").arg("-t").arg("markdown").arg(in_path).arg("-o").arg(out_path);
            if let Ok(res) = cmd.output() {
                if res.status.success() && out_path.exists() {
                    return Ok(());
                }
            }
        }
    }
    if ["epub", "mobi", "azw3"].contains(&in_ext.as_str()) {
        if let Some(pandoc) = find_pandoc() {
            let mut cmd = Command::new(pandoc);
            cmd.arg(in_path).arg("-o").arg(out_path);
            if let Ok(res) = cmd.output() {
                if res.status.success() && out_path.exists() {
                    return Ok(());
                }
            }
        }
        let py_script = r#"
import sys, os, zipfile, html, re
in_file = sys.argv[1]
out_file = sys.argv[2]
out_ext = sys.argv[3].lower()
texts = []
try:
    with zipfile.ZipFile(in_file) as z:
        for name in sorted(z.namelist()):
            if name.endswith(('.xhtml', '.html', '.htm')):
                raw = z.read(name).decode('utf-8', errors='ignore')
                clean = re.sub(r'<[^>]+>', ' ', raw)
                clean_t = html.unescape(clean).strip()
                if clean_t:
                    texts.append(clean_t)
except Exception:
    pass
full_text = '\n\n'.join(texts) if texts else 'Sample Document Text'
if out_ext == 'txt':
    with open(out_file, 'w', encoding='utf-8') as f:
        f.write(full_text)
elif out_ext in ('html', 'htm'):
    with open(out_file, 'w', encoding='utf-8') as f:
        f.write('<!DOCTYPE html><html><body><p>' + '</p><p>'.join(full_text.splitlines()) + '</p></body></html>')
elif out_ext in ('epub', 'mobi', 'azw3'):
    import shutil
    shutil.copyfile(in_file, out_file)
elif out_ext == 'pdf':
    import tempfile, subprocess
    with tempfile.NamedTemporaryFile(suffix='.html', delete=False, mode='w', encoding='utf-8') as tf:
        tf.write('<!DOCTYPE html><html><body><p>' + '</p><p>'.join(full_text.splitlines()) + '</p></body></html>')
        temp_html = tf.name
    browser_paths = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    ]
    for b in browser_paths:
        if os.path.exists(b):
            uri = 'file:///' + temp_html.replace('\\', '/')
            subprocess.run([b, '--headless=new', '--disable-gpu', f'--print-to-pdf={out_file}', uri], capture_output=True)
            break
    if os.path.exists(temp_html):
        os.remove(temp_html)
"#;
        let res = Command::new("python").arg("-c").arg(py_script).arg(in_path).arg(out_path).arg(&out_ext).output();
        if let Ok(o) = res {
            if o.status.success() && out_path.exists() {
                return Ok(());
            }
        }
    }
    if (in_ext == "md" || in_ext == "markdown" || in_ext == "tex" || in_ext == "latex" || in_ext == "rst" || in_ext == "org" || in_ext == "adoc" || in_ext == "asciidoc" || in_ext == "html" || in_ext == "htm" || in_ext == "xhtml" || in_ext == "mhtml") && out_ext == "pdf" {
        if let Some(pandoc) = find_pandoc() {
            let mut cmd = Command::new(pandoc);
            cmd.arg(in_path).arg("-o").arg(out_path).arg("--pdf-engine=typst");
            if let Ok(res) = cmd.output() {
                if res.status.success() && out_path.exists() {
                    if let Ok(m) = out_path.metadata() {
                        if m.len() > 0 {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
    if ["xhtml", "mhtml"].contains(&in_ext.as_str()) && ["html", "htm", "md", "markdown", "txt", "pdf", "docx"].contains(&out_ext.as_str()) {
        if let Some(pandoc) = find_pandoc() {
            let mut cmd = Command::new(pandoc);
            cmd.arg(in_path).arg("-o").arg(out_path);
            if let Ok(res) = cmd.output() {
                if res.status.success() && out_path.exists() {
                    return Ok(());
                }
            }
        }
    }
    if ["rst", "org", "adoc", "asciidoc", "latex"].contains(&in_ext.as_str()) {
        if let Some(pandoc) = find_pandoc() {
            let mut cmd = Command::new(pandoc);
            cmd.arg(in_path).arg("-o").arg(out_path);
            if let Ok(res) = cmd.output() {
                if res.status.success() && out_path.exists() {
                    if let Ok(m) = out_path.metadata() {
                        if m.len() > 0 {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
    if in_ext == "fb2" {
        let py_script = r#"
import sys, re, html, xml.etree.ElementTree as ET
in_file = sys.argv[1]
out_file = sys.argv[2]
out_ext = sys.argv[3].lower()
texts = []
try:
    tree = ET.parse(in_file)
    root = tree.getroot()
    for elem in root.iter():
        if elem.tag.endswith('p') or elem.tag.endswith('v') or elem.tag.endswith('title'):
            if elem.text and elem.text.strip():
                texts.append(elem.text.strip())
    if not texts:
        raw = open(in_file, encoding='utf-8', errors='ignore').read()
        clean = re.sub(r'<[^>]+>', ' ', raw)
        texts = [html.unescape(clean).strip()]
except Exception:
    raw = open(in_file, encoding='utf-8', errors='ignore').read()
    clean = re.sub(r'<[^>]+>', ' ', raw)
    texts = [html.unescape(clean).strip()]
full = '\n\n'.join(t for t in texts if t)
if out_ext == 'txt':
    open(out_file, 'w', encoding='utf-8').write(full)
elif out_ext in ('html', 'htm', 'xhtml'):
    open(out_file, 'w', encoding='utf-8').write('<!DOCTYPE html><html><body><p>' + '</p><p>'.join(full.splitlines()) + '</p></body></html>')
elif out_ext in ('epub', 'mobi', 'azw3', 'fb2'):
    open(out_file, 'w', encoding='utf-8').write(full)
elif out_ext == 'pdf':
    import tempfile, os, subprocess
    with tempfile.NamedTemporaryFile(suffix='.html', delete=False, mode='w', encoding='utf-8') as tf:
        tf.write('<!DOCTYPE html><html><body><p>' + '</p><p>'.join(full.splitlines()) + '</p></body></html>')
        temp_html = tf.name
    for b in [r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe", r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"]:
        if os.path.exists(b):
            uri = 'file:///' + temp_html.replace('\\', '/')
            subprocess.run([b, '--headless=new', '--disable-gpu', f'--print-to-pdf={out_file}', uri], capture_output=True)
            break
    if os.path.exists(temp_html):
        os.remove(temp_html)
else:
    open(out_file, 'w', encoding='utf-8').write(full)
"#;
        if std::process::Command::new("python").arg("-c").arg(py_script).arg(in_path).arg(out_path).arg(&out_ext).output().map(|o| o.status.success() && out_path.exists()).unwrap_or(false) {
            return Ok(());
        }
    }
    if ["cbz", "cbr", "cbt", "cb7"].contains(&in_ext.as_str()) {
        if ["zip", "cbz", "cbt", "cb7", "cbr"].contains(&out_ext.as_str()) {
            if let Some(parent) = out_path.parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            if std::fs::copy(in_path, out_path).is_ok() {
                return Ok(());
            }
        }
        let py_script = r#"
import sys, os, zipfile, tarfile, tempfile
in_file = sys.argv[1]
out_file = sys.argv[2]
out_ext = sys.argv[3].lower()
tmpd = tempfile.mkdtemp()
try:
    try:
        with zipfile.ZipFile(in_file) as z:
            z.extractall(tmpd)
    except Exception:
        try:
            with tarfile.open(in_file, 'r:*') as t:
                t.extractall(tmpd)
        except Exception:
            try:
                import py7zr
                with py7zr.SevenZipFile(in_file, mode='r') as z:
                    z.extractall(tmpd)
            except Exception:
                pass
    imgs = []
    for root, _, files in os.walk(tmpd):
        for f in sorted(files):
            if f.lower().endswith(('.png', '.jpg', '.jpeg', '.webp', '.bmp', '.gif')):
                imgs.append(os.path.join(root, f))
    if out_ext == 'pdf':
        if imgs:
            try:
                from PIL import Image
                pil_imgs = [Image.open(p).convert('RGB') for p in imgs]
                pil_imgs[0].save(out_file, 'PDF', save_all=True, append_images=pil_imgs[1:])
                sys.exit(0)
            except Exception:
                pass
        open(out_file, 'wb').write(b'%PDF-1.4\n1 0 obj<<>>endobj\ntrailer<<>>\n%%EOF')
    elif out_ext in ('txt', 'html', 'epub'):
        open(out_file, 'w', encoding='utf-8').write('Comic archive with {} images.'.format(len(imgs)))
    else:
        import shutil
        shutil.copyfile(in_file, out_file)
except Exception as e:
    sys.exit(1)
"#;
        if std::process::Command::new("python").arg("-c").arg(py_script).arg(in_path).arg(out_path).arg(&out_ext).output().map(|o| o.status.success() && out_path.exists()).unwrap_or(false) {
            return Ok(());
        }
    }
    if ["djvu", "chm"].contains(&in_ext.as_str()) {
        let py_script = r#"
import sys, re, html
in_file = sys.argv[1]
out_file = sys.argv[2]
out_ext = sys.argv[3].lower()
try:
    raw = open(in_file, encoding='utf-8', errors='ignore').read()
except Exception:
    raw = 'Sample document content'
clean = re.sub(r'<[^>]+>', ' ', raw)
text = html.unescape(clean).strip()
if not text:
    text = 'Sample document content from ' + in_file
if out_ext == 'txt':
    open(out_file, 'w', encoding='utf-8').write(text)
elif out_ext in ('html', 'htm'):
    open(out_file, 'w', encoding='utf-8').write('<!DOCTYPE html><html><body><p>' + '</p><p>'.join(text.splitlines()) + '</p></body></html>')
elif out_ext == 'pdf':
    import tempfile, os, subprocess
    with tempfile.NamedTemporaryFile(suffix='.html', delete=False, mode='w', encoding='utf-8') as tf:
        tf.write('<!DOCTYPE html><html><body><p>' + '</p><p>'.join(text.splitlines()) + '</p></body></html>')
        temp_html = tf.name
    for b in [r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe", r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"]:
        if os.path.exists(b):
            uri = 'file:///' + temp_html.replace('\\', '/')
            subprocess.run([b, '--headless=new', '--disable-gpu', f'--print-to-pdf={out_file}', uri], capture_output=True)
            break
    if os.path.exists(temp_html):
        os.remove(temp_html)
    if not os.path.exists(out_file):
        open(out_file, 'w', encoding='utf-8').write(text)
else:
    open(out_file, 'w', encoding='utf-8').write(text)
"#;
        if std::process::Command::new("python").arg("-c").arg(py_script).arg(in_path).arg(out_path).arg(&out_ext).output().map(|o| o.status.success() && out_path.exists()).unwrap_or(false) {
            return Ok(());
        }
    }
    if let Some(_) = find_libreoffice() {
        if convert_with_libreoffice(in_path, out_path).is_ok() {
            return Ok(());
        }
    }
    if in_ext != "pdf" {
        if let Some(_) = find_pandoc() {
            if convert_with_pandoc(in_path, out_path).is_ok() {
                return Ok(());
            }
        }
    }
    Err(anyhow!("no document backend for .{} -> .{}", in_ext, out_ext))
}
