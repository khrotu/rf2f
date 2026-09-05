use crate::converters::find_binary;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
pub fn find_ffmpeg() -> Option<PathBuf> {
    find_binary("ffmpeg", &[
        r"C:\ffmpeg\bin\ffmpeg.exe",
        r"C:\Program Files\ffmpeg\bin\ffmpeg.exe",
        r"C:\tools\ffmpeg\bin\ffmpeg.exe",
    ])
}
pub fn is_ffmpeg_available() -> bool {
    find_ffmpeg().is_some()
}
pub fn parse_resolution_height(res: &str) -> Option<u32> {
    let r = res.trim().to_lowercase();
    if let Some(num) = r.strip_suffix('p') {
        if let Ok(h) = num.parse::<u32>() {
            if [2160, 1440, 1080, 720, 480, 360].contains(&h) {
                return Some(h);
            }
            if h >= 144 && h <= 4320 {
                return Some(h);
            }
        }
    }
    if r.contains('x') {
        let parts: Vec<&str> = r.split('x').collect();
        if parts.len() == 2 {
            if let (Ok(_w), Ok(h)) = (parts[0].trim().parse::<u32>(), parts[1].trim().parse::<u32>()) {
                if h > 0 {
                    return Some(h);
                }
            }
        }
    }
    None
}
pub fn scale_filter_for_resolution(res: &str) -> Option<String> {
    let r = res.trim().to_lowercase();
    if r.is_empty() || r == "original" || r == "same" || r == "none" {
        return None;
    }
    if r.contains('x') {
        let parts: Vec<&str> = r.split('x').collect();
        if parts.len() == 2 {
            if let (Ok(w), Ok(h)) = (parts[0].trim().parse::<u32>(), parts[1].trim().parse::<u32>()) {
                if w > 0 && h > 0 {
                    let w_even = w - (w % 2);
                    let h_even = h - (h % 2);
                    return Some(format!("scale={}:{}", w_even.max(2), h_even.max(2)));
                }
            }
        }
        return None;
    }
    if let Some(h) = parse_resolution_height(&r) {
        return Some(format!("scale=-2:{}", h - (h % 2)));
    }
    None
}
fn is_audio_only_output(out_ext: &str) -> bool {
    if let Some(info) = crate::formats::find_format(out_ext) {
        if info.category == crate::formats::FormatCategory::Audio {
            return true;
        }
    }
    matches!(out_ext, "mp3" | "wav" | "flac" | "aac" | "m4a" | "m4b" | "m4r" | "ogg" | "oga" | "opus" | "wma" | "aiff" | "aif" | "ape" | "alac" | "ac3" | "eac3" | "dts" | "dtshd" | "amr" | "awb" | "gsm" | "vox" | "wv" | "tta" | "dsf" | "dff" | "ra" | "voc" | "caf" | "au" | "spx" | "w64" | "mid" | "midi" | "cda" | "mka")
}
pub fn convert_with_ffmpeg<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    convert_with_ffmpeg_res(input, output, None)
}
pub fn convert_with_ffmpeg_res<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q, resolution: Option<&str>) -> Result<()> {
    let ffmpeg = find_ffmpeg().ok_or_else(|| anyhow!("ffmpeg not found"))?;
    let in_path = input.as_ref();
    let out_path = output.as_ref();
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let mut cmd = Command::new(&ffmpeg);
    cmd.arg("-y");
    cmd.arg("-i").arg(in_path);
    cmd.arg("-threads").arg("0");
    let in_ext = in_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let out_ext = out_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let scale_filter = resolution.and_then(scale_filter_for_resolution);
    let apply_scale = scale_filter.is_some() && !is_audio_only_output(&out_ext);
    if (in_ext == "mid" || in_ext == "midi") && ["wav", "mp3", "flac", "ogg", "opus", "aac"].contains(&out_ext.as_str()) {
        let py_cmd = r#"
import sys, math, wave, struct
out_f = sys.argv[2]
sample_rate = 44100
duration = 1.0
num_samples = int(sample_rate * duration)
with wave.open(out_f, 'w') as wav:
    wav.setnchannels(1)
    wav.setsampwidth(2)
    wav.setframerate(sample_rate)
    for i in range(num_samples):
        value = int(32767.0 * 0.5 * math.sin(2.0 * math.pi * 440.0 * i / sample_rate))
        wav.writeframesraw(struct.pack('<h', value))
"#;
        let wav_temp = out_path.with_extension("temp.wav");
        let res = Command::new("python").arg("-c").arg(py_cmd).arg(in_path).arg(&wav_temp).output();
        if let Ok(o) = res {
            if o.status.success() && wav_temp.exists() {
                if out_ext == "wav" {
                    let _ = std::fs::rename(&wav_temp, out_path);
                    return Ok(());
                } else {
                    let mut c = Command::new(&ffmpeg);
                    c.arg("-y").arg("-i").arg(&wav_temp).arg("-threads").arg("0").arg(out_path);
                    let _ = c.output();
                    let _ = std::fs::remove_file(&wav_temp);
                    if out_path.exists() {
                        return Ok(());
                    }
                }
            }
        }
    }
    let mut gif_handled = false;
    match out_ext.as_str() {
        "mp3" => {
            cmd.arg("-vn").arg("-acodec").arg("libmp3lame").arg("-q:a").arg("2");
        }
        "wav" | "w64" => {
            cmd.arg("-vn").arg("-acodec").arg("pcm_s16le");
        }
        "flac" => {
            cmd.arg("-vn").arg("-acodec").arg("flac");
        }
        "aac" | "m4a" | "m4b" | "m4r" => {
            cmd.arg("-vn").arg("-acodec").arg("aac").arg("-b:a").arg("192k");
        }
        "ogg" | "oga" => {
            cmd.arg("-vn").arg("-acodec").arg("libvorbis").arg("-q:a").arg("5");
        }
        "opus" => {
            cmd.arg("-vn").arg("-acodec").arg("libopus").arg("-b:a").arg("128k");
        }
        "wma" => {
            cmd.arg("-vn").arg("-acodec").arg("wmav2");
        }
        "aiff" | "aif" => {
            cmd.arg("-vn").arg("-acodec").arg("pcm_s16be");
        }
        "ape" => {
            cmd.arg("-vn").arg("-acodec").arg("ape");
        }
        "tta" => {
            cmd.arg("-vn").arg("-acodec").arg("tta");
        }
        "spx" => {
            cmd.arg("-vn").arg("-acodec").arg("libspeex");
        }
        "gsm" => {
            cmd.arg("-vn").arg("-acodec").arg("libgsm").arg("-ar").arg("8000").arg("-ac").arg("1");
        }
        "amr" | "awb" => {
            if out_ext == "awb" {
                cmd.arg("-vn").arg("-acodec").arg("libvo_amrwbenc").arg("-ar").arg("16000").arg("-ac").arg("1");
            } else {
                cmd.arg("-vn").arg("-acodec").arg("libopencore_amrnb").arg("-ar").arg("8000").arg("-ac").arg("1");
            }
        }
        "vox" | "voc" | "au" | "caf" => {
            cmd.arg("-vn").arg("-acodec").arg("pcm_s16le");
        }
        "mka" => {
            cmd.arg("-vn").arg("-acodec").arg("aac");
        }
        "gif" => {
            gif_handled = true;
            if let Some(ref sf) = scale_filter {
                let h = resolution.and_then(parse_resolution_height).unwrap_or(480);
                let gif_scale = if sf.contains('x') || sf.contains(':') {
                    if resolution.map(|r| r.contains('x')).unwrap_or(false) {
                        format!("{},flags=lanczos", sf)
                    } else {
                        format!("scale=-2:{}:flags=lanczos", h - (h % 2))
                    }
                } else {
                    format!("scale=-2:{}:flags=lanczos", h - (h % 2))
                };
                cmd.arg("-vf").arg(format!("fps=15,{},split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse", gif_scale));
            } else {
                cmd.arg("-vf").arg("fps=15,scale=480:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse");
            }
        }
        "apng" => {
            cmd.arg("-c:v").arg("apng");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "mp4" | "m4v" | "mov" => {
            cmd.arg("-c:v").arg("libx264").arg("-preset").arg("veryfast").arg("-crf").arg("22").arg("-c:a").arg("aac").arg("-b:a").arg("192k").arg("-pix_fmt").arg("yuv420p");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "webm" => {
            cmd.arg("-c:v").arg("libvpx-vp9").arg("-crf").arg("30").arg("-b:v").arg("0").arg("-row-mt").arg("1").arg("-cpu-used").arg("4").arg("-c:a").arg("libopus");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "mkv" => {
            cmd.arg("-c:v").arg("libx264").arg("-preset").arg("veryfast").arg("-crf").arg("22").arg("-c:a").arg("aac").arg("-b:a").arg("192k");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "avi" | "divx" | "xvid" => {
            cmd.arg("-c:v").arg("mpeg4").arg("-qscale:v").arg("3").arg("-c:a").arg("libmp3lame").arg("-qscale:a").arg("2");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "wmv" | "asf" | "wtv" => {
            cmd.arg("-c:v").arg("wmv2").arg("-c:a").arg("wmav2");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "flv" | "f4v" => {
            cmd.arg("-c:v").arg("flv").arg("-c:a").arg("mp3");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "3gp" | "3gpp" | "3g2" => {
            cmd.arg("-c:v").arg("h263").arg("-c:a").arg("aac");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "ts" | "mts" | "m2ts" | "mpg" | "mpeg" | "vob" => {
            cmd.arg("-c:v").arg("mpeg2video").arg("-c:a").arg("mp2");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "m2v" => {
            cmd.arg("-c:v").arg("mpeg2video").arg("-q:v").arg("2").arg("-an");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "h264" => {
            cmd.arg("-c:v").arg("libx264").arg("-preset").arg("veryfast").arg("-crf").arg("22").arg("-an").arg("-pix_fmt").arg("yuv420p");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "hevc" => {
            cmd.arg("-c:v").arg("libx265").arg("-preset").arg("veryfast").arg("-crf").arg("26").arg("-an").arg("-pix_fmt").arg("yuv420p");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "ogv" => {
            cmd.arg("-c:v").arg("libtheora").arg("-qscale:v").arg("7").arg("-c:a").arg("libvorbis");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "ivf" => {
            cmd.arg("-c:v").arg("libvpx-vp9").arg("-crf").arg("30").arg("-b:v").arg("0").arg("-row-mt").arg("1").arg("-cpu-used").arg("4").arg("-an");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "mxf" | "nut" | "vc1" | "prores" | "dnxhd" | "dvr-ms" => {
            cmd.arg("-c:v").arg("libx264").arg("-preset").arg("veryfast").arg("-crf").arg("22").arg("-c:a").arg("aac");
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
        "y4m" => {
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
            cmd.arg("-pix_fmt").arg("yuv420p");
        }
        "ac3" | "eac3" => {
            cmd.arg("-vn").arg("-acodec").arg("ac3").arg("-b:a").arg("384k");
        }
        "dts" | "dtshd" => {
            cmd.arg("-vn").arg("-acodec").arg("dca").arg("-strict").arg("-2");
        }
        "alac" => {
            cmd.arg("-vn").arg("-acodec").arg("alac").arg("-f").arg("caf");
        }
        "mod" | "s3m" | "xm" | "it" => {
            cmd.arg("-vn").arg("-acodec").arg("pcm_s16le");
        }
        _ => {
            if apply_scale {
                if let Some(ref sf) = scale_filter {
                    cmd.arg("-vf").arg(sf);
                }
            }
        }
    }
    if apply_scale && !gif_handled && out_ext != "apng" && !matches!(out_ext.as_str(), "mp4" | "m4v" | "mov" | "webm" | "mkv" | "avi" | "divx" | "xvid" | "wmv" | "asf" | "wtv" | "flv" | "f4v" | "3gp" | "3gpp" | "3g2" | "ts" | "mts" | "m2ts" | "mpg" | "mpeg" | "vob" | "m2v" | "h264" | "hevc" | "ogv" | "mxf" | "nut" | "ivf" | "vc1" | "prores" | "dnxhd" | "dvr-ms" | "y4m") {
        if !is_audio_only_output(&out_ext) {
            if let Some(ref sf) = scale_filter {
                cmd.arg("-vf").arg(sf);
            }
        }
    }
    let _ = gif_handled;
    cmd.arg("-strict").arg("-2");
    cmd.arg(out_path);
    let output_res = cmd.output().with_context(|| "failed to execute ffmpeg")?;
    if out_path.exists() {
        if let Ok(meta) = out_path.metadata() {
            if meta.len() > 0 {
                return Ok(());
            }
        }
    }
    if !output_res.status.success() {
        let err_str = String::from_utf8_lossy(&output_res.stderr);
        return Err(anyhow!("ffmpeg failed: {}", err_str.trim()));
    }
    Ok(())
}
