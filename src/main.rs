// src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "console")]

mod benchmark;
mod fastflags;
mod memory;
mod network;
mod privilege;
mod process;
mod timer;
mod ui;

use colored::Colorize;
use serde_json::Value;
use std::io::{self, Write};

fn main() {
    ui::print_banner();

    if !is_elevated() {
        ui::err("This program requires Administrator privileges.");
        ui::err("Right-click → 'Run as administrator'.");
        ui::press_enter_to_exit();
        std::process::exit(1);
    }
    ui::ok("Running as Administrator");

    loop {
        print_menu();
        let choice = read_line().trim().to_lowercase();

        match choice.as_str() {
            "1" => run_full_optimize(),
            "2" => run_timer_only(),
            "3" => run_process_only(),
            "4" => run_memory_only(),
            "5" => run_network_only(),
            "6" => run_restore(),
            "7" => run_fastflags_menu(),
            "8" => run_benchmark_menu(),
            "q" | "0" | "exit" | "quit" => {
                println!();
                println!("  {}", "Exiting...".bright_black());
                break;
            }
            _ => {
                ui::warn("Unknown option. Please enter 1–8 or Q.");
            }
        }
    }

    ui::press_enter_to_exit();
}

fn print_menu() {
    println!();
    println!("  {}", "SELECT OPTIMIZATION:".bright_white().bold());
    println!("  {}", "─".repeat(50).bright_black());
    println!(
        "  {} {}",
        "1".bright_cyan().bold(),
        "Full Optimization (recommended)"
    );
    println!(
        "  {} {}",
        "2".bright_cyan().bold(),
        "Timer Resolution only   (0.5 ms)"
    );
    println!(
        "  {} {}",
        "3".bright_cyan().bold(),
        "Process Priority only   (Roblox → HIGH)"
    );
    println!(
        "  {} {}",
        "4".bright_cyan().bold(),
        "RAM Standby List Purge  (free cached RAM)"
    );
    println!(
        "  {} {}",
        "5".bright_cyan().bold(),
        "Network Registry Tweaks (TCPNoDelay …)"
    );
    println!(
        "  {} {}",
        "6".bright_cyan().bold(),
        "Restore Network Defaults"
    );
    println!(
        "  {} {}",
        "7".bright_magenta().bold(),
        "FastFlags Manager       (ClientAppSettings)"
    );
    println!(
        "  {} {}",
        "8".bright_yellow().bold(),
        "Benchmark & Auto-Optimize (AI flag picker)"
    );
    println!("  {} {}", "Q".bright_red().bold(), "Quit");
    println!("  {}", "─".repeat(50).bright_black());
    print!("  {} ", "Choice:".bright_white());
    let _ = io::stdout().flush();
}

fn run_fastflags_menu() {
    loop {
        println!();
        println!("  {}", "FASTFLAGS MANAGER".bright_magenta().bold());
        println!("  {}", "─".repeat(50).bright_black());
        println!("  {} Apply a Preset", "1".bright_cyan().bold());
        println!(
            "  {} Set a Single Flag (custom name & value)",
            "2".bright_cyan().bold()
        );
        println!("  {} View Current Flags", "3".bright_cyan().bold());
        println!(
            "  {} Browse Known Flags by Category",
            "4".bright_cyan().bold()
        );
        println!("  {} Export Flags to JSON", "5".bright_cyan().bold());
        println!("  {} Import Flags from JSON", "6".bright_cyan().bold());
        println!(
            "  {} Clear All Custom Flags (reset to vanilla)",
            "7".bright_cyan().bold()
        );
        println!("  {} Back", "B".bright_yellow().bold());
        println!("  {}", "─".repeat(50).bright_black());
        print!("  {} ", "Choice:".bright_white());
        let _ = io::stdout().flush();

        match read_line().trim().to_lowercase().as_str() {
            "1" => ff_apply_preset(),
            "2" => ff_set_single(),
            "3" => ff_view_current(),
            "4" => ff_browse_known(),
            "5" => ff_export(),
            "6" => ff_import(),
            "7" => ff_clear_all(),
            "b" | "0" => break,
            _ => ui::warn("Unknown option."),
        }
    }
}

fn get_roblox_dirs_or_warn() -> Option<Vec<std::path::PathBuf>> {
    let dirs = fastflags::find_roblox_versions_dirs();
    if dirs.is_empty() {
        ui::err("No Roblox installation detected.");
        ui::info("Install Roblox (or Bloxstrap) and try again.");
        return None;
    }
    Some(dirs)
}

fn ff_apply_preset() {
    let presets = fastflags::get_presets();
    println!();
    println!("  {}", "CHOOSE PRESET:".bright_white().bold());
    println!("  {}", "─".repeat(50).bright_black());
    for (i, p) in presets.iter().enumerate() {
        println!(
            "  {} {} — {}",
            format!("{}", i + 1).bright_cyan().bold(),
            p.name.bright_white(),
            p.description.bright_black()
        );
    }
    print!("  Preset number (or B to cancel): ");
    let _ = io::stdout().flush();

    let input = read_line();
    let trimmed = input.trim();
    if trimmed.eq_ignore_ascii_case("b") {
        return;
    }

    match trimmed.parse::<usize>() {
        Ok(n) if n >= 1 && n <= presets.len() => match fastflags::apply_preset(n - 1) {
            Ok((count, dirs)) => {
                ui::ok(&format!(
                    "{} flags written to {} dir(s):",
                    count,
                    dirs.len()
                ));
                for d in &dirs {
                    ui::info(&format!("  → {}", d));
                }
                ui::warn("Restart Roblox for changes to take effect.");
            }
            Err(e) => ui::err(&format!("Failed: {}", e)),
        },
        _ => ui::warn("Invalid selection."),
    }
}

fn ff_set_single() {
    println!();
    print!("  Flag name (e.g. DFIntTaskSchedulerTargetFps): ");
    let _ = io::stdout().flush();
    let name = read_line();
    let name = name.trim();
    if name.is_empty() {
        return;
    }

    print!("  Value (number, true/false, or \"string\"): ");
    let _ = io::stdout().flush();
    let raw = read_line();
    let raw = raw.trim();

    // Parse value: try number → bool → string
    let value: Value = if let Ok(n) = raw.parse::<i64>() {
        Value::Number(n.into())
    } else if let Ok(f) = raw.parse::<f64>() {
        Value::Number(serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0i64)))
    } else if raw.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if raw.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    } else {
        // Strip optional surrounding quotes
        let s = raw.trim_matches('"');
        Value::String(s.to_string())
    };

    match fastflags::set_single_flag(name, value.clone()) {
        Ok(dirs) => {
            ui::ok(&format!(
                "Set {} = {} in {} dir(s)",
                name,
                value,
                dirs.len()
            ));
            ui::warn("Restart Roblox for changes to take effect.");
        }
        Err(e) => ui::err(&format!("Failed: {}", e)),
    }
}

fn ff_view_current() {
    let dirs = match get_roblox_dirs_or_warn() {
        Some(d) => d,
        None => return,
    };

    for dir in &dirs {
        println!();
        ui::section(&format!("Flags in: {}", dir.display()));
        match fastflags::read_flags(dir) {
            Ok(map) if map.is_empty() => ui::info("No custom flags set."),
            Ok(map) => {
                let mut keys: Vec<_> = map.keys().collect();
                keys.sort();
                for k in keys {
                    ui::kv(k, &map[k].to_string());
                }
            }
            Err(e) => ui::err(&format!("Cannot read: {}", e)),
        }
    }
}

fn ff_browse_known() {
    use fastflags::FlagCategory;
    let all_flags = fastflags::get_known_flags();

    let categories = [
        FlagCategory::Graphics,
        FlagCategory::Network,
        FlagCategory::Physics,
        FlagCategory::Audio,
        FlagCategory::Debug,
    ];

    for cat in &categories {
        let flags: Vec<_> = all_flags.iter().filter(|f| &f.category == cat).collect();
        if flags.is_empty() {
            continue;
        }
        ui::section(cat.as_str());
        for f in flags {
            println!(
                "    {} {} = {}",
                "▸".bright_cyan(),
                f.name.bright_white(),
                f.value.to_string().bright_yellow()
            );
            println!("      {}", f.description.bright_black());
        }
    }

    println!();
    ui::info("Use option 2 (Set Single Flag) to apply any of these manually.");
}

fn ff_export() {
    let dirs = match get_roblox_dirs_or_warn() {
        Some(d) => d,
        None => return,
    };
    let dir = &dirs[0];

    print!("  Output path (e.g. C:\\flags_backup.json): ");
    let _ = io::stdout().flush();
    let path_str = read_line();
    let path = std::path::PathBuf::from(path_str.trim());

    match fastflags::export_flags(dir, &path) {
        Ok(n) => ui::ok(&format!("{} flags exported to {}", n, path.display())),
        Err(e) => ui::err(&format!("Export failed: {}", e)),
    }
}

fn ff_import() {
    let dirs = match get_roblox_dirs_or_warn() {
        Some(d) => d,
        None => return,
    };

    print!("  Path to JSON file: ");
    let _ = io::stdout().flush();
    let path_str = read_line();
    let path = std::path::PathBuf::from(path_str.trim());

    for dir in &dirs {
        match fastflags::import_flags(dir, &path) {
            Ok(n) => ui::ok(&format!("{} flags imported into {}", n, dir.display())),
            Err(e) => ui::err(&format!("Import failed for {}: {}", dir.display(), e)),
        }
    }
    ui::warn("Restart Roblox for changes to take effect.");
}

fn ff_clear_all() {
    println!();
    print!("  Are you sure you want to delete ALL custom flags? (yes/no): ");
    let _ = io::stdout().flush();
    let confirm = read_line();
    if confirm.trim().to_lowercase() != "yes" {
        ui::info("Cancelled.");
        return;
    }

    let dirs = match get_roblox_dirs_or_warn() {
        Some(d) => d,
        None => return,
    };

    for dir in &dirs {
        match fastflags::clear_all_flags(dir) {
            Ok(_) => ui::ok(&format!("Cleared flags in {}", dir.display())),
            Err(e) => ui::err(&format!("Failed for {}: {}", dir.display(), e)),
        }
    }
}

fn run_benchmark_menu() {
    println!();
    ui::section("Benchmark & Auto-Optimize");
    ui::info("This will:");
    ui::info("  1. Profile your hardware (CPU, RAM, GPU)");
    ui::info("  2. Run a short CPU & memory benchmark (~5 seconds)");
    ui::info("  3. Score your machine and pick the best FastFlags");
    ui::info("  4. Optionally apply them automatically");
    println!();
    print!("  Press ENTER to start benchmark (or B to cancel): ");
    let _ = io::stdout().flush();

    let input = read_line();
    if input.trim().to_lowercase() == "b" {
        return;
    }

    ui::info("Running benchmark... please wait.");

    let report = match benchmark::run_full_benchmark() {
        Ok(r) => r,
        Err(e) => {
            ui::err(&format!("Benchmark failed: {}", e));
            return;
        }
    };

    print_benchmark_report(&report);

    println!();
    print!("  Apply recommended flags to Roblox now? (yes/no): ");
    let _ = io::stdout().flush();
    let confirm = read_line();

    if confirm.trim().to_lowercase() == "yes" {
        let dirs = fastflags::find_roblox_versions_dirs();
        if dirs.is_empty() {
            ui::err("No Roblox installation found — install Roblox first.");
        } else {
            for dir in &dirs {
                match fastflags::write_flags(dir, &report.recommended_flags) {
                    Ok(_) => ui::ok(&format!("Flags written to {}", dir.display())),
                    Err(e) => ui::err(&format!("Failed for {}: {}", dir.display(), e)),
                }
            }
            ui::ok(&format!(
                "{} flags applied to {} Roblox dir(s).",
                report.recommended_flags.len(),
                dirs.len()
            ));
            ui::warn("Restart Roblox for changes to take effect.");
        }
    } else {
        ui::info("Flags not applied. You can apply them via FastFlags Manager (option 7).");
    }
}

fn print_benchmark_report(r: &benchmark::BenchmarkReport) {
    println!();
    println!("{}", "─".repeat(70).bright_black());
    println!("  {}", "BENCHMARK RESULTS".bright_white().bold());
    println!("{}", "─".repeat(70).bright_black());

    // Hardware
    ui::section("Hardware Profile");
    ui::kv("OS", &r.hardware.os_version);
    ui::kv(
        "CPU Cores",
        &format!(
            "{} logical / {} physical",
            r.hardware.logical_cores, r.hardware.physical_cores
        ),
    );
    ui::kv(
        "RAM",
        &format!(
            "{} MiB total  |  {} MiB available",
            r.hardware.total_ram_mib, r.hardware.available_ram_mib
        ),
    );
    ui::kv("GPU", &r.hardware.gpu_description);
    ui::kv(
        "GPU VRAM",
        &format!("{} MiB", r.hardware.gpu_dedicated_vram_mib),
    );

    // Scores
    ui::section("Benchmark Scores");
    ui::kv("CPU Single-Thread", &format!("{:.0} ops/ms", r.cpu_score));
    ui::kv("Memory Bandwidth", &format!("{:.0} MiB/s", r.mem_bw_mib_s));
    ui::kv(
        "Benchmark Duration",
        &format!("{:.2}s", r.benchmark_duration.as_secs_f64()),
    );

    // Tier
    println!();
    println!(
        "  {} Machine Tier: {}",
        "▶".bright_cyan(),
        r.tier.as_str().bright_yellow().bold()
    );
    println!(
        "  {} Recommended Render Quality: {}",
        "▶".bright_cyan(),
        r.tier
            .recommended_render_quality()
            .to_string()
            .bright_yellow()
    );
    println!(
        "  {} Recommended FPS Cap: {}",
        "▶".bright_cyan(),
        r.tier.recommended_fps_cap().to_string().bright_yellow()
    );

    // Recommended flags
    ui::section("Recommended FastFlags");
    let mut keys: Vec<_> = r.recommended_flags.keys().collect();
    keys.sort();
    for k in &keys {
        ui::kv(k, &r.recommended_flags[*k].to_string());
    }

    println!("{}", "─".repeat(70).bright_black());
}

fn is_elevated() -> bool {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_len: u32 = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_len,
        );

        windows::Win32::Foundation::CloseHandle(token).ok();
        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

fn run_full_optimize() {
    let mut summary_rows: Vec<(String, bool, String)> = Vec::new();

    ui::section("Enabling Required Token Privileges");
    let privs = privilege::enable_required_privileges();
    for (name, ok) in &privs {
        if *ok {
            ui::ok(&format!("Enabled: {}", name));
        } else {
            ui::warn(&format!("Skipped: {} (may not be available)", name));
        }
    }

    step_timer(&mut summary_rows);
    step_process(&mut summary_rows);
    step_memory(&mut summary_rows);
    step_network(&mut summary_rows);

    ui::summary(&summary_rows);
    ui::info("Optimization complete! Launch Roblox now for best results.");
    ui::warn("Some network changes require a NIC toggle or reboot to fully apply.");
}

fn run_timer_only() {
    let mut s = vec![];
    step_timer(&mut s);
    ui::summary(&s);
}
fn run_process_only() {
    let mut s = vec![];
    step_process(&mut s);
    ui::summary(&s);
}
fn run_memory_only() {
    let mut s = vec![];
    step_memory(&mut s);
    ui::summary(&s);
}
fn run_network_only() {
    let mut s = vec![];
    step_network(&mut s);
    ui::summary(&s);
}

fn run_restore() {
    ui::section("Restoring Network Registry Defaults");
    match network::restore_defaults() {
        Ok(_) => ui::ok("Network registry values restored to Windows defaults."),
        Err(e) => ui::err(&format!("Restore failed: {}", e)),
    }
}
fn step_timer(summary: &mut Vec<(String, bool, String)>) {
    ui::section("System Timer Resolution");
    match timer::apply() {
        Ok(r) => {
            ui::kv("Before", &format!("{:.3} ms", r.before_ms));
            ui::kv("Requested", &format!("{:.3} ms", r.requested_ms));
            ui::kv("Actual", &format!("{:.3} ms", r.after_ms));
            ui::kv(
                "HW Limit",
                &format!("{:.3} ms (system minimum)", r.system_min_ms),
            );

            let ok = r.after_ms <= 1.0;
            if ok {
                ui::ok(&format!("Timer resolution set to {:.3} ms", r.after_ms));
            } else {
                ui::warn(&format!(
                    "Hardware limit reached: {:.3} ms (target 0.5 ms not achievable on this CPU/chipset)",
                    r.after_ms
                ));
            }
            summary.push((
                "Timer Resolution".to_string(),
                true,
                format!("{:.3} ms → {:.3} ms", r.before_ms, r.after_ms),
            ));
        }
        Err(e) => {
            ui::err(&format!("Timer resolution failed: {}", e));
            summary.push(("Timer Resolution".to_string(), false, e.to_string()));
        }
    }
}

fn step_process(summary: &mut Vec<(String, bool, String)>) {
    ui::section("Process Priority");

    if process::set_self_priority() {
        ui::ok("Optimizer process priority → HIGH");
    }

    match process::apply() {
        Ok(procs) if procs.is_empty() => {
            ui::warn("No Roblox processes found.");
            ui::info("Start Roblox first, then re-run option 3 to boost its priority.");
            summary.push((
                "Process Priority".to_string(),
                false,
                "Roblox not running".to_string(),
            ));
        }
        Ok(procs) => {
            for p in &procs {
                if p.success {
                    ui::ok(&format!(
                        "[{}] {} — {} → {}",
                        p.pid, p.name, p.priority_was, p.priority_now
                    ));
                } else {
                    ui::warn(&format!(
                        "[{}] {} — could not set priority (access denied?)",
                        p.pid, p.name
                    ));
                }
            }
            let all_ok = procs.iter().all(|p| p.success);
            summary.push((
                "Process Priority".to_string(),
                all_ok,
                format!("{} process(es) found", procs.len()),
            ));
        }
        Err(e) => {
            ui::err(&format!("Process priority failed: {}", e));
            summary.push(("Process Priority".to_string(), false, e.to_string()));
        }
    }
}

fn step_memory(summary: &mut Vec<(String, bool, String)>) {
    ui::section("RAM Standby List Purge");

    let (total, avail_before) = memory::ram_status();
    ui::kv("Total RAM", &format!("{} MiB", total));
    ui::kv("Available before", &format!("{} MiB", avail_before));

    match memory::apply() {
        Ok(r) => {
            ui::kv("Available after", &format!("{} MiB", r.avail_after_mib));
            ui::kv("Method", &r.method);
            let freed = r.freed_mib;
            if freed > 0 {
                ui::ok(&format!("Freed ~{} MiB", freed));
            } else {
                ui::info("Working sets trimmed; standby list was already lean.");
            }
            summary.push((
                "RAM Purge".to_string(),
                true,
                format!("+{} MiB freed", freed.max(0)),
            ));
        }
        Err(e) => {
            ui::err(&format!("RAM purge failed: {}", e));
            summary.push(("RAM Purge".to_string(), false, e.to_string()));
        }
    }
}

fn step_network(summary: &mut Vec<(String, bool, String)>) {
    ui::section("Network Registry Tweaks");

    match network::apply() {
        Ok(result) => {
            let mut applied = 0usize;
            let mut failed = 0usize;

            for change in &result.changes {
                if change.key.contains("Interfaces") && change.applied {
                    applied += 1;
                    continue;
                }
                if change.applied {
                    ui::ok(&format!(
                        "{}  ← {}",
                        change.value,
                        change.key.split('\\').last().unwrap_or("")
                    ));
                    applied += 1;
                } else {
                    ui::warn(&format!("SKIP {} ({})", change.value, change.note));
                    failed += 1;
                }
            }

            ui::info(&format!(
                "{} registry values written, {} skipped",
                applied, failed
            ));
            ui::warn("Disable/re-enable your NIC or reboot for per-adapter tweaks to apply.");

            summary.push((
                "Network Registry".to_string(),
                failed == 0,
                format!("{} values written", applied),
            ));
        }
        Err(e) => {
            ui::err(&format!("Network tweaks failed: {}", e));
            summary.push(("Network Registry".to_string(), false, e.to_string()));
        }
    }
}

fn read_line() -> String {
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
    buf
}
