# RobloxOptimizer 🎮

> Windows System Optimizer for Roblox — **Open Source**

---

## ✅ What it does (and does NOT do)

| Feature               | Details                                                                    |
| --------------------- | -------------------------------------------------------------------------- |
| **Timer Resolution**  | Sets Windows multimedia timer to **0.5 ms** via `NtSetTimerResolution`     |
| **Process Priority**  | Sets Roblox process(es) to `HIGH_PRIORITY_CLASS` via `SetPriorityClass`    |
| **RAM Standby Purge** | Frees cached/standby pages via `NtSetSystemInformation` (same as RAMMap)   |
| **Network Registry**  | Writes `TCPNoDelay=1`, `TcpAckFrequency=1`, disables multimedia throttling |

| ❌ NOT included    | Why                                      |
| ------------------ | ---------------------------------------- |
| DLL injection      | Would trigger Hyperion/Byfron anti-cheat |
| Process memory R/W | Same — bannable offence                  |
| Kernel driver      | Unnecessary and risky                    |
| Anti-cheat bypass  | Out of scope and against ToS             |

---

## 🛠 Build from Source

### Requirements

- [Rust toolchain](https://rustup.rs/) — stable, `x86_64-pc-windows-msvc` target
- Windows 10/11 (Build is Windows-only)
- Visual Studio Build Tools (C++ workload) for MSVC linker

```powershell
# Install Rust target if needed
rustup target add x86_64-pc-windows-msvc

# Clone and build
git clone https://github.com/nelakunch-sketch/roblox-optimizer
cd roblox-optimizer
cargo build --release
```

The output binary will be at:

```
target\release\RobloxOptimizer.exe
```

### Release build (optimised, stripped)

```powershell
cargo build --release
```

---

## 🚀 Usage

1. Right-click `RobloxOptimizer.exe` → **Run as administrator**
2. Choose an option from the interactive menu:

```
SELECT OPTIMIZATION:
  1  Full Optimization (recommended)
  2  Timer Resolution only   (0.5 ms)
  3  Process Priority only   (Roblox → HIGH)
  4  RAM Standby List Purge  (free cached RAM)
  5  Network Registry Tweaks (TCPNoDelay …)
  6  Restore Network Defaults
  Q  Quit
```

3. Launch Roblox **after** running the optimizer for best effect.

> **Tip:** For Process Priority (option 3), start Roblox first, then run the optimizer so the process is found in the snapshot.

---

## 📂 Project Structure

```
roblox-optimizer/
├── Cargo.toml          # Dependencies & build profile
├── build.rs            # Embeds UAC admin manifest into .exe
├── manifest.xml        # Requests requireAdministrator elevation
├── README.md
└── src/
    ├── main.rs         # Entry point, interactive menu, orchestration
    ├── ui.rs           # ASCII banner, colored output helpers
    ├── privilege.rs    # Enable SeIncreaseBasePriority, SeDebug, etc.
    ├── timer.rs        # NtSetTimerResolution (ntdll dynamic load)
    ├── process.rs      # Snapshot → find Roblox → SetPriorityClass
    ├── memory.rs       # NtSetSystemInformation + standby list purge
    └── network.rs      # Registry tweaks via winreg crate
```

---

## ⚙️ Technical Details

### Timer Resolution (`timer.rs`)

- Loads `NtSetTimerResolution` and `NtQueryTimerResolution` dynamically from `ntdll.dll`
- Target: **5000 × 100ns = 0.5 ms**
- Clamped to hardware minimum (typically 0.5–15.6 ms depending on chipset)
- Reverts automatically when process exits (Windows restores to default)

### Process Priority (`process.rs`)

- Uses `CreateToolhelp32Snapshot` + `Process32FirstW/NextW` to enumerate processes
- Matches against known Roblox exe names (case-insensitive)
- Calls `SetPriorityClass(handle, HIGH_PRIORITY_CLASS)`
- Does **not** read/write process memory

### RAM Standby List (`memory.rs`)

- Calls `NtSetSystemInformation(SystemMemoryListInformation=80, MemoryPurgeStandbyList=4, ...)`
- Identical to what [RAMMap](https://docs.microsoft.com/en-us/sysinternals/downloads/rammap) and Process Hacker do
- Frees pages that are cached but not actively used, giving Roblox more physical RAM

### Network Registry (`network.rs`)

- `TCPNoDelay = 1` — disables Nagle's algorithm (reduces TCP batching delay)
- `TcpAckFrequency = 1` — sends ACK immediately instead of waiting to batch
- `TcpTimedWaitDelay = 30` — shortens socket TIME_WAIT state from 240s → 30s
- `NetworkThrottlingIndex = 0xFFFFFFFF` — disables Windows multimedia throttling
- Applied globally and per-NIC adapter

---

## 🔑 Required Privileges

The program enables these on its own token before running:

| Privilege                         | Purpose                           |
| --------------------------------- | --------------------------------- |
| `SeIncreaseBasePriorityPrivilege` | Raise process scheduling priority |
| `SeSystemProfilePrivilege`        | Timer resolution changes          |
| `SeLockMemoryPrivilege`           | Memory list operations            |
| `SeDebugPrivilege`                | Open other processes              |

---

## 🛡 Anti-Cheat Compatibility

This optimizer is designed to be safe with **Roblox Byfron/Hyperion**:

- No `WriteProcessMemory` / `ReadProcessMemory` calls
- No DLL loading into Roblox address space
- No hook installation (IAT, inline, SSDT)
- No driver loading
- Only OS-level public/semi-public APIs

---

## 📜 License

MIT License — see [LICENSE](LICENSE) file.

---

## 🤝 Contributing

Pull requests welcome! Ideas:

- GPU priority boost (`SetGpuPriorityClass` — not standard WinAPI, requires D3DKMT)
- MMCSS (Multimedia Class Scheduler Service) profile switching
- Power Plan switching to "High Performance" mode
- Automatic detection of game launch and re-apply on startup
