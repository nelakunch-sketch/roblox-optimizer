// src/network.rs
//
// Applies TCP/IP registry tweaks that reduce network latency for gaming.
// All changes are made via the normal Windows registry API — no packet
// injection, no raw socket access, no kernel patching.
//
// Tweaks applied:
//   HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters
//     TcpAckFrequency        = 1   (ACK every segment, reduce buffering)
//     TCPNoDelay             = 1   (Nagle algorithm disabled)
//     DefaultTTL             = 64  (standard TTL)
//     TcpTimedWaitDelay      = 30  (reduce TIME_WAIT from 240s → 30s)
//     MaxUserPort            = 65534
//     TcpWindowSize          = 65535 (window size for LAN)
//
//   Per-adapter interfaces (same keys per NIC):
//     TcpAckFrequency = 1
//     TCPNoDelay      = 1
//
//   HKLM\SOFTWARE\Microsoft\MSMQ\Parameters
//     TCPNoDelay = 1
//
//   HKLM\SYSTEM\CurrentControlSet\Services\AFD\Parameters
//     FastSendDatagramThreshold = 1024
//     DefaultReceiveWindow      = 65536
//     DefaultSendWindow         = 65536
//
// ⚠ A reboot (or at least disabling/re-enabling the NIC) is required for
//   some per-adapter values to take effect.

use anyhow::{Context, Result};
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Clone)]
pub struct RegChange {
    pub key: String,
    pub value: String,
    pub applied: bool,
    pub note: String,
}

pub struct NetworkResult {
    pub changes: Vec<RegChange>,
}

fn set_dword(hklm: &RegKey, sub_path: &str, value_name: &str, data: u32) -> RegChange {
    let key_path = sub_path.to_string();
    let val_name = value_name.to_string();

    let result: Result<()> = (|| {
        let (key, _) = hklm
            .create_subkey(sub_path)
            .with_context(|| format!("create_subkey({})", sub_path))?;
        key.set_value(value_name, &data)
            .with_context(|| format!("set_value({} = {})", value_name, data))?;
        Ok(())
    })();

    RegChange {
        key: format!("HKLM\\{}", key_path),
        value: format!("{} = {}", val_name, data),
        applied: result.is_ok(),
        note: if result.is_ok() {
            "OK".to_string()
        } else {
            result.unwrap_err().to_string()
        },
    }
}

pub fn apply() -> Result<NetworkResult> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let mut changes: Vec<RegChange> = Vec::new();

    let tcp_params = r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters";

    changes.push(set_dword(&hklm, tcp_params, "TcpAckFrequency", 1));
    changes.push(set_dword(&hklm, tcp_params, "TCPNoDelay", 1));
    changes.push(set_dword(&hklm, tcp_params, "DefaultTTL", 64));
    changes.push(set_dword(&hklm, tcp_params, "TcpTimedWaitDelay", 30));
    changes.push(set_dword(&hklm, tcp_params, "MaxUserPort", 65534));
    changes.push(set_dword(&hklm, tcp_params, "TcpWindowSize", 65535));

    let interfaces_path = r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces";

    if let Ok(interfaces) = hklm.open_subkey(interfaces_path) {
        for sub in interfaces.enum_keys().filter_map(|k| k.ok()) {
            let adapter_path = format!(r"{}\{}", interfaces_path, sub);
            changes.push(set_dword(&hklm, &adapter_path, "TcpAckFrequency", 1));
            changes.push(set_dword(&hklm, &adapter_path, "TCPNoDelay", 1));
        }
    }

    changes.push(set_dword(
        &hklm,
        r"SOFTWARE\Microsoft\MSMQ\Parameters",
        "TCPNoDelay",
        1,
    ));

    let afd = r"SYSTEM\CurrentControlSet\Services\AFD\Parameters";
    changes.push(set_dword(&hklm, afd, "FastSendDatagramThreshold", 1024));
    changes.push(set_dword(&hklm, afd, "DefaultReceiveWindow", 65536));
    changes.push(set_dword(&hklm, afd, "DefaultSendWindow", 65536));

    let mm_profile = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
    changes.push(set_dword(
        &hklm,
        mm_profile,
        "NetworkThrottlingIndex",
        0xFFFF_FFFF,
    ));
    changes.push(set_dword(&hklm, mm_profile, "SystemResponsiveness", 0)); // 0 = optimise for games

    Ok(NetworkResult { changes })
}

pub fn restore_defaults() -> Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let tcp_params = r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters";

    let (key, _) = hklm.create_subkey(tcp_params)?;
    key.delete_value("TcpAckFrequency").ok();
    key.delete_value("TCPNoDelay").ok();
    key.delete_value("TcpTimedWaitDelay").ok();
    key.delete_value("MaxUserPort").ok();
    key.delete_value("TcpWindowSize").ok();

    let mm_profile = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
    let (key2, _) = hklm.create_subkey(mm_profile)?;

    key2.set_value("NetworkThrottlingIndex", &10u32).ok();
    key2.set_value("SystemResponsiveness", &20u32).ok();

    Ok(())
}
