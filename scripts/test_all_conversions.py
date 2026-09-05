import os
import sys
import subprocess
import shutil
import re
from pathlib import Path
BASE_DIR = Path(__file__).resolve().parent.parent
SAMPLES_DIR = BASE_DIR / "tests" / "samples"
OUTPUT_DIR = BASE_DIR / "tests" / "test_outputs"
RF2F_EXE = BASE_DIR / "target" / "release" / "rf2f.exe"
def parse_formats():
    formats_file = BASE_DIR / "src" / "formats.rs"
    content = formats_file.read_text(encoding="utf-8")
    pattern = re.compile(
        r'FormatInfo\s*\{\s*ext:\s*"([^"]+)",\s*name:\s*"([^"]+)",\s*category:\s*FormatCategory::(\w+),\s*mime:\s*"[^"]+",\s*suggested_targets:\s*&\[([^\]]+)\]'
    )
    formats = []
    for match in pattern.finditer(content):
        ext, name, cat, targets_str = match.groups()
        targets = [t.strip().strip('"') for t in targets_str.split(",") if t.strip().strip('"')]
        formats.append({
            "ext": ext,
            "name": name,
            "category": cat,
            "targets": targets,
        })
    return formats
def run_all_tests():
    formats = parse_formats()
    print(f"Loaded {len(formats)} format definitions from src/formats.rs")
    if OUTPUT_DIR.exists():
        shutil.rmtree(OUTPUT_DIR)
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    total_combinations = 0
    passed = []
    failed = []
    missing_samples = []
    for fmt in formats:
        ext = fmt["ext"]
        cat = fmt["category"]
        targets = list(fmt["targets"])
        if cat in ("Image", "Video") and ext != "gif" and "gif" not in targets:
            targets.append("gif")
        cat_dir = SAMPLES_DIR / cat
        sample_file = cat_dir / f"sample.{ext}"
        if not sample_file.exists() or sample_file.stat().st_size == 0:
            missing_samples.append(f"{cat}/sample.{ext}")
            continue
        for target_ext in targets:
            if target_ext.lower().strip() == ext.lower().strip():
                continue
            clean_target = target_ext.lower().strip()
            total_combinations += 1
            test_run_dir = OUTPUT_DIR / f"{cat}_{ext}_to_{clean_target}"
            test_run_dir.mkdir(parents=True, exist_ok=True)
            cmd = [str(RF2F_EXE), "convert", str(sample_file), "--format", clean_target, "--output", str(test_run_dir)]
            try:
                res = subprocess.run(cmd, capture_output=True, text=True, timeout=25)
                produced_files = [f for f in test_run_dir.glob("*") if f.is_file() and f.stat().st_size > 0]
                if res.returncode == 0 and len(produced_files) > 0:
                    passed.append((ext, clean_target, cat, res.stdout.strip()))
                    print(f"  [OK] {cat}: {ext} -> {clean_target}")
                else:
                    err = res.stderr.strip() or res.stdout.strip()
                    failed.append((ext, clean_target, cat, err))
                    print(f"  [FAIL] {cat}: {ext} -> {clean_target} | {err[:80]}")
            except subprocess.TimeoutExpired:
                failed.append((ext, clean_target, cat, "timeout > 25s"))
                print(f"  [FAIL] {cat}: {ext} -> {clean_target} | timeout")
            finally:
                shutil.rmtree(test_run_dir, ignore_errors=True)
    print(f"\n==========================================")
    print(f"Total Combinations Tested: {total_combinations}")
    print(f"Passed: {len(passed)}")
    print(f"Failed: {len(failed)}")
    print(f"Missing Samples: {len(missing_samples)}")
    print(f"==========================================")
    if missing_samples:
        print(f"\nMissing sample files ({len(missing_samples)}):")
        for ms in missing_samples:
            print(f"  - {ms}")
    if failed:
        print(f"\nFailed combinations ({len(failed)}):")
        for in_ext, out_ext, cat, err in failed:
            print(f"  {cat}: {in_ext} -> {out_ext} -> {err[:120]}")
    if OUTPUT_DIR.exists():
        shutil.rmtree(OUTPUT_DIR, ignore_errors=True)
    return len(failed)
if __name__ == "__main__":
    run_all_tests()
