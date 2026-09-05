import sys
import os
import re
def clean_rust_c_js(content):
    res = []
    i = 0
    n = len(content)
    in_str = None
    in_line_cmt = False
    in_block_cmt = False
    raw_str = False
    while i < n:
        c = content[i]
        nxt = content[i + 1] if i + 1 < n else ''
        if in_line_cmt:
            if c == '\n':
                in_line_cmt = False
                res.append('\n')
            i += 1
        elif in_block_cmt:
            if c == '*' and nxt == '/':
                in_block_cmt = False
                i += 2
            else:
                if c == '\n':
                    res.append('\n')
                i += 1
        elif in_str:
            res.append(c)
            if not raw_str and c == '\\':
                if i + 1 < n:
                    res.append(content[i + 1])
                    i += 2
                    continue
            elif c == in_str:
                in_str = None
                raw_str = False
            i += 1
        else:
            if c == 'r' and (nxt == '"' or nxt == '#'):
                res.append(c)
                i += 1
            elif c in ('"', "'"):
                in_str = c
                res.append(c)
                i += 1
            elif c == '/' and nxt == '/':
                in_line_cmt = True
                i += 2
            elif c == '/' and nxt == '*':
                in_block_cmt = True
                i += 2
            else:
                res.append(c)
                i += 1
    cleaned = "".join(res)
    lines = [line.strip('\r') for line in cleaned.split('\n') if line.strip()]
    return "\n".join(lines) + ("\n" if lines else "")
def clean_hash_comment(content):
    lines = []
    for line in content.splitlines():
        line = line.strip('\r')
        if not line.strip():
            continue
        in_str = None
        cmt_idx = -1
        for idx, ch in enumerate(line):
            if in_str:
                if ch == '\\' and idx + 1 < len(line):
                    continue
                elif ch == in_str:
                    in_str = None
            else:
                if ch in ('"', "'"):
                    in_str = ch
                elif ch == '#':
                    cmt_idx = idx
                    break
        stripped = line[:cmt_idx].rstrip() if cmt_idx >= 0 else line
        if stripped.strip():
            lines.append(stripped)
    return "\n".join(lines) + ("\n" if lines else "")
def clean_file(path):
    ext = os.path.splitext(path)[1].lower()
    try:
        with open(path, 'r', encoding='utf-8') as f:
            content = f.read()
    except Exception as e:
        return False
    if ext in ('.rs', '.c', '.cpp', '.h', '.hpp', '.js', '.ts', '.jsx', '.tsx', '.java', '.cs', '.go', '.css', '.scss'):
        new_content = clean_rust_c_js(content)
    elif ext in ('.py', '.sh', '.bash', '.ps1', '.toml', '.yaml', '.yml', '.ini', '.env'):
        new_content = clean_hash_comment(content)
    else:
        lines = [line.strip('\r') for line in content.splitlines() if line.strip()]
        new_content = "\n".join(lines) + ("\n" if lines else "")
    with open(path, 'w', encoding='utf-8') as f:
        f.write(new_content)
    return True
def process_target(target):
    if os.path.isfile(target):
        clean_file(target)
    elif os.path.isdir(target):
        for root, _, files in os.walk(target):
            if any(p in root for p in ['.git', 'target', 'node_modules', '_reference']):
                continue
            for file in files:
                fpath = os.path.join(root, file)
                clean_file(fpath)
if __name__ == '__main__':
    targets = sys.argv[1:] if len(sys.argv) > 1 else ['.']
    for t in targets:
        process_target(t)
