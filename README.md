# rf2f

Rust-based file-to-file converter, used from the terminal or the Windows right-click menu.

## Convert between...

<details>
<summary>Archive (15)</summary>

zip, tar, gz, tgz, 7z, rar, bz2, xz, zst, lz4, iso, cab, tbz2, txz, tzst

</details>

<details>
<summary>Audio (41)</summary>

mp3, wav, flac, aac, m4a, m4b, m4r, ogg, oga, opus, wma, aiff, aif, ape, alac, ac3, eac3, dts, amr, mid, midi, wv, cda, dtshd, awb, gsm, vox, mod, s3m, xm, it, mka, tta, dsf, dff, ra, voc, caf, au, spx, w64

</details>

<details>
<summary>Data (24)</summary>

json, json5, jsonc, yaml, yml, toml, xml, csv, tsv, ron, plist, ini, env, ndjson, kdl, properties, psv, parquet, arrow, sqlite, db, msgpack, cbor, bson

</details>

<details>
<summary>Document (59)</summary>

pdf, docx, doc, xlsx, xls, pptx, ppt, odt, ods, odp, rtf, txt, md, markdown, html, htm, epub, mobi, azw3, pages, numbers, key, tex, latex, typ, dot, dotx, docm, ott, fodt, xlt, xltx, xlsm, xlsb, ots, fods, pot, potx, pptm, pps, ppsx, otp, fodp, wpd, wps, fb2, cbz, cbr, cbt, cb7, djvu, chm, rst, adoc, asciidoc, org, xhtml, mhtml, keynote

</details>

<details>
<summary>Font (8)</summary>

ttf, otf, woff, woff2, eot, dfont, pfa, pfb

</details>

<details>
<summary>Image (79)</summary>

png, jpg, jpeg, jfif, webp, avif, jxl, gif, bmp, ico, cur, tiff, tif, tga, qoi, svg, svgz, dds, psd, psb, xcf, heic, heif, hif, cr2, cr3, nef, arw, dng, raf, orf, rw2, exr, hdr, ppm, pgm, pbm, pnm, pam, pcx, icns, eps, ai, farbfeld, crw, pef, srw, srf, mrw, dcr, kdc, erf, mos, nrw, mef, x3f, 3fr, rwl, cdr, kra, clip, emf, wmf, pic, fits, ktx, ktx2, pvr, astc, vtf, sgi, rgb, rgba, bw, ras, sun, xbm, xpm, wbmp

</details>

<details>
<summary>Model3D (20)</summary>

obj, stl, ply, gltf, glb, fbx, 3mf, usdz, off, dae, 3ds, blend, x3d, step, stp, iges, igs, dxf, usda, usdc

</details>

<details>
<summary>Subtitle (9)</summary>

srt, vtt, ass, ssa, lrc, sub, sbv, ttml, smi

</details>

<details>
<summary>Video (39)</summary>

mp4, mkv, webm, avi, mov, wmv, flv, m4v, 3gp, 3gpp, 3g2, ts, mts, m2ts, vob, ogv, mpg, mpeg, m2v, bik, bik2, rm, rmvb, f4v, asf, y4m, wtv, dvr-ms, divx, xvid, mxf, nut, ivf, h264, hevc, vc1, apng, prores, dnxhd

</details>

## Requirements

rf2f uses the following external software:

- ffmpeg
- ImageMagick
- LibreOffice
- pandoc
- typst
- Edge/Chrome

Use `rf2f doctor` to check which are installed, and `rf2f doctor --install` to install the missing ones.

## Installation

Run in PowerShell:

```powershell
irm https://raw.githubusercontent.com/khrotu/rf2f/main/install.ps1 | iex
```

Or build from source:

```
cargo build --release
```

Register the context menu and `PATH` entry:

```
rf2f register
```

Remove them:

```
rf2f unregister
```

On Windows 11, third-party entries are listed in "Show more options". Use rf2f from there, or follow [this guide](https://www.guidingtech.com/top-ways-to-disable-the-show-more-options-context-menu-in-windows-11/) to disable it.

## Usage

### Context menu

`Convert with rf2f`: Obvious enough.
`Scale with rf2f`: Shows when an image/video file is selected, with `2160p/1080p/720p/480p` options. Other resolutions can be specified through the CLI.

### CLI

Convert files:

```
rf2f convert input.png --format jpg
rf2f convert input.mp4 --format mkv --resolution 720p
rf2f convert ./photos --format webp --output ./out --recursive
```

Shorthand without the subcommand also works:

```
rf2f input.png --format jpg
```

`--output` accepts a directory or a file path.
`--resolution` accepts `2160p`, `1080p`, `720p`, `480p`, or some `WxH` combo (e.g. `1280x720`).

List formats:

```
rf2f formats
rf2f formats --category video
rf2f formats --ext mp4
```
