// src/privilege.rs
use anyhow::{Context, Result};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

fn enable_privilege(privilege_name: &str) -> Result<()> {
    unsafe {
        let mut token_handle: HANDLE = HANDLE::default();
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token_handle,
        )
        .context("OpenProcessToken failed")?;

        let wide: Vec<u16> = privilege_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut luid = windows::Win32::Foundation::LUID::default();
        LookupPrivilegeValueW(PCWSTR::null(), PCWSTR(wide.as_ptr()), &mut luid).context(
            format!("LookupPrivilegeValueW failed for '{}'", privilege_name),
        )?;

        let mut tp = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [windows::Win32::Security::LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: SE_PRIVILEGE_ENABLED,
            }],
        };

        AdjustTokenPrivileges(
            token_handle,
            false,
            Some(&mut tp),
            std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
            None,
            None,
        )
        .context(format!(
            "AdjustTokenPrivileges failed for '{}'",
            privilege_name
        ))?;

        CloseHandle(token_handle).ok();
    }
    Ok(())
}

pub fn enable_required_privileges() -> Vec<(String, bool)> {
    let privileges = [
        "SeIncreaseBasePriorityPrivilege",
        "SeSystemProfilePrivilege",
        "SeLockMemoryPrivilege",
        "SeDebugPrivilege",
        "SeIncreaseWorkingSetPrivilege",
    ];

    privileges
        .iter()
        .map(|&p| {
            let ok = enable_privilege(p).is_ok();
            (p.to_string(), ok)
        })
        .collect()
}
