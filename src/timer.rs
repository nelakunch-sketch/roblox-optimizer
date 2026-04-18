// src/timer.rs
//
// Adjusts the Windows System Timer Resolution using the undocumented but
// widely used NtSetTimerResolution / NtQueryTimerResolution NT native APIs,
// loaded dynamically from ntdll.dll at runtime.
//
// Resolution units: 100-nanosecond (hns) intervals.
//   1ms  = 10_000 hns
//   0.5ms =  5_000 hns  ← target
//
// Safe: only affects the multimedia timer globally — no process memory touched.

use anyhow::{bail, Context, Result};
use windows::core::PCSTR;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

const STATUS_SUCCESS: i32 = 0;

/// Target resolution: 0.5 ms expressed in 100-nanosecond units.
pub const TARGET_RESOLUTION_HNS: u32 = 5_000;

type FnNtQueryTimerResolution = unsafe extern "system" fn(
    MinimumResolution: *mut u32,
    MaximumResolution: *mut u32,
    CurrentResolution: *mut u32,
) -> i32;

type FnNtSetTimerResolution = unsafe extern "system" fn(
    DesiredResolution: u32,
    SetResolution: u8, // BOOLEAN: 1 = set, 0 = restore
    CurrentResolution: *mut u32,
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

pub fn query_resolution() -> Result<TimerResolutionInfo> {
    let query_fn: FnNtQueryTimerResolution = get_ntdll_proc(b"NtQueryTimerResolution\0")?;

    let mut min: u32 = 0;
    let mut max: u32 = 0;
    let mut current: u32 = 0;

    let status = unsafe { query_fn(&mut min, &mut max, &mut current) };
    if status != STATUS_SUCCESS {
        bail!(
            "NtQueryTimerResolution returned NTSTATUS {:#010x}",
            status as u32
        );
    }

    Ok(TimerResolutionInfo {
        min_hns: min,
        max_hns: max,
        current_hns: current,
    })
}

pub fn set_resolution(desired_hns: u32) -> Result<u32> {
    let set_fn: FnNtSetTimerResolution = get_ntdll_proc(b"NtSetTimerResolution\0")?;

    let mut actual: u32 = 0;
    let status = unsafe {
        set_fn(desired_hns, 1 /* Set = TRUE */, &mut actual)
    };

    if status != STATUS_SUCCESS {
        bail!(
            "NtSetTimerResolution returned NTSTATUS {:#010x}",
            status as u32
        );
    }
    Ok(actual)
}

pub fn restore_resolution() -> Result<()> {
    let set_fn: FnNtSetTimerResolution = get_ntdll_proc(b"NtSetTimerResolution\0")?;

    let mut actual: u32 = 0;
    let status = unsafe { set_fn(TARGET_RESOLUTION_HNS, 0, &mut actual) };

    if status != STATUS_SUCCESS {
        bail!(
            "NtSetTimerResolution (restore) returned NTSTATUS {:#010x}",
            status as u32
        );
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct TimerResolutionInfo {
    /// Slowest possible resolution (largest interval).
    pub min_hns: u32,
    /// Fastest possible resolution (smallest interval).
    pub max_hns: u32,
    /// Current active resolution.
    pub current_hns: u32,
}

impl TimerResolutionInfo {
    pub fn current_ms(&self) -> f64 {
        self.current_hns as f64 / 10_000.0
    }
    pub fn min_ms(&self) -> f64 {
        self.min_hns as f64 / 10_000.0
    }
    pub fn max_ms(&self) -> f64 {
        self.max_hns as f64 / 10_000.0
    }
}

pub struct TimerResult {
    pub before_ms: f64,
    pub after_ms: f64,
    pub system_min_ms: f64,
    pub requested_ms: f64,
}

pub fn apply() -> Result<TimerResult> {
    let info_before = query_resolution()?;

    let desired = TARGET_RESOLUTION_HNS.max(info_before.max_hns);
    let actual = set_resolution(desired)?;

    Ok(TimerResult {
        before_ms: info_before.current_ms(),
        after_ms: actual as f64 / 10_000.0,
        system_min_ms: info_before.max_ms(), // max_hns = fastest = smallest ms
        requested_ms: desired as f64 / 10_000.0,
    })
}
