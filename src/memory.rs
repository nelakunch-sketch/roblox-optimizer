// src/memory.rs

// The function never hard-fails; it falls back gracefully and reports how
// much RAM was actually recovered.

use anyhow::{Context, Result};
use windows::core::PCSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Memory::{SetProcessWorkingSetSizeEx, SETPROCESSWORKINGSETSIZEEX_FLAGS};
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA,
};

const STATUS_SUCCESS: i32 = 0;
const SYSTEM_MEMORY_LIST_INFORMATION: i32 = 80;

#[allow(dead_code)]
#[repr(u32)]
enum MemoryListCommand {
    MemoryFlushModifiedList = 3,
    MemoryPurgeStandbyList = 4,
    MemoryPurgeLowPriorityStandbyList = 5,
}

type FnNtSetSystemInformation = unsafe extern "system" fn(
    system_information_class: i32,
    system_information: *mut std::ffi::c_void,
    system_information_length: u32,
) -> i32;

fn get_ntdll_proc<T>(name: &[u8]) -> Result<T> {
    unsafe {
        let ntdll_wide: Vec<u16> = "ntdll.dll\0".encode_utf16().collect();
        let hmod = GetModuleHandleW(windows::core::PCWSTR(ntdll_wide.as_ptr()))
            .context("GetModuleHandleW(ntdll.dll) failed")?;
        let proc = GetProcAddress(hmod, PCSTR(name.as_ptr())).ok_or_else(|| {
            anyhow::anyhow!("GetProcAddress({:?}) not found", std::str::from_utf8(name))
        })?;
        Ok(std::mem::transmute_copy(&proc))
    }
}


pub fn ram_status() -> (u64, u64) {
    let mut msx = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    unsafe {
        if GlobalMemoryStatusEx(&mut msx).is_ok() {
            return (
                msx.ullTotalPhys / 1024 / 1024,
                msx.ullAvailPhys / 1024 / 1024,
            );
        }
    }
    (0, 0)
}

pub struct MemoryResult {
    #[allow(dead_code)]
    pub avail_before_mib: u64,
    pub avail_after_mib: u64,
    #[allow(dead_code)]
    pub total_mib: u64,
    pub freed_mib: i64,
    /// Human-readable description of which layers succeeded.
    pub method: String,
}

/// Returns Ok(true) if the purge succeeded, Ok(false) if privilege was denied.
fn try_kernel_purge() -> Result<bool> {
    let set_sys_info: FnNtSetSystemInformation =
        get_ntdll_proc(b"NtSetSystemInformation\0")?;

    // Flush modified list first (best-effort)
    let mut cmd = MemoryListCommand::MemoryFlushModifiedList as u32;
    unsafe {
        set_sys_info(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &mut cmd as *mut u32 as *mut std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        );
    }

    let mut cmd = MemoryListCommand::MemoryPurgeStandbyList as u32;
    let status = unsafe {
        set_sys_info(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &mut cmd as *mut u32 as *mut std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
    };

    if status == STATUS_SUCCESS {
        // Also purge low-priority standby (best-effort)
        let mut cmd = MemoryListCommand::MemoryPurgeLowPriorityStandbyList as u32;
        unsafe {
            set_sys_info(
                SYSTEM_MEMORY_LIST_INFORMATION,
                &mut cmd as *mut u32 as *mut std::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            );
        }
        return Ok(true);
    }

    // 0xC0000061 = STATUS_PRIVILEGE_NOT_HELD — expected on hardened Win10/11
    // Any other NTSTATUS is also non-fatal; we just fall through.
    Ok(false)
}

fn empty_all_working_sets() -> usize {
    // Trim our own process first
    unsafe {
        let _ = SetProcessWorkingSetSizeEx(
            GetCurrentProcess(),
            usize::MAX,
            usize::MAX,
            SETPROCESSWORKINGSETSIZEEX_FLAGS(0),
        );
    }

    let snapshot = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) } {
        Ok(h) => h,
        Err(_) => return 0,
    };

    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut trimmed = 0usize;

    if unsafe { Process32FirstW(snapshot, &mut entry) }.is_err() {
        unsafe { CloseHandle(snapshot).ok() };
        return 0;
    }

    loop {
        let pid = entry.th32ProcessID;
        // Skip System (PID 4) and Idle (PID 0)
        if pid > 4 {
            let access = PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA;
            if let Ok(handle) = unsafe { OpenProcess(access, false, pid) } {
                if unsafe { EmptyWorkingSet(handle) }.is_ok() {
                    trimmed += 1;
                }
                unsafe { CloseHandle(handle).ok() };
            }
        }

        if unsafe { Process32NextW(snapshot, &mut entry) }.is_err() {
            break;
        }
    }

    unsafe { CloseHandle(snapshot).ok() };
    trimmed
}

pub fn apply() -> Result<MemoryResult> {
    let (total_mib, avail_before) = ram_status();
    let mut methods: Vec<&str> = Vec::new();

    // Layer 1 — try kernel standby purge (may be denied on Win11)
    match try_kernel_purge() {
        Ok(true) => methods.push("StandbyPurge"),
        Ok(false) => methods.push("StandbyPurge(denied→fallback)"),
        Err(_) => methods.push("StandbyPurge(err→fallback)"),
    }

    // Layer 2 — always run EmptyWorkingSet sweep
    let trimmed = empty_all_working_sets();
    methods.push("EmptyWorkingSet");

    let (_, avail_after) = ram_status();
    let freed = avail_after as i64 - avail_before as i64;

    Ok(MemoryResult {
        avail_before_mib: avail_before,
        avail_after_mib: avail_after,
        total_mib,
        freed_mib: freed,
        method: format!("{} ({} processes trimmed)", methods.join(" + "), trimmed),
    })
}