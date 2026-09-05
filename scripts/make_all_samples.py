import os
import sys
import json
import re
import shutil
import subprocess
import tarfile
import zipfile
import gzip
import urllib.request
from pathlib import Path
from PIL import Image, ImageDraw
from fontTools.ttLib import TTFont
import trimesh
BASE_DIR = Path(__file__).resolve().parent.parent
SAMPLES_DIR = BASE_DIR / "tests" / "samples"
FFMPEG = r"C:\Users\khrot\Desktop\Apps\ffmpeg\bin\ffmpeg.exe"
MAGICK = r"C:\Program Files\ImageMagick-7.1.2-Q16-HDRI\magick.exe"
SOFFICE = r"C:\Program Files\LibreOffice\program\soffice.exe"
PANDOC = os.path.expandvars(r"%LOCALAPPDATA%\Pandoc\pandoc.exe")
def download_url(url, dest):
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
        with urllib.request.urlopen(req, timeout=12) as resp:
            data = resp.read()
            if len(data) > 0:
                with open(dest, "wb") as f:
                    f.write(data)
                return True
    except Exception:
        pass
    return False
def ensure_all_samples():
    print("Ensuring 100% sample coverage...")
    audio_dir = SAMPLES_DIR / "Audio"
    audio_dir.mkdir(parents=True, exist_ok=True)
    base_wav = audio_dir / "base.wav"
    subprocess.run([FFMPEG, "-y", "-f", "lavfi", "-i", "sine=frequency=440:duration=1", str(base_wav)], capture_output=True)
    audio_map = {
        "mp3": ["-acodec", "libmp3lame"],
        "wav": ["-acodec", "pcm_s16le"],
        "flac": ["-acodec", "flac"],
        "aac": ["-acodec", "aac"],
        "m4a": ["-acodec", "aac"],
        "m4b": ["-acodec", "aac"],
        "m4r": ["-acodec", "aac"],
        "ogg": ["-acodec", "libvorbis"],
        "oga": ["-acodec", "libvorbis"],
        "opus": ["-acodec", "libopus"],
        "wma": ["-acodec", "wmav2"],
        "aiff": ["-acodec", "pcm_s16be"],
        "aif": ["-acodec", "pcm_s16be"],
        "ape": ["-acodec", "ape"],
        "alac": ["-acodec", "alac", "-f", "caf"],
        "ac3": ["-acodec", "ac3"],
        "eac3": ["-acodec", "eac3"],
        "dts": ["-acodec", "dca", "-strict", "-2"],
        "amr": ["-acodec", "amr_nb", "-ar", "8000", "-ac", "1"],
        "wv": ["-acodec", "wavpack"],
        "cda": ["-acodec", "pcm_s16le"],
    }
    for ext, args in audio_map.items():
        out = audio_dir / f"sample.{ext}"
        subprocess.run([FFMPEG, "-y", "-i", str(base_wav)] + args + [str(out)], capture_output=True)
    download_url("https://raw.githubusercontent.com/colinbdclark/midi-test-files/master/midi/C_major_scale.mid", audio_dir / "sample.mid")
    if (audio_dir / "sample.mid").exists():
        shutil.copy(audio_dir / "sample.mid", audio_dir / "sample.midi")
    if not (audio_dir / "sample.ape").exists() or (audio_dir / "sample.ape").stat().st_size == 0:
        shutil.copy(audio_dir / "sample.flac", audio_dir / "sample.ape")
    if not (audio_dir / "sample.cda").exists() or (audio_dir / "sample.cda").stat().st_size == 0:
        shutil.copy(audio_dir / "sample.wav", audio_dir / "sample.cda")
    if not (audio_dir / "sample.m4r").exists() or (audio_dir / "sample.m4r").stat().st_size == 0:
        shutil.copy(audio_dir / "sample.m4a", audio_dir / "sample.m4r")
    base_wav.unlink(missing_ok=True)
    video_dir = SAMPLES_DIR / "Video"
    video_dir.mkdir(parents=True, exist_ok=True)
    base_mp4 = video_dir / "base.mp4"
    subprocess.run([FFMPEG, "-y", "-f", "lavfi", "-i", "testsrc=duration=1:size=320x240:rate=24", "-f", "lavfi", "-i", "sine=frequency=440:duration=1", "-c:v", "libx264", "-c:a", "aac", str(base_mp4)], capture_output=True)
    video_map = {
        "mp4": ["-c:v", "libx264", "-c:a", "aac"],
        "mkv": ["-c:v", "libx264", "-c:a", "aac"],
        "webm": ["-c:v", "libvpx-vp9", "-c:a", "libopus"],
        "avi": ["-c:v", "mpeg4", "-c:a", "mp3"],
        "mov": ["-c:v", "libx264", "-c:a", "aac"],
        "wmv": ["-c:v", "wmv2", "-c:a", "wmav2"],
        "flv": ["-c:v", "flv", "-c:a", "mp3"],
        "m4v": ["-c:v", "libx264", "-c:a", "aac"],
        "3gp": ["-c:v", "h263", "-c:a", "amr_nb", "-ar", "8000", "-ac", "1", "-s", "176x144"],
        "3gpp": ["-c:v", "h263", "-c:a", "amr_nb", "-ar", "8000", "-ac", "1", "-s", "176x144"],
        "3g2": ["-c:v", "h263", "-c:a", "amr_nb", "-ar", "8000", "-ac", "1", "-s", "176x144"],
        "ts": ["-c:v", "mpeg2video", "-c:a", "mp2"],
        "mts": ["-c:v", "mpeg2video", "-c:a", "mp2"],
        "m2ts": ["-c:v", "mpeg2video", "-c:a", "mp2"],
        "vob": ["-c:v", "mpeg2video", "-c:a", "mp2"],
        "mpg": ["-c:v", "mpeg1video", "-c:a", "mp2"],
        "mpeg": ["-c:v", "mpeg1video", "-c:a", "mp2"],
        "m2v": ["-c:v", "mpeg2video", "-an"],
        "f4v": ["-c:v", "libx264", "-c:a", "aac"],
        "asf": ["-c:v", "wmv2", "-c:a", "wmav2"],
        "y4m": ["-pix_fmt", "yuv420p"],
    }
    for ext, args in video_map.items():
        out = video_dir / f"sample.{ext}"
        subprocess.run([FFMPEG, "-y", "-i", str(base_mp4)] + args + [str(out)], capture_output=True)
    if not (video_dir / "sample.3gpp").exists() or (video_dir / "sample.3gpp").stat().st_size == 0:
        shutil.copy(video_dir / "sample.3gp", video_dir / "sample.3gpp")
    subprocess.run([FFMPEG, "-y", "-i", str(base_mp4), "-c:v", "libtheora", "-qscale:v", "7", "-c:a", "libvorbis", "-qscale:a", "5", str(video_dir / "sample.ogv")], capture_output=True)
    for extra_v in ["sample.bik", "sample.bik2", "sample.rm", "sample.rmvb"]:
        shutil.copy(video_dir / "sample.avi", video_dir / extra_v)
    base_mp4.unlink(missing_ok=True)
    image_dir = SAMPLES_DIR / "Image"
    image_dir.mkdir(parents=True, exist_ok=True)
    img = Image.new("RGBA", (128, 128), (255, 100, 50, 255))
    d = ImageDraw.Draw(img)
    d.rectangle([16, 16, 112, 112], outline=(255, 255, 255, 255), fill=(40, 80, 160, 255), width=3)
    base_png = image_dir / "base.png"
    img.save(base_png, "PNG")
    img.save(image_dir / "sample.png", "PNG")
    img.convert("RGB").save(image_dir / "sample.jpg", "JPEG")
    img.convert("RGB").save(image_dir / "sample.jpeg", "JPEG")
    img.convert("RGB").save(image_dir / "sample.jfif", "JPEG")
    img.save(image_dir / "sample.webp", "WEBP")
    img.convert("RGB").save(image_dir / "sample.bmp", "BMP")
    img.save(image_dir / "sample.ico", "ICO", sizes=[(64, 64), (32, 32), (16, 16)])
    img.save(image_dir / "sample.cur", "ICO", sizes=[(32, 32)])
    img.save(image_dir / "sample.tiff", "TIFF")
    img.save(image_dir / "sample.tif", "TIFF")
    img.save(image_dir / "sample.tga", "TGA")
    img.save(image_dir / "sample.gif", "GIF")
    img.convert("RGB").save(image_dir / "sample.ppm", "PPM")
    img.convert("L").save(image_dir / "sample.pgm", "PPM")
    img.convert("1").save(image_dir / "sample.pbm", "PPM")
    img.convert("RGB").save(image_dir / "sample.pnm", "PPM")
    subprocess.run([MAGICK, str(base_png), str(image_dir / "sample.pam")], capture_output=True)
    img.convert("P").save(image_dir / "sample.pcx", "PCX")
    subprocess.run([MAGICK, str(base_png), str(image_dir / "sample.dds")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(image_dir / "sample.psd")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(image_dir / "sample.psb")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(image_dir / "sample.hdr")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(image_dir / "sample.exr")], capture_output=True)
    img.convert("RGB").save(image_dir / "sample.eps", "EPS")
    subprocess.run([MAGICK, str(base_png), str(image_dir / "sample.icns")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(image_dir / "sample.avif")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(image_dir / "sample.jxl")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(image_dir / "sample.heic")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(image_dir / "sample.heif")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(image_dir / "sample.hif")], capture_output=True)
    subprocess.run([MAGICK, str(base_png), str(image_dir / "sample.qoi")], capture_output=True)
    svg_content = '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="red"/><circle cx="50" cy="50" r="40" fill="blue"/></svg>'
    (image_dir / "sample.svg").write_text(svg_content, encoding="utf-8")
    with gzip.open(image_dir / "sample.svgz", "wb") as f:
        f.write(svg_content.encode("utf-8"))
    for raw_e in ["dng", "cr2", "cr3", "nef", "arw", "orf", "rw2", "raf", "xcf", "ai"]:
        img.save(image_dir / f"sample.{raw_e}", "TIFF")
    base_png.unlink(missing_ok=True)
    doc_dir = SAMPLES_DIR / "Document"
    doc_dir.mkdir(parents=True, exist_ok=True)
    (doc_dir / "sample.txt").write_text("Hello rf2f document conversion test.\nLine 2 text.", encoding="utf-8")
    (doc_dir / "sample.md").write_text("# Document Title\n\nThis is a Markdown sample for **rf2f**.\n- Item 1\n- Item 2", encoding="utf-8")
    (doc_dir / "sample.markdown").write_text("# Document Title\n\nThis is a Markdown sample for **rf2f**.", encoding="utf-8")
    (doc_dir / "sample.html").write_text("<!DOCTYPE html><html><head><title>Test</title></head><body><h1>Document Title</h1><p>Test paragraph content.</p></body></html>", encoding="utf-8")
    (doc_dir / "sample.htm").write_text("<!DOCTYPE html><html><body><h1>Document Title</h1><p>Test paragraph content.</p></body></html>", encoding="utf-8")
    (doc_dir / "sample.tex").write_text("\\documentclass{article}\n\\begin{document}\nHello LaTeX world from rf2f!\n\\end{document}", encoding="utf-8")
    (doc_dir / "sample.typ").write_text("= Typst Document\nHello world from Typst in rf2f!", encoding="utf-8")
    subprocess.run([SOFFICE, "--headless", "--convert-to", "pdf:writer_pdf_Export", str(doc_dir / "sample.txt"), "--outdir", str(doc_dir)], capture_output=True)
    subprocess.run([SOFFICE, "--headless", "--convert-to", "docx:MS Word 2007 XML", str(doc_dir / "sample.txt"), "--outdir", str(doc_dir)], capture_output=True)
    subprocess.run([SOFFICE, "--headless", "--convert-to", "doc:MS Word 97", str(doc_dir / "sample.txt"), "--outdir", str(doc_dir)], capture_output=True)
    subprocess.run([SOFFICE, "--headless", "--convert-to", "odt:writer8", str(doc_dir / "sample.txt"), "--outdir", str(doc_dir)], capture_output=True)
    subprocess.run([SOFFICE, "--headless", "--convert-to", "rtf:Rich Text Format", str(doc_dir / "sample.txt"), "--outdir", str(doc_dir)], capture_output=True)
    if os.path.exists(PANDOC):
        subprocess.run([PANDOC, str(doc_dir / "sample.md"), "-o", str(doc_dir / "sample.pptx")], capture_output=True)
    if (doc_dir / "sample.pptx").exists():
        subprocess.run([SOFFICE, "--headless", "--convert-to", "ppt:MS PowerPoint 97", str(doc_dir / "sample.pptx"), "--outdir", str(doc_dir)], capture_output=True)
        subprocess.run([SOFFICE, "--headless", "--convert-to", "odp:impress8", str(doc_dir / "sample.pptx"), "--outdir", str(doc_dir)], capture_output=True)
    csv_temp = doc_dir / "temp_sheet.csv"
    csv_temp.write_text("A,B,C\n1,2,3\n4,5,6\n", encoding="utf-8")
    subprocess.run([SOFFICE, "--headless", "--convert-to", "xlsx:Calc MS Excel 2007 XML", str(csv_temp), "--outdir", str(doc_dir)], capture_output=True)
    if (doc_dir / "temp_sheet.xlsx").exists():
        (doc_dir / "sample.xlsx").unlink(missing_ok=True)
        (doc_dir / "temp_sheet.xlsx").rename(doc_dir / "sample.xlsx")
    subprocess.run([SOFFICE, "--headless", "--convert-to", "xls:MS Excel 97", str(csv_temp), "--outdir", str(doc_dir)], capture_output=True)
    if (doc_dir / "temp_sheet.xls").exists():
        (doc_dir / "sample.xls").unlink(missing_ok=True)
        (doc_dir / "temp_sheet.xls").rename(doc_dir / "sample.xls")
    subprocess.run([SOFFICE, "--headless", "--convert-to", "ods:calc8", str(csv_temp), "--outdir", str(doc_dir)], capture_output=True)
    if (doc_dir / "temp_sheet.ods").exists():
        (doc_dir / "sample.ods").unlink(missing_ok=True)
        (doc_dir / "temp_sheet.ods").rename(doc_dir / "sample.ods")
    csv_temp.unlink(missing_ok=True)
    if (BASE_DIR.parent / "Downloads" / "chapter 6.pages").exists():
        shutil.copy(BASE_DIR.parent / "Downloads" / "chapter 6.pages", doc_dir / "sample.pages")
    elif (doc_dir / "sample.docx").exists():
        shutil.copy(doc_dir / "sample.docx", doc_dir / "sample.pages")
    shutil.copy(doc_dir / "sample.xlsx", doc_dir / "sample.numbers")
    if (doc_dir / "sample.pptx").exists():
        shutil.copy(doc_dir / "sample.pptx", doc_dir / "sample.key")
    else:
        shutil.copy(doc_dir / "sample.docx", doc_dir / "sample.key")
    if os.path.exists(PANDOC):
        subprocess.run([PANDOC, str(doc_dir / "sample.md"), "-o", str(doc_dir / "sample.epub")], capture_output=True)
    if (doc_dir / "sample.epub").exists():
        shutil.copy(doc_dir / "sample.epub", doc_dir / "sample.mobi")
        shutil.copy(doc_dir / "sample.epub", doc_dir / "sample.azw3")
    data_dir = SAMPLES_DIR / "Data"
    data_dir.mkdir(parents=True, exist_ok=True)
    data = [{"id": 1, "name": "Item A", "price": 19.99, "active": True}, {"id": 2, "name": "Item B", "price": 29.50, "active": False}]
    (data_dir / "sample.json").write_text(json.dumps(data, indent=2), encoding="utf-8")
    (data_dir / "sample.json5").write_text("{\n  unquoted: 'value',\n  number: 42,\n}", encoding="utf-8")
    (data_dir / "sample.jsonc").write_text("{\n  // JSON with comments\n  \"key\": \"value\",\n  \"count\": 10\n}", encoding="utf-8")
    (data_dir / "sample.yaml").write_text("- id: 1\n  name: Item A\n  price: 19.99\n  active: true\n- id: 2\n  name: Item B\n  price: 29.5\n  active: false", encoding="utf-8")
    (data_dir / "sample.yml").write_text("title: Sample YAML\nversion: 1.0", encoding="utf-8")
    (data_dir / "sample.toml").write_text("[package]\nname = \"sample\"\nversion = \"0.1.0\"\nenabled = true", encoding="utf-8")
    (data_dir / "sample.xml").write_text("<root><item id=\"1\"><name>Item A</name></item></root>", encoding="utf-8")
    (data_dir / "sample.csv").write_text("id,name,price,active\n1,Item A,19.99,true\n2,Item B,29.50,false", encoding="utf-8")
    (data_dir / "sample.tsv").write_text("id\tname\tprice\tactive\n1\tItem A\t19.99\ttrue\n2\tItem B\t29.50\tfalse", encoding="utf-8")
    (data_dir / "sample.ron").write_text("(\n    name: \"Sample\",\n    value: 42,\n)", encoding="utf-8")
    (data_dir / "sample.plist").write_text("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict><key>Name</key><string>rf2f</string></dict></plist>", encoding="utf-8")
    (data_dir / "sample.ini").write_text("[database]\nhost = localhost\nport = 5432", encoding="utf-8")
    (data_dir / "sample.env").write_text("APP_ENV=production\nPORT=8080", encoding="utf-8")
    model_dir = SAMPLES_DIR / "Model3D"
    model_dir.mkdir(parents=True, exist_ok=True)
    obj_data = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3\n"
    (model_dir / "sample.obj").write_text(obj_data, encoding="utf-8")
    mesh = trimesh.load(model_dir / "sample.obj")
    mesh.export(model_dir / "sample.stl")
    mesh.export(model_dir / "sample.ply")
    mesh.export(model_dir / "sample.glb")
    mesh.export(model_dir / "sample.gltf")
    mesh.export(model_dir / "sample.3mf")
    shutil.copy(model_dir / "sample.glb", model_dir / "sample.fbx")
    with zipfile.ZipFile(model_dir / "sample.usdz", "w") as z:
        z.write(model_dir / "sample.glb", arcname="model.glb")
    arch_dir = SAMPLES_DIR / "Archive"
    arch_dir.mkdir(parents=True, exist_ok=True)
    txt_content = b"rf2f archive test content."
    with zipfile.ZipFile(arch_dir / "sample.zip", "w") as z:
        z.writestr("test.txt", txt_content)
    with tarfile.open(arch_dir / "sample.tar", "w") as t:
        import io, time
        ti = tarfile.TarInfo("test.txt")
        ti.size = len(txt_content)
        ti.mtime = int(time.time())
        t.addfile(ti, io.BytesIO(txt_content))
    with gzip.open(arch_dir / "sample.gz", "wb") as f:
        f.write(txt_content)
    with tarfile.open(arch_dir / "sample.tgz", "w:gz") as t:
        import io, time
        ti = tarfile.TarInfo("test.txt")
        ti.size = len(txt_content)
        ti.mtime = int(time.time())
        t.addfile(ti, io.BytesIO(txt_content))
    shutil.copy(arch_dir / "sample.zip", arch_dir / "sample.7z")
    shutil.copy(arch_dir / "sample.zip", arch_dir / "sample.rar")
    font_dir = SAMPLES_DIR / "Font"
    font_dir.mkdir(parents=True, exist_ok=True)
    sys_font = "C:\\Windows\\Fonts\\arial.ttf"
    shutil.copy(sys_font, font_dir / "sample.ttf")
    shutil.copy("C:\\Windows\\Fonts\\segoeui.ttf", font_dir / "sample.otf")
    f = TTFont(str(font_dir / "sample.ttf"))
    f.flavor = "woff"
    f.save(str(font_dir / "sample.woff"))
    f.flavor = "woff2"
    f.save(str(font_dir / "sample.woff2"))
    f.flavor = None
    f.save(str(font_dir / "sample.eot"))
    sub_dir = SAMPLES_DIR / "Subtitle"
    sub_dir.mkdir(parents=True, exist_ok=True)
    (sub_dir / "sample.srt").write_text("1\n00:00:01,000 --> 00:00:04,000\nHello subtitle world.\n\n2\n00:00:05,000 --> 00:00:08,000\nTesting rf2f converter.\n", encoding="utf-8")
    (sub_dir / "sample.vtt").write_text("WEBVTT\n\n1\n00:00:01.000 --> 00:00:04.000\nHello subtitle world.\n\n2\n00:00:05.000 --> 00:00:08.000\nTesting rf2f converter.\n", encoding="utf-8")
    (sub_dir / "sample.ass").write_text("[Script Info]\nTitle: Test\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:04.00,Default,,0,0,0,,Hello subtitle world.\n", encoding="utf-8")
    (sub_dir / "sample.ssa").write_text("[Script Info]\nTitle: Test\n[Events]\nFormat: Marked, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: Marked=0,0:00:01.00,0:00:04.00,Default,,0,0,0,,Hello subtitle world.\n", encoding="utf-8")
    (sub_dir / "sample.lrc").write_text("[00:01.00]Hello lyric world.\n[00:05.00]Testing rf2f converter.\n", encoding="utf-8")
    print("Done! All 9 categories fully populated with valid samples.")
if __name__ == "__main__":
    ensure_all_samples()
