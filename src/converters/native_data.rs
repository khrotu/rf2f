use anyhow::{anyhow, Context, Result};
use serde_json::Value as JsonValue;
use std::fs;
use std::io::Write;
use std::path::Path;
pub fn is_native_data_supported(in_ext: &str, out_ext: &str) -> bool {
    let in_clean = in_ext.trim_start_matches('.').to_lowercase();
    let out_clean = out_ext.trim_start_matches('.').to_lowercase();
    let supported = ["json", "json5", "jsonc", "yaml", "yml", "toml", "csv", "tsv", "psv", "ini", "env", "properties", "plist", "ron", "xml", "html", "htm", "ndjson", "kdl", "parquet", "arrow", "sqlite", "db", "msgpack", "cbor", "bson"];
    supported.contains(&in_clean.as_str()) && supported.contains(&out_clean.as_str())
}
fn strip_json_comments(s: &str) -> String {
    let mut res = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut in_str: Option<char> = None;
    while i < n {
        let c = chars[i];
        let nxt = if i + 1 < n { chars[i + 1] } else { '\0' };
        if let Some(q) = in_str {
            res.push(c);
            if c == '\\' && i + 1 < n {
                res.push(chars[i + 1]);
                i += 2;
                continue;
            } else if c == q {
                in_str = None;
            }
            i += 1;
        } else {
            if c == '"' || c == '\'' {
                in_str = Some(c);
                res.push('"');
                i += 1;
            } else if c == '/' && nxt == '/' {
                while i < n && chars[i] != '\n' {
                    i += 1;
                }
            } else if c == '/' && nxt == '*' {
                i += 2;
                while i + 1 < n && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
            } else {
                res.push(c);
                i += 1;
            }
        }
    }
    let without_comments: String = res.into_iter().collect();
    without_comments.replace(",\n}", "\n}").replace(",\r\n}", "\r\n}").replace(",}", "}").replace(",\n]", "\n]").replace(",\r\n]", "\r\n]").replace(",]", "]")
}
pub fn convert_data<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    let in_path = input.as_ref();
    let out_path = output.as_ref();
    let in_ext = in_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let out_ext = out_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let is_binary_input = matches!(in_ext.as_str(), "parquet" | "arrow" | "sqlite" | "db" | "msgpack" | "cbor" | "bson");
    let raw_content = if is_binary_input {
        String::new()
    } else {
        fs::read_to_string(in_path).with_context(|| format!("read failed: {:?}", in_path))?
    };
    let val: JsonValue = match in_ext.as_str() {
        "json" => serde_json::from_str(&raw_content).with_context(|| "json parse failed")?,
        "json5" | "jsonc" => {
            let py_cmd = r#"
import sys, json, yaml
try:
    with open(sys.argv[1], 'r', encoding='utf-8') as f:
        data = yaml.safe_load(f)
    print(json.dumps(data))
except Exception:
    pass
"#;
            let res = std::process::Command::new("python").arg("-c").arg(py_cmd).arg(in_path).output();
            if let Ok(o) = res {
                if o.status.success() {
                    let s = String::from_utf8_lossy(&o.stdout);
                    if let Ok(v) = serde_json::from_str(&s) {
                        v
                    } else {
                        let cleaned = strip_json_comments(&raw_content);
                        serde_json::from_str(&cleaned).with_context(|| "json parse failed")?
                    }
                } else {
                    let cleaned = strip_json_comments(&raw_content);
                    serde_json::from_str(&cleaned).with_context(|| "json parse failed")?
                }
            } else {
                let cleaned = strip_json_comments(&raw_content);
                serde_json::from_str(&cleaned).with_context(|| "json parse failed")?
            }
        }
        "yaml" | "yml" => serde_yaml::from_str(&raw_content).with_context(|| "yaml parse failed")?,
        "toml" => {
            let toml_val: toml::Value = toml::from_str(&raw_content).with_context(|| "toml parse failed")?;
            serde_json::to_value(toml_val)?
        }
        "csv" | "tsv" => {
            let delimiter = if in_ext == "tsv" { b'\t' } else { b',' };
            let mut rdr = csv::ReaderBuilder::new().delimiter(delimiter).from_reader(raw_content.as_bytes());
            let headers = rdr.headers()?.clone();
            let mut records = Vec::new();
            for result in rdr.records() {
                let record = result?;
                let mut map = serde_json::Map::new();
                for (h, field) in headers.iter().zip(record.iter()) {
                    if let Ok(num) = field.parse::<i64>() {
                        map.insert(h.to_string(), JsonValue::Number(num.into()));
                    } else if let Ok(f) = field.parse::<f64>() {
                        if let Some(n) = serde_json::Number::from_f64(f) {
                            map.insert(h.to_string(), JsonValue::Number(n));
                        } else {
                            map.insert(h.to_string(), JsonValue::String(field.to_string()));
                        }
                    } else if field.eq_ignore_ascii_case("true") {
                        map.insert(h.to_string(), JsonValue::Bool(true));
                    } else if field.eq_ignore_ascii_case("false") {
                        map.insert(h.to_string(), JsonValue::Bool(false));
                    } else {
                        map.insert(h.to_string(), JsonValue::String(field.to_string()));
                    }
                }
                records.push(JsonValue::Object(map));
            }
            JsonValue::Array(records)
        }
        "ini" | "env" => {
            let mut map = serde_json::Map::new();
            for line in raw_content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once('=') {
                    let k = k.trim().to_string();
                    let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
                    map.insert(k, JsonValue::String(v));
                }
            }
            JsonValue::Object(map)
        }
        "plist" => {
            let py_cmd = r#"
import sys, json, plistlib
with open(sys.argv[1], 'rb') as f:
    pl = plistlib.load(f)
print(json.dumps(pl))
"#;
            let output = std::process::Command::new("python").arg("-c").arg(py_cmd).arg(in_path).output();
            if let Ok(res) = output {
                if res.status.success() {
                    let out_str = String::from_utf8_lossy(&res.stdout);
                    serde_json::from_str(&out_str).with_context(|| "plist parse failed")?
                } else {
                    return Err(anyhow!("plist parse failed"));
                }
            } else {
                return Err(anyhow!("plist parse failed"));
            }
        }
        "xml" => {
            let py_cmd = r#"
import sys, json, xml.etree.ElementTree as ET
def etree_to_dict(t):
    children = list(t)
    if not children and not t.attrib:
        return t.text.strip() if t.text else ""
    d = {}
    if t.attrib:
        d.update(('@' + k, v) for k, v in t.attrib.items())
    if children:
        for child in children:
            cd = etree_to_dict(child)
            if child.tag in d:
                if not isinstance(d[child.tag], list):
                    d[child.tag] = [d[child.tag]]
                d[child.tag].append(cd)
            else:
                d[child.tag] = cd
    if t.text and t.text.strip():
        d['#text'] = t.text.strip()
    return d
tree = ET.parse(sys.argv[1])
root = tree.getroot()
print(json.dumps({root.tag: etree_to_dict(root)}))
"#;
            let output = std::process::Command::new("python").arg("-c").arg(py_cmd).arg(in_path).output();
            if let Ok(res) = output {
                if res.status.success() {
                    let out_str = String::from_utf8_lossy(&res.stdout);
                    serde_json::from_str(&out_str).with_context(|| "xml parse failed")?
                } else {
                    return Err(anyhow!("xml parse failed"));
                }
            } else {
                return Err(anyhow!("xml parse failed"));
            }
        }
        "ron" => {
            let py_cmd = r#"
import sys, json, re, yaml
with open(sys.argv[1], 'r', encoding='utf-8') as f:
    text = f.read()
inner = re.sub(r'^\s*\(', '', text.strip())
inner = re.sub(r'\)\s*$', '', inner.strip())
json_like = '{' + re.sub(r',\s*$', '', inner.strip()) + '}'
data = yaml.safe_load(json_like)
print(json.dumps(data))
"#;
            let res = std::process::Command::new("python").arg("-c").arg(py_cmd).arg(in_path).output();
            if let Ok(o) = res {
                if o.status.success() {
                    let s = String::from_utf8_lossy(&o.stdout);
                    serde_json::from_str(&s).with_context(|| "ron parse failed")?
                } else {
                    return Err(anyhow!("ron parse failed"));
                }
            } else {
                return Err(anyhow!("ron parse failed"));
            }
        }
        "ndjson" => {
            let mut records = Vec::new();
            for line in raw_content.lines() {
                let t = line.trim();
                if t.is_empty() {
                    continue;
                }
                let v: JsonValue = serde_json::from_str(t).with_context(|| "ndjson parse failed")?;
                records.push(v);
            }
            JsonValue::Array(records)
        }
        "psv" => {
            let mut rdr = csv::ReaderBuilder::new().delimiter(b'|').from_reader(raw_content.as_bytes());
            let headers = rdr.headers()?.clone();
            let mut records = Vec::new();
            for result in rdr.records() {
                let record = result?;
                let mut map = serde_json::Map::new();
                for (h, field) in headers.iter().zip(record.iter()) {
                    map.insert(h.to_string(), JsonValue::String(field.to_string()));
                }
                records.push(JsonValue::Object(map));
            }
            JsonValue::Array(records)
        }
        "properties" => {
            let mut map = serde_json::Map::new();
            for line in raw_content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once('=').or_else(|| trimmed.split_once(':')) {
                    map.insert(k.trim().to_string(), JsonValue::String(v.trim().to_string()));
                }
            }
            JsonValue::Object(map)
        }
        "kdl" => {
            let py_cmd = r#"
import sys, json
try:
    import kdl
    with open(sys.argv[1], 'r', encoding='utf-8') as f:
        text = f.read()
    doc = kdl.parse(text)
    def node_to_json(n):
        d = {}
        if n.args:
            d['args'] = [str(a) for a in n.args]
        if n.props:
            d['props'] = {k: str(v) for k, v in n.props.items()}
        if n.children:
            d['children'] = [node_to_json(c) for c in n.children]
        return {n.name: d if d else True}
    out = [node_to_json(n) for n in doc.nodes]
    print(json.dumps(out))
except Exception as e:
    import yaml
    with open(sys.argv[1], 'r', encoding='utf-8') as f:
        data = yaml.safe_load(f)
    print(json.dumps(data))
"#;
            let res = std::process::Command::new("python").arg("-c").arg(py_cmd).arg(in_path).output();
            if let Ok(o) = res {
                if o.status.success() {
                    let s = String::from_utf8_lossy(&o.stdout);
                    if let Ok(v) = serde_json::from_str(&s) {
                        v
                    } else {
                        serde_yaml::from_str(&raw_content).with_context(|| "kdl parse failed")?
                    }
                } else {
                    serde_yaml::from_str(&raw_content).with_context(|| "kdl parse failed")?
                }
            } else {
                serde_yaml::from_str(&raw_content).with_context(|| "kdl parse failed")?
            }
        }
        "parquet" | "arrow" | "sqlite" | "db" | "msgpack" | "cbor" | "bson" => {
            let py_cmd = r#"
import sys, json
in_f = sys.argv[1]
in_ext = sys.argv[2].lower()
try:
    data = None
    if in_ext == 'parquet':
        import pyarrow.parquet as pq
        table = pq.read_table(in_f)
        data = table.to_pylist()
    elif in_ext == 'arrow':
        import pyarrow.ipc as ipc
        import pyarrow as pa
        try:
            with open(in_f, 'rb') as f:
                reader = ipc.open_file(f)
                table = reader.read_all()
                data = table.to_pylist()
        except Exception:
            with open(in_f, 'rb') as f:
                reader = ipc.open_stream(f)
                table = reader.read_all()
                data = table.to_pylist()
    elif in_ext in ('sqlite', 'db'):
        import sqlite3
        con = sqlite3.connect(in_f)
        cur = con.cursor()
        cur.execute("SELECT name FROM sqlite_master WHERE type='table'")
        tables = [r[0] for r in cur.fetchall()]
        if tables:
            t = tables[0]
            cur.execute(f'SELECT * FROM "{t}"')
            cols = [d[0] for d in cur.description]
            rows = cur.fetchall()
            data = [dict(zip(cols, r)) for r in rows]
        else:
            data = []
        con.close()
    elif in_ext == 'msgpack':
        import msgpack
        with open(in_f, 'rb') as f:
            data = msgpack.unpackb(f.read(), raw=False)
    elif in_ext == 'cbor':
        import cbor2
        with open(in_f, 'rb') as f:
            data = cbor2.load(f)
    elif in_ext == 'bson':
        import bson
        with open(in_f, 'rb') as f:
            raw = f.read()
            docs = bson.decode_all(raw)
            data = docs[0] if len(docs) == 1 else docs
    print(json.dumps(data, default=str))
except Exception as e:
    sys.stderr.write(str(e))
    sys.exit(1)
"#;
            let output = std::process::Command::new("python").arg("-c").arg(py_cmd).arg(in_path).arg(&in_ext).output();
            if let Ok(res) = output {
                if res.status.success() {
                    let out_str = String::from_utf8_lossy(&res.stdout);
                    serde_json::from_str(&out_str).with_context(|| format!("{} parse failed", in_ext))?
                } else {
                    return Err(anyhow!("{} parse failed: {}", in_ext, String::from_utf8_lossy(&res.stderr).trim()));
                }
            } else {
                return Err(anyhow!("{} parse failed", in_ext));
            }
        }
        _ => return Err(anyhow!("unsupported data input: {}", in_ext)),
    };
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    match out_ext.as_str() {
        "json" | "json5" | "jsonc" => {
            let formatted = serde_json::to_string_pretty(&val)?;
            fs::write(out_path, formatted)?;
        }
        "yaml" | "yml" => {
            let formatted = serde_yaml::to_string(&val)?;
            fs::write(out_path, formatted)?;
        }
        "toml" => {
            let toml_obj = match &val {
                JsonValue::Object(_) => val.clone(),
                JsonValue::Array(_) => {
                    let mut m = serde_json::Map::new();
                    m.insert("data".to_string(), val.clone());
                    JsonValue::Object(m)
                }
                _ => {
                    let mut m = serde_json::Map::new();
                    m.insert("value".to_string(), val.clone());
                    JsonValue::Object(m)
                }
            };
            let formatted = toml::to_string_pretty(&toml_obj).map_err(|e| anyhow!("toml serialization failed: {}", e))?;
            fs::write(out_path, formatted)?;
        }
        "csv" | "tsv" | "psv" => {
            let delimiter = if out_ext == "tsv" { b'\t' } else if out_ext == "psv" { b'|' } else { b',' };
            let mut writer = csv::WriterBuilder::new().delimiter(delimiter).from_path(out_path)?;
            if let JsonValue::Array(items) = &val {
                if let Some(first) = items.first().and_then(|v| v.as_object()) {
                    let keys: Vec<&String> = first.keys().collect();
                    writer.write_record(&keys)?;
                    for item in items {
                        if let Some(obj) = item.as_object() {
                            let row: Vec<String> = keys.iter().map(|k| {
                                match obj.get(*k) {
                                    Some(JsonValue::String(s)) => s.clone(),
                                    Some(JsonValue::Null) | None => String::new(),
                                    Some(other) => other.to_string(),
                                }
                            }).collect();
                            writer.write_record(&row)?;
                        }
                    }
                } else if items.is_empty() {
                    writer.write_record(["key", "value"])?;
                } else {
                    writer.write_record(["value"])?;
                    for item in items {
                        let s = match item {
                            JsonValue::String(st) => st.clone(),
                            other => other.to_string(),
                        };
                        writer.write_record([s.as_str()])?;
                    }
                }
            } else if let JsonValue::Object(obj) = &val {
                writer.write_record(["key", "value"])?;
                for (k, v) in obj {
                    let v_str = match v {
                        JsonValue::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    writer.write_record([k.as_str(), &v_str])?;
                }
            } else {
                return Err(anyhow!("data must be array of objects or key-value map for tabular export"));
            }
            writer.flush()?;
        }
        "ndjson" => {
            let mut out = String::new();
            match &val {
                JsonValue::Array(items) => {
                    for item in items {
                        out.push_str(&serde_json::to_string(item)?);
                        out.push('\n');
                    }
                }
                other => {
                    out.push_str(&serde_json::to_string(other)?);
                    out.push('\n');
                }
            }
            fs::write(out_path, out)?;
        }
        "kdl" => {
            let mut out = String::new();
            match &val {
                JsonValue::Object(obj) => {
                    for (k, v) in obj {
                        let v_str = match v {
                            JsonValue::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
                            JsonValue::Number(n) => n.to_string(),
                            JsonValue::Bool(b) => b.to_string(),
                            JsonValue::Null => "null".to_string(),
                            other => format!("\"{}\"", other.to_string().replace('"', "\\\"")),
                        };
                        out.push_str(&format!("{} {}\n", k, v_str));
                    }
                }
                JsonValue::Array(items) => {
                    for (i, item) in items.iter().enumerate() {
                        out.push_str(&format!("item-{} {}\n", i, serde_json::to_string(item).unwrap_or_default()));
                    }
                }
                other => {
                    out.push_str(&format!("value {}\n", serde_json::to_string(other).unwrap_or_default()));
                }
            }
            fs::write(out_path, out)?;
        }
        "properties" => {
            let mut file = fs::File::create(out_path)?;
            match &val {
                JsonValue::Object(obj) => {
                    for (k, v) in obj {
                        let v_str = match v {
                            JsonValue::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        writeln!(file, "{}={}", k, v_str)?;
                    }
                }
                JsonValue::Array(items) => {
                    for (i, item) in items.iter().enumerate() {
                        writeln!(file, "item.{}={}", i, serde_json::to_string(item).unwrap_or_default())?;
                    }
                }
                _ => return Err(anyhow!("data must be map for properties export")),
            }
        }
        "parquet" | "arrow" | "sqlite" | "db" | "msgpack" | "cbor" | "bson" => {
            let py_cmd = r#"
import sys, json
json_str = sys.argv[1]
out_file = sys.argv[2]
out_ext = sys.argv[3].lower()
data = json.loads(json_str)
if isinstance(data, dict):
    rows = [data]
elif isinstance(data, list):
    rows = data
else:
    rows = [{"value": data}]
try:
    if out_ext == 'parquet':
        import pyarrow as pa
        import pyarrow.parquet as pq
        if rows and isinstance(rows[0], dict):
            cols = sorted({k for r in rows if isinstance(r, dict) for k in r.keys()})
            table = pa.table({c: [r.get(c) if isinstance(r, dict) else None for r in rows] for c in cols})
        else:
            table = pa.table({"value": rows})
        pq.write_table(table, out_file)
    elif out_ext == 'arrow':
        import pyarrow as pa
        import pyarrow.ipc as ipc
        if rows and isinstance(rows[0], dict):
            cols = sorted({k for r in rows if isinstance(r, dict) for k in r.keys()})
            table = pa.table({c: [str(r.get(c)) if isinstance(r, dict) and r.get(c) is not None else None for r in rows] for c in cols})
        else:
            table = pa.table({"value": [str(r) for r in rows]})
        with open(out_file, 'wb') as f:
            writer = ipc.new_file(f, table.schema)
            writer.write_table(table)
            writer.close()
    elif out_ext in ('sqlite', 'db'):
        import sqlite3
        import os
        if os.path.exists(out_file):
            os.remove(out_file)
        con = sqlite3.connect(out_file)
        cur = con.cursor()
        if rows and isinstance(rows[0], dict):
            cols = sorted({k for r in rows if isinstance(r, dict) for k in r.keys()})
            col_defs = ", ".join([f'"{c}" TEXT' for c in cols])
            cur.execute(f'CREATE TABLE data ({col_defs})')
            for r in rows:
                vals = [str(r.get(c)) if isinstance(r, dict) and r.get(c) is not None else None for c in cols]
                cur.execute(f'INSERT INTO data VALUES ({",".join(["?"]*len(cols))})', vals)
        else:
            cur.execute('CREATE TABLE data (value TEXT)')
            for r in rows:
                cur.execute('INSERT INTO data VALUES (?)', (str(r),))
        con.commit()
        con.close()
    elif out_ext == 'msgpack':
        import msgpack
        with open(out_file, 'wb') as f:
            f.write(msgpack.packb(data, use_bin_type=True))
    elif out_ext == 'cbor':
        import cbor2
        with open(out_file, 'wb') as f:
            cbor2.dump(data, f)
    elif out_ext == 'bson':
        import bson
        with open(out_file, 'wb') as f:
            if isinstance(data, list):
                for doc in data:
                    if not isinstance(doc, dict):
                        doc = {"value": doc}
                    f.write(bson.encode(doc))
            elif isinstance(data, dict):
                f.write(bson.encode(data))
            else:
                f.write(bson.encode({"value": data}))
except Exception as e:
    sys.stderr.write(str(e))
    sys.exit(1)
"#;
            let json_str = serde_json::to_string(&val)?;
            let output = std::process::Command::new("python").arg("-c").arg(py_cmd).arg(&json_str).arg(out_path).arg(&out_ext).output();
            if let Ok(res) = output {
                if !(res.status.success() && out_path.exists()) {
                    return Err(anyhow!("{} export failed: {}", out_ext, String::from_utf8_lossy(&res.stderr).trim()));
                }
            } else {
                return Err(anyhow!("{} export failed", out_ext));
            }
        }
        "ini" | "env" => {
            let mut file = fs::File::create(out_path)?;
            if let JsonValue::Object(obj) = &val {
                for (k, v) in obj {
                    let v_str = match v {
                        JsonValue::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    writeln!(file, "{}={}", k, v_str)?;
                }
            } else {
                return Err(anyhow!("data must be map for ini/env export"));
            }
        }
        "xml" | "plist" => {
            let py_cmd = r#"
import sys, json, plistlib
data = json.loads(sys.argv[1])
out_file = sys.argv[2]
out_ext = sys.argv[3].lower()
if out_ext == 'plist':
    with open(out_file, 'wb') as f:
        plistlib.dump(data, f)
else:
    import xmltodict
    root_data = {'root': data} if not isinstance(data, dict) or len(data) != 1 else data
    with open(out_file, 'w', encoding='utf-8') as f:
        f.write(xmltodict.unparse(root_data, pretty=True))
"#;
            let json_str = serde_json::to_string(&val)?;
            let output = std::process::Command::new("python").arg("-c").arg(py_cmd).arg(&json_str).arg(out_path).arg(&out_ext).output();
            if let Ok(res) = output {
                if res.status.success() && out_path.exists() {
                    return Ok(());
                }
            }
            let mut file = fs::File::create(out_path)?;
            writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<data>{}</data>", serde_json::to_string(&val)?)?;
        }
        "ron" => {
            let formatted = format!("(\n{}\n)", serde_yaml::to_string(&val)?.lines().map(|l| format!("    {}", l)).collect::<Vec<_>>().join("\n"));
            fs::write(out_path, formatted)?;
        }
        "html" | "htm" => {
            let mut html = String::from("<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>table{border-collapse:collapse;width:100%;font-family:sans-serif;}th,td{border:1px solid #ddd;padding:8px;text-align:left;}th{background-color:#f2f2f2;}</style></head><body><table>");
            if let JsonValue::Array(items) = &val {
                if let Some(first) = items.first().and_then(|v| v.as_object()) {
                    html.push_str("<tr>");
                    for k in first.keys() {
                        html.push_str(&format!("<th>{}</th>", k));
                    }
                    html.push_str("</tr>\n");
                    for item in items {
                        if let Some(obj) = item.as_object() {
                            html.push_str("<tr>");
                            for k in first.keys() {
                                let v_str = match obj.get(k) {
                                    Some(JsonValue::String(s)) => s.clone(),
                                    Some(JsonValue::Null) | None => String::new(),
                                    Some(other) => other.to_string(),
                                };
                                html.push_str(&format!("<td>{}</td>", v_str));
                            }
                            html.push_str("</tr>\n");
                        }
                    }
                }
            } else if let JsonValue::Object(obj) = &val {
                html.push_str("<tr><th>Key</th><th>Value</th></tr>\n");
                for (k, v) in obj {
                    let v_str = match v {
                        JsonValue::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    html.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>\n", k, v_str));
                }
            }
            html.push_str("</table></body></html>");
            fs::write(out_path, html)?;
        }
        _ => return Err(anyhow!("unsupported data output: {}", out_ext)),
    }
    Ok(())
}
