use anyhow::Result;
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use rf2f::engine::{ConversionEngine, ConversionJob};
use rf2f::formats::{find_format, get_suggested_conversions, is_video_source, FormatCategory, VIDEO_RESOLUTIONS, FORMAT_DATABASE};
use rf2f::installer::{install_tools, TOOLS};
use rf2f::shell::{register_context_menu, unregister_context_menu};
use std::path::PathBuf;
#[derive(Parser)]
#[command(name = "rf2f")]
#[command(author = "khrotu")]
#[command(version = "0.1.0")]
#[command(about = "Local file converter")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short, long)]
    format: Option<String>,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(short, long)]
    resolution: Option<String>,
    files: Vec<PathBuf>,
}
#[derive(Subcommand)]
enum Commands {
    Convert {
        #[arg(required = true)]
        files: Vec<PathBuf>,
        #[arg(short, long, required = true)]
        format: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(short, long)]
        recursive: bool,
        #[arg(short, long)]
        resolution: Option<String>,
    },
    Register,
    Unregister,
    Formats {
        #[arg(short, long)]
        category: Option<String>,
        #[arg(short, long)]
        ext: Option<String>,
    },
    Doctor {
        #[arg(short, long)]
        install: bool,
        #[arg(short, long)]
        all: bool,
        #[arg(short, long)]
        tool: Option<Vec<String>>,
    },
}
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Convert { files, format, output, recursive, resolution }) => {
            run_convert(files, format, output, recursive, resolution)?;
        }
        Some(Commands::Register) => {
            register_context_menu()?;
            println!("registered context menu and path");
        }
        Some(Commands::Unregister) => {
            unregister_context_menu()?;
            println!("unregistered context menu and path");
        }
        Some(Commands::Formats { category, ext }) => {
            run_formats(category, ext);
        }
        Some(Commands::Doctor { install, all, tool }) => {
            run_doctor(install, all, tool)?;
        }
        None => {
            if !cli.files.is_empty() && cli.format.is_some() {
                run_convert(cli.files, cli.format.unwrap(), cli.output, false, cli.resolution)?;
            } else {
                println!("rf2f: specify input files or run with --help");
            }
        }
    }
    Ok(())
}
fn run_convert(files: Vec<PathBuf>, format: String, output_dir: Option<PathBuf>, recursive: bool, resolution: Option<String>) -> Result<()> {
    let mut target_files = Vec::new();
    for path in files {
        if path.is_file() {
            target_files.push(path);
        } else if path.is_dir() {
            if recursive {
                for entry in walkdir::WalkDir::new(path).into_iter().flatten() {
                    if entry.path().is_file() {
                        target_files.push(entry.path().to_path_buf());
                    }
                }
            } else {
                target_files.push(path);
            }
        }
    }
    if target_files.is_empty() {
        println!("no input files provided");
        return Ok(());
    }
    let engine = ConversionEngine::new();
    let pb = ProgressBar::new(target_files.len() as u64);
    pb.set_style(ProgressStyle::default_bar().template("[{elapsed_precise}] {pos}/{len}").unwrap());
    let results: Vec<Result<rf2f::engine::ConversionResult>> = target_files.par_iter().map(|file_path| {
        pb.inc(1);
        let out_path = if let Some(ref dir) = output_dir {
            Some(dir.clone())
        } else {
            None
        };
        let job = ConversionJob {
            input_path: file_path.clone(),
            target_format: format.clone(),
            output_path: out_path,
            resolution: resolution.clone(),
        };
        engine.execute(&job)
    }).collect();
    pb.finish_and_clear();
    let mut success_count = 0;
    let mut fail_count = 0;
    for res in results {
        match res {
            Ok(r) => {
                success_count += 1;
                let in_name = r.input_path.file_name().unwrap_or_default().to_string_lossy();
                let out_name = r.output_path.file_name().unwrap_or_default().to_string_lossy();
                println!("{} -> {} [{}ms, {}]", in_name, out_name, r.duration_ms, r.backend);
            }
            Err(e) => {
                fail_count += 1;
                eprintln!("error: {}", e);
            }
        }
    }
    println!("done: {} ok, {} failed", success_count, fail_count);
    if fail_count > 0 {
        std::process::exit(1);
    }
    Ok(())
}
fn run_formats(category_filter: Option<String>, ext_filter: Option<String>) {
    if let Some(ext) = ext_filter {
        if let Some(info) = find_format(&ext) {
            println!("format: .{} ({})", info.ext, info.name);
            println!("category: {}", info.category.as_str());
            println!("mime: {}", info.mime);
            println!("targets: {}", get_suggested_conversions(&ext).join(", "));
            if is_video_source(&info.ext) || info.category == FormatCategory::Image {
                let res_list: Vec<String> = VIDEO_RESOLUTIONS.iter().map(|(label, dims)| format!("{} ({})", label, dims)).collect();
                println!("resolutions: {} (use --resolution <label|WxH>)", res_list.join(", "));
            }
        } else {
            eprintln!("unknown format: .{}", ext);
        }
        return;
    }
    println!("supported formats ({}):", FORMAT_DATABASE.len());
    for f in FORMAT_DATABASE {
        if let Some(ref cat) = category_filter {
            if !f.category.as_str().eq_ignore_ascii_case(cat) {
                continue;
            }
        }
        let targets = get_suggested_conversions(f.ext).join(", ");
        if is_video_source(f.ext) || f.category == FormatCategory::Image {
            let res_labels: Vec<&str> = VIDEO_RESOLUTIONS.iter().map(|(l, _)| *l).collect();
            println!("  .{:<8} {:<32} [{}] -> [{}] + resolutions [{}]", f.ext, f.name, f.category.as_str(), targets, res_labels.join(", "));
        } else {
            println!("  .{:<8} {:<32} [{}] -> [{}]", f.ext, f.name, f.category.as_str(), targets);
        }
    }
}
fn run_doctor(install: bool, force_all: bool, tools: Option<Vec<String>>) -> Result<()> {
    if install || force_all || tools.is_some() {
        install_tools(tools.as_deref(), force_all)?;
        println!();
    }
    println!("diagnostics:");
    println!("  native: active (image, data, archive, subtitle)");
    let mut missing_count = 0;
    for tool in TOOLS {
        let path_opt = (tool.finder)();
        if let Some(p) = path_opt {
            println!("  {}: {:?} ({})", tool.name, p, tool.desc);
        } else {
            missing_count += 1;
            println!("  {}: not found ({})", tool.name, tool.desc);
        }
    }
    if missing_count > 0 && !install {
        println!("\nrun 'rf2f doctor --install' to install missing tools");
    }
    Ok(())
}
