// src/process.rs
use anyhow::{Context, Result};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    GetPriorityClass, OpenProcess, SetPriorityClass, HIGH_PRIORITY_CLASS,
    PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION,
};

const ROBLOX_PROCESS_NAMES: &[&str] = &[
    "robloxplayerbeta.exe",
    "robloxplayer.exe",
    "robloxcrashhandler.exe",
    "robloxstudiobeta.exe",
    "robloxstudio.exe",
];

fn priority_name(class: u32) -> &'static str {
    match class {
        0x00000040 => "IDLE",
        0x00004000 => "BELOW_NORMAL",
        0x00000020 => "NORMAL",
        0x00008000 => "ABOVE_NORMAL",
        0x00000080 => "HIGH",
        0x00000100 => "REALTIME",
        _ => "UNKNOWN",
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub priority_was: String,
    pub priority_now: String,
    pub success: bool,
}

pub fn apply() -> Result<Vec<ProcessInfo>> {
    let snapshot = unsafe {
        CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .context("CreateToolhelp32Snapshot failed")?
    };

    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut results: Vec<ProcessInfo> = Vec::new();

    let first_ok = unsafe { Process32FirstW(snapshot, &mut entry) };
    if first_ok.is_err() {
        unsafe { CloseHandle(snapshot).ok() };
        return Ok(results);
    }

    loop {
        let name_wide = &entry.szExeFile;
        let len = name_wide
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(name_wide.len());
        let exe_name: String = String::from_utf16_lossy(&name_wide[..len]).to_lowercase();

        if ROBLOX_PROCESS_NAMES.iter().any(|&n| exe_name == n) {
            results.push(set_high_priority(entry.th32ProcessID, &exe_name));
        }

        if unsafe { Process32NextW(snapshot, &mut entry) }.is_err() {
            break;
        }
    }

    unsafe { CloseHandle(snapshot).ok() };
    Ok(results)
}

fn set_high_priority(pid: u32, name: &str) -> ProcessInfo {
    let access = PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION;

    match unsafe { OpenProcess(access, false, pid) } {
        Err(_) => ProcessInfo {
            pid,
            name: name.to_string(),
            priority_was: "N/A".to_string(),
            priority_now: "N/A".to_string(),
            success: false,
        },
        Ok(handle) => {
            let was = unsafe { GetPriorityClass(handle) };
            let was_name = priority_name(was).to_string();
            let success = unsafe { SetPriorityClass(handle, HIGH_PRIORITY_CLASS) }.is_ok();
            let now = unsafe { GetPriorityClass(handle) };
            unsafe { CloseHandle(handle).ok() };

            ProcessInfo {
                pid,
                name: name.to_string(),
                priority_was: was_name,
                priority_now: priority_name(now).to_string(),
                success,
            }
        }
    }
}

pub fn set_self_priority() -> bool {
    use windows::Win32::System::Threading::GetCurrentProcess;
    unsafe { SetPriorityClass(GetCurrentProcess(), HIGH_PRIORITY_CLASS).is_ok() }
}
