import os
import sys
import json
import urllib.request
import subprocess
import shutil
from pathlib import Path
BASE_DIR = Path(__file__).resolve().parent.parent
SAMPLES_DIR = BASE_DIR / "tests" / "samples"
FFMPEG = r"C:\Users\khrot\Desktop\Apps\ffmpeg\bin\ffmpeg.exe"
MAGICK = r"C:\Program Files\ImageMagick-7.1.2-Q16-HDRI\magick.exe"
SOFFICE = r"C:\Program Files\LibreOffice\program\soffice.exe"
PANDOC = os.path.expandvars(r"%LOCALAPPDATA%\Pandoc\pandoc.exe")
TYPST = os.path.expandvars(r"%LOCALAPPDATA%\Microsoft\WinGet\Links\typst.exe")
def ensure_dir(d):
    d.mkdir(parents=True, exist_ok=True)
def download_url(url, dest):
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
        with urllib.request.urlopen(req, timeout=15) as resp:
            data = resp.read()
            if len(data) > 0:
                with open(dest, "wb") as f:
                    f.write(data)
                return True
    except Exception:
        pass
    return False
def create_audio_samples():
    cat_dir = SAMPLES_DIR / "Audio"
    ensure_dir(cat_dir)
    base_wav = cat_dir / "base.wav"
    subprocess.run([FFMPEG, "-y", "-f", "lavfi", "-i", "sine=frequency=440:duration=1", str(base_wav)], capture_output=True)
    audio_formats = [
        ("mp3", ["-acodec", "libmp3lame"]),
        ("wav", ["-acodec", "pcm_s16le"]),
        ("flac", ["-acodec", "flac"]),
        ("aac", ["-acodec", "aac"]),
        ("m4a", ["-acodec", "aac"]),
        ("m4b", ["-acodec", "aac"]),
        ("m4r", ["-acodec", "aac"]),
        ("ogg", ["-acodec", "libvorbis"]),
        ("oga", ["-acodec", "libvorbis"]),
        ("opus", ["-acodec", "libopus"]),
        ("wma", ["-acodec", "wmav2"]),
        ("aiff", ["-acodec", "pcm_s16be"]),
        ("aif", ["-acodec", "pcm_s16be"]),
        ("ape", ["-acodec", "ape"]),
        ("alac", ["-acodec", "alac"]),
        ("ac3", ["-acodec", "ac3"]),
        ("eac3", ["-acodec", "eac3"]),
        ("dts", ["-acodec", "dca", "-strict", "-2"]),
        ("amr", ["-acodec", "amr_nb", "-ar", "8000", "-ac", "1"]),
        ("wv", ["-acodec", "wavpack"]),
        ("cda", ["-acodec", "pcm_s16le"]),
    ]
    for ext, args in audio_formats:
        out = cat_dir / f"sample.{ext}"
        cmd = [FFMPEG, "-y", "-i", str(base_wav)] + args + [str(out)]
        subprocess.run(cmd, capture_output=True)
    download_url("https://raw.githubusercontent.com/colinbdclark/midi-test-files/master/midi/C_major_scale.mid", cat_dir / "sample.mid")
    if (cat_dir / "sample.mid").exists():
        shutil.copy(cat_dir / "sample.mid", cat_dir / "sample.midi")
    if base_wav.exists():
        base_wav.unlink()
def create_video_samples():
    cat_dir = SAMPLES_DIR / "Video"
    ensure_dir(cat_dir)
    base_mp4 = cat_dir / "base.mp4"
    subprocess.run([FFMPEG, "-y", "-f", "lavfi", "-i", "testsrc=duration=1:size=320x240:rate=24", "-f", "lavfi", "-i", "sine=frequency=440:duration=1", "-c:v", "libx264", "-c:a", "aac", str(base_mp4)], capture_output=True)
    video_formats = [
        ("mp4", ["-c:v", "libx264", "-c:a", "aac"]),
        ("mkv", ["-c:v", "libx264", "-c:a", "aac"]),
        ("webm", ["-c:v", "libvpx-vp9", "-c:a", "libopus"]),
        ("avi", ["-c:v", "mpeg4", "-c:a", "mp3"]),
        ("mov", ["-c:v", "libx264", "-c:a", "aac"]),
        ("wmv", ["-c:v", "wmv2", "-c:a", "wmav2"]),
        ("flv", ["-c:v", "flv", "-c:a", "mp3"]),
        ("m4v", ["-c:v", "libx264", "-c:a", "aac"]),
        ("3gp", ["-c:v", "h263", "-c:a", "amr_nb", "-ar", "8000", "-ac", "1", "-s", "176x144"]),
        ("3gpp", ["-c:v", "h263", "-c:a", "amr_nb", "-ar", "8000", "-ac", "1", "-s", "176x144"]),
        ("3g2", ["-c:v", "h263", "-c:a", "amr_nb", "-ar", "8000", "-ac", "1", "-s", "176x144"]),
        ("ts", ["-c:v", "mpeg2video", "-c:a", "mp2"]),
        ("mts", ["-c:v", "mpeg2video", "-c:a", "mp2"]),
        ("m2ts", ["-c:v", "mpeg2video", "-c:a", "mp2"]),
        ("vob", ["-c:v", "mpeg2video", "-c:a", "mp2"]),
        ("mpg", ["-c:v", "mpeg1video", "-c:a", "mp2"]),
        ("mpeg", ["-c:v", "mpeg1video", "-c:a", "mp2"]),
        ("m2v", ["-c:v", "mpeg2video", "-an"]),
        ("f4v", ["-c:v", "libx264", "-c:a", "aac"]),
        ("asf", ["-c:v", "wmv2", "-c:a", "wmav2"]),
        ("y4m", ["-pix_fmt", "yuv420p"]),
    ]
    for ext, args in video_formats:
        out = cat_dir / f"sample.{ext}"
        cmd = [FFMPEG, "-y", "-i", str(base_mp4)] + args + [str(out)]
        subprocess.run(cmd, capture_output=True)
    subprocess.run([FFMPEG, "-y", "-f", "lavfi", "-i", "testsrc=duration=1:size=320x240:rate=24", "-f", "lavfi", "-i", "sine=frequency=440:duration=1", "-c:v", "libtheora", "-pix_fmt", "yuv420p", "-c:a", "libvorbis", "-f", "ogg", str(cat_dir / "sample.ogv")], capture_output=True)
    download_url("https://samples.mplayerhq.hu/game-formats/bink/smk_bik/intro.bik", cat_dir / "sample.bik")
    if (cat_dir / "sample.bik").exists():
        shutil.copy(cat_dir / "sample.bik", cat_dir / "sample.bik2")
    download_url("https://samples.mplayerhq.hu/real/general/intro.rm", cat_dir / "sample.rm")
    if (cat_dir / "sample.rm").exists():
        shutil.copy(cat_dir / "sample.rm", cat_dir / "sample.rmvb")
    if base_mp4.exists():
        base_mp4.unlink()
def create_image_samples():
    cat_dir = SAMPLES_DIR / "Image"
    ensure_dir(cat_dir)
    from PIL import Image, ImageDraw
    img = Image.new("RGBA", (128, 128), (255, 100, 50, 255))
    d = ImageDraw.Draw(img)
    d.rectangle([16, 16, 112, 112], outline=(255, 255, 255, 255), fill=(40, 80, 160, 255), width=3)
    base_png = cat_dir / "base.png"
    img.save(base_png, "PNG")
    img.save(cat_dir / "sample.png", "PNG")
    img.convert("RGB").save(cat_dir / "sample.jpg", "JPEG")
    img.convert("RGB").save(cat_dir / "sample.jpeg", "JPEG")
    img.convert("RGB").save(cat_dir / "sample.jfif", "JPEG")
    img.save(cat_dir / "sample.webp", "WEBP")
    img.convert("RGB").save(cat_dir / "sample.bmp", "BMP")
    img.save(cat_dir / "sample.ico", "ICO", sizes=[(64, 64), (32, 32), (16, 16)])
    img.save(cat_dir / "sample.cur", "ICO", sizes=[(32, 32)])
    img.save(cat_dir / "sample.tiff", "TIFF")
    img.save(cat_dir / "sample.tif", "TIFF")
    img.save(cat_dir / "sample.tga", "TGA")
    img.save(cat_dir / "sample.gif", "GIF")
    img.convert("RGB").save(cat_dir / "sample.ppm", "PPM")
    img.convert("L").save(cat_dir / "sample.pgm", "PPM")
    img.convert("1").save(cat_dir / "sample.pbm", "PPM")
    img.convert("RGB").save(cat_dir / "sample.pnm", "PPM")
    subprocess.run([MAGICK, str(base_png), str(cat_dir / "sample.pam")], capture_output=True)
    img.convert("P").save(cat_dir / "sample.pcx", "PCX")
    subprocess.run([MAGICK, str(base_png), str(cat_dir / "sample.dds")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(cat_dir / "sample.psd")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(cat_dir / "sample.psb")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(cat_dir / "sample.hdr")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(cat_dir / "sample.exr")], capture_output=True)
    img.convert("RGB").save(cat_dir / "sample.eps", "EPS")
    subprocess.run([MAGICK, str(base_png), str(cat_dir / "sample.icns")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(cat_dir / "sample.avif")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(cat_dir / "sample.jxl")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(cat_dir / "sample.heic")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(cat_dir / "sample.heif")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(cat_dir / "sample.hif")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(cat_dir / "sample.qoi")], capture_output=True)
    svg_content = '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="red"/><circle cx="50" cy="50" r="40" fill="blue"/></svg>'
    (cat_dir / "sample.svg").write_text(svg_content, encoding="utf-8")
    import gzip
    with gzip.open(cat_dir / "sample.svgz", "wb") as f:
        f.write(svg_content.encode("utf-8"))
    for raw_e in ["dng", "cr2", "cr3", "nef", "arw", "orf", "rw2", "raf", "xcf", "ai"]:
        img.save(cat_dir / f"sample.{raw_e}", "PNG")
    if base_png.exists():
        base_png.unlink()
def create_document_samples():
    cat_dir = SAMPLES_DIR / "Document"
    ensure_dir(cat_dir)
    (cat_dir / "sample.txt").write_text("Hello rf2f document conversion test.\nLine 2 text.", encoding="utf-8")
    (cat_dir / "sample.md").write_text("# Document Title\n\nThis is a Markdown sample for **rf2f**.\n- Item 1\n- Item 2", encoding="utf-8")
    (cat_dir / "sample.markdown").write_text("# Document Title\n\nThis is a Markdown sample for **rf2f**.", encoding="utf-8")
    (cat_dir / "sample.html").write_text("<!DOCTYPE html><html><head><title>Test</title></head><body><h1>Document Title</h1><p>Test paragraph content.</p></body></html>", encoding="utf-8")
    (cat_dir / "sample.htm").write_text("<!DOCTYPE html><html><body><h1>Document Title</h1><p>Test paragraph content.</p></body></html>", encoding="utf-8")
    (cat_dir / "sample.tex").write_text("\\documentclass{article}\n\\begin{document}\nHello LaTeX world from rf2f!\n\\end{document}", encoding="utf-8")
    (cat_dir / "sample.typ").write_text("= Typst Document\nHello world from Typst in rf2f!", encoding="utf-8")
    subprocess.run([SOFFICE, "--headless", "--convert-to", "pdf:writer_pdf_Export", str(cat_dir / "sample.txt"), "--outdir", str(cat_dir)], capture_output=True)
    subprocess.run([SOFFICE, "--headless", "--convert-to", "docx:MS Word 2007 XML", str(cat_dir / "sample.txt"), "--outdir", str(cat_dir)], capture_output=True)
    subprocess.run([SOFFICE, "--headless", "--convert-to", "doc:MS Word 97", str(cat_dir / "sample.txt"), "--outdir", str(cat_dir)], capture_output=True)
    subprocess.run([SOFFICE, "--headless", "--convert-to", "odt:writer8", str(cat_dir / "sample.txt"), "--outdir", str(cat_dir)], capture_output=True)
    subprocess.run([SOFFICE, "--headless", "--convert-to", "rtf:Rich Text Format", str(cat_dir / "sample.txt"), "--outdir", str(cat_dir)], capture_output=True)
    csv_temp = cat_dir / "temp_sheet.csv"
    csv_temp.write_text("A,B,C\n1,2,3\n4,5,6\n", encoding="utf-8")
    subprocess.run([SOFFICE, "--headless", "--convert-to", "xlsx:Calc MS Excel 2007 XML", str(csv_temp), "--outdir", str(cat_dir)], capture_output=True)
    if (cat_dir / "temp_sheet.xlsx").exists():
        (cat_dir / "sample.xlsx").unlink(missing_ok=True)
        (cat_dir / "temp_sheet.xlsx").rename(cat_dir / "sample.xlsx")
    subprocess.run([SOFFICE, "--headless", "--convert-to", "xls:MS Excel 97", str(csv_temp), "--outdir", str(cat_dir)], capture_output=True)
    if (cat_dir / "temp_sheet.xls").exists():
        (cat_dir / "sample.xls").unlink(missing_ok=True)
        (cat_dir / "temp_sheet.xls").rename(cat_dir / "sample.xls")
    subprocess.run([SOFFICE, "--headless", "--convert-to", "ods:calc8", str(csv_temp), "--outdir", str(cat_dir)], capture_output=True)
    if (cat_dir / "temp_sheet.ods").exists():
        (cat_dir / "sample.ods").unlink(missing_ok=True)
        (cat_dir / "temp_sheet.ods").rename(cat_dir / "sample.ods")
    csv_temp.unlink(missing_ok=True)
    if (BASE_DIR.parent / "Downloads" / "chapter 6.pages").exists():
        shutil.copy(BASE_DIR.parent / "Downloads" / "chapter 6.pages", cat_dir / "sample.pages")
    doc_downloads = [
        ("sample.epub", "https://raw.githubusercontent.com/IDPF/epub3-samples/master/30/wasteland/wasteland.epub"),
        ("sample.mobi", "https://raw.githubusercontent.com/alexpovel/srndpt/master/test_data/sample.mobi"),
        ("sample.azw3", "https://raw.githubusercontent.com/koreader/koreader/master/spec/unit/data/sample.azw3"),
        ("sample.pptx", "https://raw.githubusercontent.com/gitbrent/PptxGenJS/master/demos/common/sample.pptx"),
        ("sample.ppt", "https://github.com/scanny/python-pptx/raw/master/tests/test_files/simple.ppt"),
        ("sample.odp", "https://github.com/scanny/python-pptx/raw/master/tests/test_files/simple.odp"),
        ("sample.numbers", "https://raw.githubusercontent.com/masahiroy/node-numbers-parser/master/test/sample.numbers"),
        ("sample.key", "https://raw.githubusercontent.com/obenshi/keynote-parser/master/tests/sample.key"),
    ]
    for name, url in doc_downloads:
        dest = cat_dir / name
        download_url(url, dest)
def create_data_samples():
    cat_dir = SAMPLES_DIR / "Data"
    ensure_dir(cat_dir)
    data = [{"id": 1, "name": "Item A", "price": 19.99, "active": True}, {"id": 2, "name": "Item B", "price": 29.50, "active": False}]
    (cat_dir / "sample.json").write_text(json.dumps(data, indent=2), encoding="utf-8")
    (cat_dir / "sample.json5").write_text("{\n  // JSON5 sample\n  unquoted: 'value',\n  number: 42,\n}", encoding="utf-8")
    (cat_dir / "sample.jsonc").write_text("{\n  // JSON with comments\n  \"key\": \"value\",\n  \"count\": 10\n}", encoding="utf-8")
    (cat_dir / "sample.yaml").write_text("- id: 1\n  name: Item A\n  price: 19.99\n  active: true\n- id: 2\n  name: Item B\n  price: 29.5\n  active: false", encoding="utf-8")
    (cat_dir / "sample.yml").write_text("title: Sample YAML\nversion: 1.0", encoding="utf-8")
    (cat_dir / "sample.toml").write_text("[package]\nname = \"sample\"\nversion = \"0.1.0\"\nenabled = true", encoding="utf-8")
    (cat_dir / "sample.xml").write_text("<root><item id=\"1\"><name>Item A</name></item></root>", encoding="utf-8")
    (cat_dir / "sample.csv").write_text("id,name,price,active\n1,Item A,19.99,true\n2,Item B,29.50,false", encoding="utf-8")
    (cat_dir / "sample.tsv").write_text("id\tname\tprice\tactive\n1\tItem A\t19.99\ttrue\n2\tItem B\t29.50\tfalse", encoding="utf-8")
    (cat_dir / "sample.ron").write_text("(\n    name: \"Sample\",\n    value: 42,\n)", encoding="utf-8")
    (cat_dir / "sample.plist").write_text("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict><key>Name</key><string>rf2f</string></dict></plist>", encoding="utf-8")
    (cat_dir / "sample.ini").write_text("[database]\nhost = localhost\nport = 5432", encoding="utf-8")
    (cat_dir / "sample.env").write_text("APP_ENV=production\nPORT=8080", encoding="utf-8")
def create_model3d_samples():
    cat_dir = SAMPLES_DIR / "Model3D"
    ensure_dir(cat_dir)
    obj_data = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3\n"
    (cat_dir / "sample.obj").write_text(obj_data, encoding="utf-8")
    stl_data = "solid cube\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0\nvertex 1 1 0\nendloop\nendfacet\nendsolid cube\n"
    (cat_dir / "sample.stl").write_text(stl_data, encoding="utf-8")
    ply_data = "ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nelement face 1\nproperty list uchar int vertex_indices\nend_header\n0 0 0\n1 0 0\n0 1 0\n3 0 1 2\n"
    (cat_dir / "sample.ply").write_text(ply_data, encoding="utf-8")
    gltf_data = '{"asset":{"version":"2.0"},"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}]}'
    (cat_dir / "sample.gltf").write_text(gltf_data, encoding="utf-8")
    models = [
        ("sample.glb", "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Models/master/2.0/Box/glTF-Binary/Box.glb"),
        ("sample.fbx", "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Models/master/2.0/Box/FBX/Box.fbx"),
        ("sample.3mf", "https://raw.githubusercontent.com/3MFConsortium/3mf-samples/master/examples/box.3mf"),
        ("sample.usdz", "https://raw.githubusercontent.com/google/model-viewer/master/packages/shared-assets/models/Astronaut.usdz"),
    ]
    for name, url in models:
        dest = cat_dir / name
        download_url(url, dest)
def create_archive_samples():
    cat_dir = SAMPLES_DIR / "Archive"
    ensure_dir(cat_dir)
    import zipfile, tarfile, gzip
    txt_content = b"rf2f archive test content."
    with zipfile.ZipFile(cat_dir / "sample.zip", "w") as z:
        z.writestr("test.txt", txt_content)
    with tarfile.open(cat_dir / "sample.tar", "w") as t:
        import io, time
        ti = tarfile.TarInfo("test.txt")
        ti.size = len(txt_content)
        ti.mtime = int(time.time())
        t.addfile(ti, io.BytesIO(txt_content))
    with gzip.open(cat_dir / "sample.gz", "wb") as f:
        f.write(txt_content)
    with tarfile.open(cat_dir / "sample.tgz", "w:gz") as t:
        import io, time
        ti = tarfile.TarInfo("test.txt")
        ti.size = len(txt_content)
        ti.mtime = int(time.time())
        t.addfile(ti, io.BytesIO(txt_content))
    download_url("https://raw.githubusercontent.com/mcmtroffaes/pys7z/master/tests/sample.7z", cat_dir / "sample.7z")
    download_url("https://raw.githubusercontent.com/markokr/rarfile/master/test/files/rar3-simple.rar", cat_dir / "sample.rar")
def create_font_samples():
    cat_dir = SAMPLES_DIR / "Font"
    ensure_dir(cat_dir)
    sys_fonts = [
        ("C:\\Windows\\Fonts\\arial.ttf", "sample.ttf"),
        ("C:\\Windows\\Fonts\\segoeui.ttf", "sample.otf"),
    ]
    for src, name in sys_fonts:
        if os.path.exists(src):
            shutil.copy(src, cat_dir / name)
    font_urls = [
        ("sample.woff", "https://raw.githubusercontent.com/google/fonts/main/apache/robotoserif/static/RobotoSerif-Regular.woff"),
        ("sample.woff2", "https://raw.githubusercontent.com/google/fonts/main/apache/robotoserif/static/RobotoSerif-Regular.woff2"),
        ("sample.eot", "https://raw.githubusercontent.com/google/fonts/main/apache/robotoserif/static/RobotoSerif-Regular.eot"),
    ]
    for name, url in font_urls:
        dest = cat_dir / name
        download_url(url, dest)
def create_subtitle_samples():
    cat_dir = SAMPLES_DIR / "Subtitle"
    ensure_dir(cat_dir)
    (cat_dir / "sample.srt").write_text("1\n00:00:01,000 --> 00:00:04,000\nHello subtitle world.\n\n2\n00:00:05,000 --> 00:00:08,000\nTesting rf2f converter.\n", encoding="utf-8")
    (cat_dir / "sample.vtt").write_text("WEBVTT\n\n1\n00:00:01.000 --> 00:00:04.000\nHello subtitle world.\n\n2\n00:00:05.000 --> 00:00:08.000\nTesting rf2f converter.\n", encoding="utf-8")
    (cat_dir / "sample.ass").write_text("[Script Info]\nTitle: Test\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:04.00,Default,,0,0,0,,Hello subtitle world.\n", encoding="utf-8")
    (cat_dir / "sample.ssa").write_text("[Script Info]\nTitle: Test\n[Events]\nFormat: Marked, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: Marked=0,0:00:01.00,0:00:04.00,Default,,0,0,0,,Hello subtitle world.\n", encoding="utf-8")
    (cat_dir / "sample.lrc").write_text("[00:01.00]Hello lyric world.\n[00:05.00]Testing rf2f converter.\n", encoding="utf-8")
if __name__ == "__main__":
    create_audio_samples()
    create_video_samples()
    create_image_samples()
    create_document_samples()
    create_data_samples()
    create_model3d_samples()
    create_archive_samples()
    create_font_samples()
    create_subtitle_samples()
    print("Sample generation & downloads complete.")
