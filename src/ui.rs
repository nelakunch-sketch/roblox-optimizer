// src/ui.rs

use colored::Colorize;

pub const APP_VERSION: &str = "1.0.0";
pub const APP_NAME: &str = "RobloxOptimizer";

/// Prints the ASCII art banner.
pub fn print_banner() {
    println!(
        "{}",
        r#"
 ██████╗  ██████╗ ██████╗ ██╗      ██████╗ ██╗  ██╗
 ██╔══██╗██╔═══██╗██╔══██╗██║     ██╔═══██╗╚██╗██╔╝
 ██████╔╝██║   ██║██████╔╝██║     ██║   ██║ ╚███╔╝ 
 ██╔══██╗██║   ██║██╔══██╗██║     ██║   ██║ ██╔██╗ 
 ██║  ██║╚██████╔╝██████╔╝███████╗╚██████╔╝██╔╝ ██╗
 ╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚══════╝ ╚═════╝ ╚═╝  ╚═╝
     ██████╗ ██████╗ ████████╗██╗███╗   ███╗██╗███████╗███████╗██████╗ 
    ██╔═══██╗██╔══██╗╚══██╔══╝██║████╗ ████║██║╚══███╔╝██╔════╝██╔══██╗
    ██║   ██║██████╔╝   ██║   ██║██╔████╔██║██║  ███╔╝ █████╗  ██████╔╝
    ██║   ██║██╔═══╝    ██║   ██║██║╚██╔╝██║██║ ███╔╝  ██╔══╝  ██╔══██╗
    ╚██████╔╝██║        ██║   ██║██║ ╚═╝ ██║██║███████╗███████╗██║  ██║
     ╚═════╝ ╚═╝        ╚═╝   ╚═╝╚═╝     ╚═╝╚═╝╚══════╝╚══════╝╚═╝  ╚═╝"#
            .bright_cyan()
            .bold()
    );

    println!(
        "  {} {} — {}",
        "Version:".bright_black(),
        APP_VERSION.bright_white().bold(),
        "Windows System Optimizer for Roblox".bright_yellow()
    );
    println!(
        "  {} {}",
        "⚠ ".yellow(),
        "No code injection | No memory tampering".bright_green()
    );
    println!("{}", "─".repeat(70).bright_black());
    println!();
}

/// Prints a section header (e.g., "[ Timer Resolution ]").
pub fn section(title: &str) {
    println!();
    println!("  {} {}", "▶".bright_cyan(), title.bright_white().bold());
    println!("  {}", "─".repeat(50).bright_black());
}

/// Print a success line: ✔ message
pub fn ok(msg: &str) {
    println!("    {} {}", "✔".bright_green().bold(), msg.bright_white());
}

/// Print an info line: ℹ message
pub fn info(msg: &str) {
    println!("    {} {}", "ℹ".bright_blue(), msg);
}

/// Print a warning line: ⚠ message
pub fn warn(msg: &str) {
    println!("    {} {}", "⚠".yellow(), msg.yellow());
}

/// Print an error line: ✘ message
pub fn err(msg: &str) {
    println!("    {} {}", "✘".bright_red().bold(), msg.bright_red());
}

/// Print a key-value pair.
pub fn kv(key: &str, value: &str) {
    println!(
        "    {:.<40} {}",
        format!("  {} ", key).bright_black(),
        value.bright_yellow().bold()
    );
}

/// Print a final summary box.
pub fn summary(results: &[(String, bool, String)]) {
    println!();
    println!("{}", "─".repeat(70).bright_black());
    println!("  {}", "OPTIMIZATION SUMMARY".bright_white().bold());
    println!("{}", "─".repeat(70).bright_black());
    for (task, success, detail) in results {
        let (icon, color_task) = if *success {
            ("✔".bright_green(), task.bright_white().to_string())
        } else {
            ("✘".bright_red(), task.bright_red().to_string())
        };
        println!("  {} {:<40} {}", icon, color_task, detail.bright_black());
    }
    println!("{}", "─".repeat(70).bright_black());
    println!();
}

/// Wait for Enter key press.
pub fn press_enter_to_exit() {
    println!("  {}", "Press ENTER to exit...".bright_black());
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);
}
