import os
import subprocess
import sys
from PIL import Image
def render_logo():
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    assets_dir = os.path.join(base_dir, "assets")
    svg_file = os.path.join(assets_dir, "logo.svg")
    with open(svg_file, "r", encoding="utf-8") as f:
        svg_content = f.read()
    html_content = f"""<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  html, body {{
    margin: 0;
    padding: 0;
    width: 512px;
    height: 512px;
    background: transparent;
    overflow: hidden;
  }}
  svg {{
    width: 512px;
    height: 512px;
    display: block;
  }}
</style>
</head>
<body>
{svg_content}
</body>
</html>"""
    render_html = os.path.join(assets_dir, "render.html")
    with open(render_html, "w", encoding="utf-8") as f:
        f.write(html_content)
    raw_png = os.path.join(assets_dir, "logo_raw.png")
    edge_paths = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    ]
    browser = None
    for bp in edge_paths:
        if os.path.exists(bp):
            browser = bp
            break
    if not browser:
        print("Error: No Chromium browser found.")
        sys.exit(1)
    render_uri = "file:///" + render_html.replace("\\", "/")
    cmd = [
        browser,
        "--headless=new",
        "--disable-gpu",
        f"--screenshot={raw_png}",
        "--window-size=512,512",
        "--default-background-color=00000000",
        render_uri,
    ]
    subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if os.path.exists(raw_png):
        img = Image.open(raw_png).convert("RGBA")
        logo_png = os.path.join(assets_dir, "logo.png")
        logo_512 = os.path.join(assets_dir, "logo_512.png")
        logo_ico = os.path.join(assets_dir, "logo.ico")
        img.save(logo_png, "PNG")
        img.save(logo_512, "PNG")
        ico_sizes = [(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (16, 16)]
        img.save(logo_ico, format="ICO", sizes=ico_sizes)
        os.remove(raw_png)
    if os.path.exists(render_html):
        os.remove(render_html)
if __name__ == "__main__":
    render_logo()
