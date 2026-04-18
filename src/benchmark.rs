// src/benchmark.rs

// Benchmark Tool — measures system performance metrics and determines
// the optimal FastFlags configuration for the current machine.

// Strategy:
//   1. Collect hardware profile (CPU cores, RAM, GPU tier via DXGI).
//   2. Run a CPU micro-benchmark to gauge single-thread perf.
//   3. Measure memory bandwidth.
//   4. Score the machine and select the best flag preset.
//   5. Optionally apply those flags automatically.

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct HardwareProfile {
    pub logical_cores: u32,
    pub physical_cores: u32,
    pub total_ram_mib: u64,
    pub available_ram_mib: u64,
    pub os_version: String,
    pub gpu_description: String,
    pub gpu_dedicated_vram_mib: u64,
}

pub fn get_hardware_profile() -> HardwareProfile {
    use windows::Win32::System::SystemInformation::{
        GetSystemInfo, GlobalMemoryStatusEx, MEMORYSTATUSEX, SYSTEM_INFO,
    };

    let mut si = SYSTEM_INFO::default();
    let logical_cores = unsafe {
        GetSystemInfo(&mut si);
        si.dwNumberOfProcessors
    };

    let (total_ram, available_ram) = {
        let mut ms = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };
        unsafe {
            if GlobalMemoryStatusEx(&mut ms).is_ok() {
                (ms.ullTotalPhys / 1024 / 1024, ms.ullAvailPhys / 1024 / 1024)
            } else {
                (0, 0)
            }
        }
    };

    // GPU via DXGI
    let (gpu_desc, vram_mib) = get_primary_gpu_info();

    // OS version from registry
    let os_version = get_os_version();

    // Estimate physical cores (logical / 2 if HT likely, otherwise logical)
    let physical_cores = if logical_cores > 4 {
        logical_cores / 2
    } else {
        logical_cores
    };

    HardwareProfile {
        logical_cores,
        physical_cores,
        total_ram_mib: total_ram,
        available_ram_mib: available_ram,
        os_version,
        gpu_description: gpu_desc,
        gpu_dedicated_vram_mib: vram_mib,
    }
}

fn get_primary_gpu_info() -> (String, u64) {
    // Use DXGI to enumerate adapters
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};
        unsafe {
            if let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() {
                if let Ok(adapter) = factory.EnumAdapters1(0) {
                    if let Ok(desc) = adapter.GetDesc1() {
                        let desc_str: String = desc
                            .Description
                            .iter()
                            .take_while(|&&c| c != 0)
                            .map(|&c| char::from_u32(c as u32).unwrap_or('?'))
                            .collect();
                        let vram = desc.DedicatedVideoMemory as u64 / 1024 / 1024;
                        return (desc_str, vram);
                    }
                }
            }
        }
    }
    ("Unknown GPU".to_string(), 0)
}

fn get_os_version() -> String {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion") {
        let product: String = key.get_value("ProductName").unwrap_or_default();
        let build: String = key.get_value("CurrentBuildNumber").unwrap_or_default();
        return format!("{} (Build {})", product, build);
    }
    "Windows (unknown version)".to_string()
}

/// Returns single-thread score: iterations per millisecond (higher = faster).
pub fn cpu_benchmark() -> f64 {
    const ITERATIONS: u64 = 50_000_000;
    let start = Instant::now();

    // Mixed integer + float workload (compiler-resistant via black_box pattern)
    let mut acc: u64 = 1;
    for i in 1u64..=ITERATIONS {
        acc = acc.wrapping_mul(6364136223846793005).wrapping_add(i);
        acc ^= acc >> 33;
    }

    let elapsed = start.elapsed();
    let ms = elapsed.as_secs_f64() * 1000.0;
    // Prevent optimisation away
    let _ = acc;

    ITERATIONS as f64 / ms
}

/// Returns memory bandwidth estimate in MiB/s.
pub fn memory_bandwidth_benchmark() -> f64 {
    const BUF_SIZE: usize = 32 * 1024 * 1024; // 32 MiB
    let mut buf: Vec<u64> = vec![0u64; BUF_SIZE / 8];

    let start = Instant::now();
    for (i, val) in buf.iter_mut().enumerate() {
        *val = i as u64 * 6364136223846793005;
    }
    let elapsed = start.elapsed();
    let _ = buf[0]; // prevent elision

    let secs = elapsed.as_secs_f64().max(1e-9);
    BUF_SIZE as f64 / secs / 1024.0 / 1024.0
}

#[derive(Debug, Clone, PartialEq)]
pub enum MachineTier {
    UltraLow, // Old/budget hardware
    Low,      // 4 cores, 8 GB RAM, integrated GPU
    Mid,      // 6-8 cores, 16 GB, discrete GPU ≤4 GB
    High,     // 8+ cores, 16 GB+, discrete GPU >4 GB
    Ultra,    // 12+ cores, 32 GB+, high-end GPU
}

impl MachineTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            MachineTier::UltraLow => "Ultra-Low (potato)",
            MachineTier::Low => "Low-End",
            MachineTier::Mid => "Mid-Range",
            MachineTier::High => "High-End",
            MachineTier::Ultra => "Ultra / Enthusiast",
        }
    }

    /// Recommended FastFlag preset index (matches fastflags::get_presets())
    #[allow(dead_code)]
    pub fn recommended_preset(&self) -> usize {
        match self {
            MachineTier::UltraLow => 0, // Max FPS / Performance
            MachineTier::Low => 0,      // Max FPS / Performance
            MachineTier::Mid => 2,      // Balanced
            MachineTier::High => 2,     // Balanced
            MachineTier::Ultra => 4,    // Unlock All Textures
        }
    }

    pub fn recommended_render_quality(&self) -> u32 {
        match self {
            MachineTier::UltraLow => 1,
            MachineTier::Low => 2,
            MachineTier::Mid => 5,
            MachineTier::High => 10,
            MachineTier::Ultra => 21,
        }
    }

    pub fn recommended_fps_cap(&self) -> u32 {
        match self {
            MachineTier::UltraLow => 30,
            MachineTier::Low => 60,
            MachineTier::Mid => 144,
            MachineTier::High => 240,
            MachineTier::Ultra => 9999,
        }
    }
}

pub fn score_machine(hw: &HardwareProfile, cpu_score: f64, mem_bw: f64) -> MachineTier {
    let mut points = 0u32;

    // CPU cores
    points += match hw.logical_cores {
        0..=3 => 0,
        4..=5 => 1,
        6..=7 => 2,
        8..=11 => 3,
        _ => 4,
    };

    // RAM
    points += match hw.total_ram_mib {
        0..=4095 => 0,
        4096..=7999 => 1,
        8000..=15999 => 2,
        16000..=31999 => 3,
        _ => 4,
    };

    // GPU VRAM
    points += match hw.gpu_dedicated_vram_mib {
        0..=511 => 0, // Integrated
        512..=2047 => 1,
        2048..=4095 => 2,
        4096..=8191 => 3,
        _ => 4,
    };

    // CPU micro-benchmark (iterations/ms)
    points += if cpu_score < 20_000.0 {
        0
    } else if cpu_score < 50_000.0 {
        1
    } else if cpu_score < 100_000.0 {
        2
    } else if cpu_score < 200_000.0 {
        3
    } else {
        4
    };

    // Memory bandwidth (MiB/s)
    points += if mem_bw < 5_000.0 {
        0
    } else if mem_bw < 15_000.0 {
        1
    } else if mem_bw < 30_000.0 {
        2
    } else {
        3
    };

    match points {
        0..=4 => MachineTier::UltraLow,
        5..=8 => MachineTier::Low,
        9..=12 => MachineTier::Mid,
        13..=16 => MachineTier::High,
        _ => MachineTier::Ultra,
    }
}

pub fn build_recommended_flags(tier: &MachineTier, hw: &HardwareProfile) -> HashMap<String, Value> {
    use serde_json::json;
    let mut flags: HashMap<String, Value> = HashMap::new();

    // Always: disable telemetry
    flags.insert(
        "FFlagDebugDisableTelemetryEphemeralCounter".into(),
        json!(true),
    );
    flags.insert(
        "FFlagDebugDisableTelemetryEphemeralStat".into(),
        json!(true),
    );
    flags.insert("FFlagDebugDisableTelemetryEventIngest".into(), json!(true));
    flags.insert("FFlagDebugDisableTelemetryPoint".into(), json!(true));
    flags.insert("FFlagDebugDisableTelemetryV2Counter".into(), json!(true));
    flags.insert("FFlagDebugDisableTelemetryV2Event".into(), json!(true));
    flags.insert("FFlagDebugDisableTelemetryV2Stat".into(), json!(true));

    // Always: remove FPS cap
    flags.insert(
        "FFlagTaskSchedulerLimitTargetFpsTo2402".into(),
        json!(false),
    );
    flags.insert(
        "DFIntTaskSchedulerTargetFps".into(),
        json!(tier.recommended_fps_cap()),
    );

    // Render quality
    flags.insert(
        "DFIntDebugFRMQualityLevelOverride".into(),
        json!(tier.recommended_render_quality()),
    );

    // Shadow intensity
    let shadow = match tier {
        MachineTier::UltraLow | MachineTier::Low => json!(0),
        MachineTier::Mid => json!(1),
        _ => json!(3),
    };
    flags.insert("FIntRenderShadowIntensity".into(), shadow);

    // MSAA
    let msaa: i64 = match tier {
        MachineTier::UltraLow | MachineTier::Low => -1,
        MachineTier::Mid => 1,
        MachineTier::High => 2,
        MachineTier::Ultra => 4,
    };
    flags.insert("FIntDebugForceMSAASamples".into(), json!(msaa));

    // Texture compression for low-end
    if matches!(tier, MachineTier::UltraLow | MachineTier::Low) {
        flags.insert("FFlagRenderGpuTextureCompressor".into(), json!(true));
        flags.insert("FFlagGraphicsGLTextureReductionAmount".into(), json!(true));
        flags.insert("DFIntRenderClampRoughnessMax".into(), json!(-640000000i64));
    }

    // Network tweaks (always beneficial)
    flags.insert("DFIntConnectionMTUSize".into(), json!(900));
    flags.insert("DFIntMaxMissedWorldStepsRemembered".into(), json!(1));
    flags.insert("DFIntDataSendRate".into(), json!(40));
    flags.insert("DFIntS2PhysicsSendRate".into(), json!(60));

    // RAM-adaptive: reduce sim radius on low RAM
    if hw.total_ram_mib < 8192 {
        flags.insert("DFIntSimRadiusScale".into(), json!(0));
    }

    // Audio channels: reduce on low-end
    let audio_ch: u32 = if matches!(tier, MachineTier::UltraLow) {
        8
    } else if matches!(tier, MachineTier::Low) {
        16
    } else {
        32
    };
    flags.insert("DFIntAudioNumChannels".into(), json!(audio_ch));

    flags
}

pub struct BenchmarkReport {
    pub hardware: HardwareProfile,
    pub cpu_score: f64,
    pub mem_bw_mib_s: f64,
    pub tier: MachineTier,
    pub recommended_flags: HashMap<String, Value>,
    pub benchmark_duration: Duration,
}

pub fn run_full_benchmark() -> Result<BenchmarkReport> {
    let start = Instant::now();

    let hardware = get_hardware_profile();
    let cpu_score = cpu_benchmark();
    let mem_bw = memory_bandwidth_benchmark();
    let tier = score_machine(&hardware, cpu_score, mem_bw);
    let recommended_flags = build_recommended_flags(&tier, &hardware);

    Ok(BenchmarkReport {
        hardware,
        cpu_score,
        mem_bw_mib_s: mem_bw,
        tier,
        recommended_flags,
        benchmark_duration: start.elapsed(),
    })
}
