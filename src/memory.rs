// src/memory.rs
use anyhow::{bail, Context, Result};
use windows::core::PCSTR;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::SystemInformation::GlobalMemoryStatusEx;
use windows::Win32::System::SystemInformation::MEMORYSTATUSEX;

const STATUS_SUCCESS: i32 = 0;
const SYSTEM_MEMORY_LIST_INFORMATION: i32 = 80;

/// Commands accepted by SystemMemoryListInformation.
#[allow(dead_code)]
#[repr(u32)]
enum MemoryListCommand {
    MemoryFlushModifiedList = 3,
    MemoryPurgeStandbyList = 4,
    MemoryPurgeLowPriorityStandbyList = 5,
}

type FnNtSetSystemInformation = unsafe extern "system" fn(
    SystemInformationClass: i32,
    SystemInformation: *mut std::ffi::c_void,
    SystemInformationLength: u32,
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
            let total = msx.ullTotalPhys / 1024 / 1024;
            let avail = msx.ullAvailPhys / 1024 / 1024;
            return (total, avail);
        }
    }
    (0, 0)
}

pub struct MemoryResult {
    pub avail_before_mib: u64,
    pub avail_after_mib: u64,
    pub total_mib: u64,
    pub freed_mib: i64,
}

pub fn apply() -> Result<MemoryResult> {
    let set_sys_info: FnNtSetSystemInformation = get_ntdll_proc(b"NtSetSystemInformation\0")?;

    let (total_mib, avail_before) = ram_status();

    let mut cmd_flush = MemoryListCommand::MemoryFlushModifiedList as u32;
    let status = unsafe {
        set_sys_info(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &mut cmd_flush as *mut u32 as *mut std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
    };
    let _ = status;

    let mut cmd_purge = MemoryListCommand::MemoryPurgeStandbyList as u32;
    let status = unsafe {
        set_sys_info(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &mut cmd_purge as *mut u32 as *mut std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
    };
    if status != STATUS_SUCCESS {
        bail!(
            "NtSetSystemInformation (PurgeStandbyList) returned NTSTATUS {:#010x}",
            status as u32
        );
    }

    let mut cmd_lp = MemoryListCommand::MemoryPurgeLowPriorityStandbyList as u32;
    let _ = unsafe {
        set_sys_info(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &mut cmd_lp as *mut u32 as *mut std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
    };

    let (_, avail_after) = ram_status();
    let freed = avail_after as i64 - avail_before as i64;

    Ok(MemoryResult {
        avail_before_mib: avail_before,
        avail_after_mib: avail_after,
        total_mib,
        freed_mib: freed,
    })
}
