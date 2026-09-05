use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
pub fn is_native_subtitle_supported(in_ext: &str, out_ext: &str) -> bool {
    let in_clean = in_ext.trim_start_matches('.').to_lowercase();
    let out_clean = out_ext.trim_start_matches('.').to_lowercase();
    let supported = ["srt", "vtt", "ass", "ssa", "sub", "sbv", "lrc", "ttml", "smi", "txt"];
    supported.contains(&in_clean.as_str()) && supported.contains(&out_clean.as_str())
}
#[derive(Debug, Clone)]
pub struct SubtitleItem {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}
fn parse_time_srt(s: &str) -> Option<u64> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let h: u64 = parts[0].trim().parse().ok()?;
    let m: u64 = parts[1].trim().parse().ok()?;
    let sec_parts: Vec<&str> = parts[2].split(',').collect();
    if sec_parts.is_empty() {
        return None;
    }
    let sec: u64 = sec_parts[0].trim().parse().ok()?;
    let ms: u64 = if sec_parts.len() > 1 {
        let ms_str = format!("{:0<3}", sec_parts[1].trim());
        ms_str[..3.min(ms_str.len())].parse().unwrap_or(0)
    } else {
        0
    };
    Some(h * 3600000 + m * 60000 + sec * 1000 + ms)
}
fn parse_time_vtt(s: &str) -> Option<u64> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() == 3 {
        let h: u64 = parts[0].trim().parse().ok()?;
        let m: u64 = parts[1].trim().parse().ok()?;
        let sec_parts: Vec<&str> = parts[2].split('.').collect();
        let sec: u64 = sec_parts.first()?.trim().parse().ok()?;
        let ms: u64 = if sec_parts.len() > 1 {
            let ms_str = format!("{:0<3}", sec_parts[1].trim());
            ms_str[..3.min(ms_str.len())].parse().unwrap_or(0)
        } else {
            0
        };
        Some(h * 3600000 + m * 60000 + sec * 1000 + ms)
    } else if parts.len() == 2 {
        let m: u64 = parts[0].trim().parse().ok()?;
        let sec_parts: Vec<&str> = parts[1].split('.').collect();
        let sec: u64 = sec_parts.first()?.trim().parse().ok()?;
        let ms: u64 = if sec_parts.len() > 1 {
            let ms_str = format!("{:0<3}", sec_parts[1].trim());
            ms_str[..3.min(ms_str.len())].parse().unwrap_or(0)
        } else {
            0
        };
        Some(m * 60000 + sec * 1000 + ms)
    } else {
        None
    }
}
fn parse_time_ass(s: &str) -> Option<u64> {
    let s = s.trim();
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let h: u64 = parts[0].trim().parse().ok()?;
    let m: u64 = parts[1].trim().parse().ok()?;
    let sec_parts: Vec<&str> = parts[2].split('.').collect();
    let sec: u64 = sec_parts.first()?.trim().parse().ok()?;
    let cs: u64 = if sec_parts.len() > 1 {
        let cs_str = format!("{:0<2}", sec_parts[1].trim());
        cs_str[..2.min(cs_str.len())].parse().unwrap_or(0)
    } else {
        0
    };
    Some(h * 3600000 + m * 60000 + sec * 1000 + cs * 10)
}
fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            out.push(c);
        }
    }
    let mut res = out.replace("{\\an8}", "").replace("\\N", "\n").replace("\\n", "\n").replace("\\h", " ");
    loop {
        if let Some(start) = res.find('{') {
            if let Some(end) = res[start..].find('}') {
                res.replace_range(start..start + end + 1, "");
            } else {
                break;
            }
        } else {
            break;
        }
    }
    res.trim().to_string()
}
fn format_time_srt(ms: u64) -> String {
    let h = ms / 3600000;
    let rem = ms % 3600000;
    let m = rem / 60000;
    let rem2 = rem % 60000;
    let s = rem2 / 1000;
    let millis = rem2 % 1000;
    return format!("{:02}:{:02}:{:02},{:03}", h, m, s, millis);
}
fn format_time_vtt(ms: u64) -> String {
    let h = ms / 3600000;
    let rem = ms % 3600000;
    let m = rem / 60000;
    let rem2 = rem % 60000;
    let s = rem2 / 1000;
    let millis = rem2 % 1000;
    return format!("{:02}:{:02}:{:02}.{:03}", h, m, s, millis);
}
fn format_time_lrc(ms: u64) -> String {
    let m = ms / 60000;
    let rem = ms % 60000;
    let s = rem / 1000;
    let cs = (rem % 1000) / 10;
    return format!("{:02}:{:02}.{:02}", m, s, cs);
}
fn format_time_ass(ms: u64) -> String {
    let h = ms / 3600000;
    let rem = ms % 3600000;
    let m = rem / 60000;
    let rem2 = rem % 60000;
    let s = rem2 / 1000;
    let cs = (rem2 % 1000) / 10;
    return format!("{}:{:02}:{:02}.{:02}", h, m, s, cs);
}
fn format_time_sbv(ms: u64) -> String {
    let h = ms / 3600000;
    let rem = ms % 3600000;
    let m = rem / 60000;
    let rem2 = rem % 60000;
    let s = rem2 / 1000;
    let millis = rem2 % 1000;
    return format!("{}:{:02}:{:02}.{:03}", h, m, s, millis);
}
pub fn parse_subtitles(content: &str, ext: &str) -> Vec<SubtitleItem> {
    let mut items = Vec::new();
    match ext {
        "srt" => {
            let blocks = content.replace("\r\n", "\n").split("\n\n").map(|s| s.to_string()).collect::<Vec<_>>();
            for block in blocks {
                let lines: Vec<&str> = block.lines().filter(|l| !l.trim().is_empty()).collect();
                if lines.len() >= 2 {
                    let time_line_idx = if lines[0].parse::<u32>().is_ok() { 1 } else { 0 };
                    if time_line_idx < lines.len() {
                        let time_line = lines[time_line_idx];
                        if let Some((start_s, end_s)) = time_line.split_once("-->") {
                            if let (Some(start), Some(end)) = (parse_time_srt(start_s).or_else(|| parse_time_vtt(start_s)), parse_time_srt(end_s).or_else(|| parse_time_vtt(end_s))) {
                                let text = lines[(time_line_idx + 1)..].join("\n");
                                if !text.trim().is_empty() {
                                    items.push(SubtitleItem { start_ms: start, end_ms: end, text: strip_tags(&text) });
                                }
                            }
                        }
                    }
                }
            }
        }
        "vtt" => {
            let blocks = content.replace("\r\n", "\n").split("\n\n").map(|s| s.to_string()).collect::<Vec<_>>();
            for block in blocks {
                let lines: Vec<&str> = block.lines().filter(|l| !l.trim().is_empty() && !l.starts_with("WEBVTT") && !l.starts_with("NOTE")).collect();
                for (idx, line) in lines.iter().enumerate() {
                    if line.contains("-->") {
                        if let Some((start_s, end_s)) = line.split_once("-->") {
                            let end_token = end_s.trim().split_whitespace().next().unwrap_or(end_s.trim());
                            if let (Some(start), Some(end)) = (parse_time_vtt(start_s), parse_time_vtt(end_token)) {
                                let text = lines[(idx + 1)..].join("\n");
                                if !text.trim().is_empty() {
                                    items.push(SubtitleItem { start_ms: start, end_ms: end, text: strip_tags(&text) });
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
        "ass" | "ssa" => {
            for line in content.lines() {
                let t = line.trim();
                if t.starts_with("Dialogue:") {
                    let payload = t.trim_start_matches("Dialogue:").trim();
                    let parts: Vec<&str> = payload.splitn(10, ',').collect();
                    if parts.len() == 10 {
                        if let (Some(start), Some(end)) = (parse_time_ass(parts[1]), parse_time_ass(parts[2])) {
                            let text = strip_tags(parts[9]);
                            if !text.is_empty() {
                                items.push(SubtitleItem { start_ms: start, end_ms: end, text });
                            }
                        }
                    }
                }
            }
        }
        "sbv" => {
            let blocks = content.replace("\r\n", "\n").split("\n\n").map(|s| s.to_string()).collect::<Vec<_>>();
            for block in blocks {
                let lines: Vec<&str> = block.lines().filter(|l| !l.trim().is_empty()).collect();
                if lines.is_empty() {
                    continue;
                }
                if lines[0].contains(',') {
                    if let Some((start_s, end_s)) = lines[0].split_once(',') {
                        if let (Some(start), Some(end)) = (parse_time_vtt(start_s), parse_time_vtt(end_s)) {
                            let text = lines[1..].join("\n");
                            if !text.trim().is_empty() {
                                items.push(SubtitleItem { start_ms: start, end_ms: end, text: strip_tags(&text) });
                            }
                        }
                    }
                }
            }
        }
        "sub" => {
            for line in content.lines() {
                let t = line.trim();
                if t.starts_with('{') {
                    if let Some(first_close) = t.find('}') {
                        let rest = &t[first_close + 1..];
                        if let Some(second_close) = rest.find('}') {
                            let start_frame: u64 = t[1..first_close].parse().unwrap_or(0);
                            let end_frame: u64 = rest[1..second_close].parse().unwrap_or(0);
                            let text = rest[second_close + 1..].replace('|', "\n");
                            let text = strip_tags(&text);
                            if !text.trim().is_empty() {
                                let start_ms = ((start_frame as f64) * 1000.0 / 23.976) as u64;
                                let end_ms = ((end_frame as f64) * 1000.0 / 23.976) as u64;
                                items.push(SubtitleItem { start_ms, end_ms, text });
                            }
                        }
                    }
                }
            }
        }
        "ttml" => {
            let mut pos = 0;
            let bytes = content.as_bytes();
            while let Some(p_start) = content[pos..].find("<p") {
                let abs_start = pos + p_start;
                if let Some(tag_end) = content[abs_start..].find('>') {
                    let tag = &content[abs_start..abs_start + tag_end];
                    let mut begin_ms: Option<u64> = None;
                    let mut end_ms: Option<u64> = None;
                    for attr in ["begin", "start"] {
                        if let Some(a) = tag.find(attr) {
                            let sub = &tag[a..];
                            if let Some(q1) = sub.find('"') {
                                if let Some(q2) = sub[q1 + 1..].find('"') {
                                    let val = &sub[q1 + 1..q1 + 1 + q2];
                                    begin_ms = parse_time_vtt(val).or_else(|| parse_time_srt(val));
                                    break;
                                }
                            }
                        }
                    }
                    for attr in ["end", "dur"] {
                        if let Some(a) = tag.find(attr) {
                            let sub = &tag[a..];
                            if let Some(q1) = sub.find('"') {
                                if let Some(q2) = sub[q1 + 1..].find('"') {
                                    let val = &sub[q1 + 1..q1 + 1 + q2];
                                    end_ms = parse_time_vtt(val).or_else(|| parse_time_srt(val));
                                    break;
                                }
                            }
                        }
                    }
                    let content_start = abs_start + tag_end + 1;
                    if let Some(p_end) = content[content_start..].find("</p>") {
                        let raw_text = &content[content_start..content_start + p_end];
                        let text = strip_tags(raw_text);
                        if let (Some(s), Some(e)) = (begin_ms, end_ms) {
                            if !text.trim().is_empty() {
                                items.push(SubtitleItem { start_ms: s, end_ms: e, text });
                            }
                        } else if let Some(s) = begin_ms {
                            if !text.trim().is_empty() {
                                items.push(SubtitleItem { start_ms: s, end_ms: s + 3000, text });
                            }
                        }
                        pos = content_start + p_end + 4;
                        let _ = bytes;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        "smi" => {
            let lower = content.to_lowercase();
            let mut syncs: Vec<(u64, String)> = Vec::new();
            let mut search_from = 0;
            while let Some(idx) = lower[search_from..].find("<sync") {
                let abs_idx = search_from + idx;
                let tag_end = lower[abs_idx..].find('>').map(|i| abs_idx + i).unwrap_or(abs_idx);
                let tag = &content[abs_idx..=tag_end.min(content.len() - 1)];
                let mut start_ms: Option<u64> = None;
                if let Some(si) = tag.to_lowercase().find("start") {
                    let sub = &tag[si..];
                    if let Some(eq) = sub.find('=') {
                        let after = sub[eq + 1..].trim().trim_start_matches('=').trim();
                        let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                        if let Ok(v) = num_str.parse::<u64>() {
                            start_ms = Some(v);
                        }
                    }
                }
                let next_sync = lower[tag_end..].find("<sync").map(|i| tag_end + i).unwrap_or(content.len());
                let body = &content[tag_end + 1..next_sync.min(content.len())];
                let text = strip_tags(body);
                if let Some(s) = start_ms {
                    if !text.trim().is_empty() && text.to_lowercase() != "&nbsp;" {
                        syncs.push((s, text));
                    }
                }
                search_from = next_sync;
                if search_from >= content.len() {
                    break;
                }
            }
            for (i, (s, t)) in syncs.iter().enumerate() {
                let e = if i + 1 < syncs.len() { syncs[i + 1].0 } else { s + 3000 };
                items.push(SubtitleItem { start_ms: *s, end_ms: e.max(*s + 500), text: t.clone() });
            }
        }
        "lrc" => {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with('[') {
                    let mut rest = line;
                    let mut times = Vec::new();
                    let mut text_part = String::new();
                    while rest.starts_with('[') {
                        if let Some(close_idx) = rest.find(']') {
                            let time_str = &rest[1..close_idx];
                            if let Some(ms) = parse_time_vtt(time_str).or_else(|| parse_time_ass(time_str)) {
                                times.push(ms);
                            }
                            rest = rest[close_idx + 1..].trim();
                            text_part = rest.to_string();
                        } else {
                            break;
                        }
                    }
                    let text = strip_tags(&text_part);
                    if !text.is_empty() {
                        for ms in times {
                            items.push(SubtitleItem { start_ms: ms, end_ms: ms + 3000, text: text.clone() });
                        }
                    }
                }
            }
            items.sort_by_key(|i| i.start_ms);
        }
        _ => {
            for (idx, line) in content.lines().enumerate() {
                let text = strip_tags(line).trim().to_string();
                if !text.is_empty() {
                    let ms = (idx as u64) * 3000;
                    items.push(SubtitleItem { start_ms: ms, end_ms: ms + 3000, text });
                }
            }
        }
    }
    items
}
pub fn convert_subtitles<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    let in_path = input.as_ref();
    let out_path = output.as_ref();
    let in_ext = in_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let out_ext = out_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let content = fs::read_to_string(in_path)?;
    let mut items = parse_subtitles(&content, &in_ext);
    if items.is_empty() {
        let fallback_text = strip_tags(&content);
        if !fallback_text.trim().is_empty() {
            for (idx, line) in fallback_text.lines().enumerate() {
                let t = line.trim();
                if !t.is_empty() {
                    let ms = (idx as u64) * 3000;
                    items.push(SubtitleItem { start_ms: ms, end_ms: ms + 3000, text: t.to_string() });
                }
            }
        }
        if items.is_empty() {
            return Err(anyhow!("no subtitles found in {:?}", in_path));
        }
    }
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    match out_ext.as_str() {
        "srt" => {
            let mut out = String::new();
            for (idx, item) in items.iter().enumerate() {
                out.push_str(&format!("{}\n{} --> {}\n{}\n\n", idx + 1, format_time_srt(item.start_ms), format_time_srt(item.end_ms), item.text));
            }
            fs::write(out_path, out.trim_end())?;
        }
        "vtt" => {
            let mut out = String::from("WEBVTT\n\n");
            for item in items.iter() {
                out.push_str(&format!("{} --> {}\n{}\n\n", format_time_vtt(item.start_ms), format_time_vtt(item.end_ms), item.text));
            }
            fs::write(out_path, out.trim_end())?;
        }
        "ass" => {
            let mut out = String::from("[Script Info]\nTitle: rf2f\nScriptType: v4.00+\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
            for item in &items {
                out.push_str(&format!("Dialogue: 0,{}, {},Default,,0,0,0,,{}\n", format_time_ass(item.start_ms), format_time_ass(item.end_ms), item.text.replace('\n', "\\N")));
            }
            fs::write(out_path, out)?;
        }
        "ssa" => {
            let mut out = String::from("[Script Info]\nTitle: rf2f\nScriptType: v4.00\n\n[V4 Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, TertiaryColour, BackColour, Bold, Italic, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, AlphaLevel, Encoding\nStyle: Default,Arial,20,16777215,65535,0,0,0,0,1,2,2,2,10,10,10,0,0\n\n[Events]\nFormat: Marked, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
            for item in &items {
                out.push_str(&format!("Dialogue: Marked=0,{},{},Default,,0,0,0,,{}\n", format_time_ass(item.start_ms), format_time_ass(item.end_ms), item.text.replace('\n', "\\N")));
            }
            fs::write(out_path, out)?;
        }
        "sub" => {
            let mut out = String::new();
            for item in &items {
                let sf = ((item.start_ms as f64) * 23.976 / 1000.0) as u64;
                let ef = ((item.end_ms as f64) * 23.976 / 1000.0) as u64;
                out.push_str(&format!("{{{}}}{}{}\n", sf, ef, item.text.replace('\n', "|")));
            }
            fs::write(out_path, out)?;
        }
        "sbv" => {
            let mut out = String::new();
            for item in &items {
                out.push_str(&format!("{},{}\n{}\n\n", format_time_sbv(item.start_ms), format_time_sbv(item.end_ms), item.text));
            }
            fs::write(out_path, out.trim_end())?;
        }
        "ttml" => {
            let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<tt xmlns=\"http://www.w3.org/ns/ttml\"><body><div>\n");
            for item in &items {
                out.push_str(&format!("  <p begin=\"{}\" end=\"{}\">{}</p>\n", format_time_vtt(item.start_ms), format_time_vtt(item.end_ms), item.text.replace('\n', "<br/>")));
            }
            out.push_str("</div></body></tt>");
            fs::write(out_path, out)?;
        }
        "smi" => {
            let mut out = String::from("<SAMI><HEAD><TITLE>rf2f</TITLE></HEAD><BODY>\n");
            for item in &items {
                out.push_str(&format!("<SYNC Start={}><P Class=ENCC>{}<SYNC Start={}><P Class=ENCC>&nbsp;\n", item.start_ms, item.text.replace('\n', "<br>"), item.end_ms));
            }
            out.push_str("</BODY></SAMI>");
            fs::write(out_path, out)?;
        }
        "lrc" => {
            let mut out = String::new();
            for item in &items {
                out.push_str(&format!("[{}]{}\n", format_time_lrc(item.start_ms), item.text.replace('\n', " ")));
            }
            fs::write(out_path, out)?;
        }
        "txt" => {
            let out = items.iter().map(|it| it.text.as_str()).collect::<Vec<_>>().join("\n");
            fs::write(out_path, out)?;
        }
        _ => return Err(anyhow!("unsupported subtitle output: {}", out_ext)),
    }
    Ok(())
}
