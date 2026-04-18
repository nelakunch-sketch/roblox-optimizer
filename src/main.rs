// src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "console")]

mod memory;
mod network;
mod privilege;
mod process;
mod timer;
mod ui;

use colored::Colorize;
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
            "q" | "0" | "exit" | "quit" => {
                println!();
                println!("  {}", "Exiting...".bright_black());
                break;
            }
            _ => {
                ui::warn("Unknown option. Please enter 1–6 or Q.");
            }
        }
    }

    ui::press_enter_to_exit();
}

// Menu
fn print_menu() {
    println!();
    println!("  {}", "SELECT OPTIMIZATION:".bright_white().bold());
    println!("  {}", "─".repeat(40).bright_black());
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
    println!("  {} {}", "Q".bright_yellow().bold(), "Quit");
    println!("  {}", "─".repeat(40).bright_black());
    print!("  {} ", "Choice:".bright_white());
    let _ = io::stdout().flush();
}

fn read_line() -> String {
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
    buf
}

// Admin elevation check
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

// ─────────────────────────────────────────────────────────────────────────────
// Full optimization
// ─────────────────────────────────────────────────────────────────────────────

fn run_full_optimize() {
    let mut summary_rows: Vec<(String, bool, String)> = Vec::new();

    // 0. Enable privileges
    ui::section("Enabling Required Token Privileges");
    let privs = privilege::enable_required_privileges();
    for (name, ok) in &privs {
        if *ok {
            ui::ok(&format!("Enabled: {}", name));
        } else {
            ui::warn(&format!("Skipped: {} (may not be available)", name));
        }
    }

    // 1. Timer Resolution
    step_timer(&mut summary_rows);

    // 2. Process Priority
    step_process(&mut summary_rows);

    // 3. RAM Standby Purge
    step_memory(&mut summary_rows);

    // 4. Network Registry
    step_network(&mut summary_rows);

    // Summary
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

// ─────────────────────────────────────────────────────────────────────────────
// Individual steps
// ─────────────────────────────────────────────────────────────────────────────

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

    // Set the optimizer itself to HIGH first.
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
            let freed = r.freed_mib;
            if freed > 0 {
                ui::ok(&format!("Freed ~{} MiB from standby list", freed));
            } else {
                ui::info("Standby list was already mostly empty.");
            }
            summary.push((
                "RAM Standby Purge".to_string(),
                true,
                format!("+{} MiB freed", freed.max(0)),
            ));
        }
        Err(e) => {
            ui::err(&format!("RAM purge failed: {}", e));
            summary.push(("RAM Standby Purge".to_string(), false, e.to_string()));
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
                // Only show per-adapter lines in aggregate to avoid screen flood
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
