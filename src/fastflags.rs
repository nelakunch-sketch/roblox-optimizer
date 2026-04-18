// src/fastflags.rs
//
// FastFlags Manager — read/write Roblox ClientAppSettings.json
// Supports any flag name/value pair with presets and custom entries.
// Goes beyond Bloxstrap by: full JSON merge, typed values, preset groups,
// export/import, and per-flag comments.

use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A single FastFlag entry.
#[derive(Debug, Clone)]
pub struct FlagEntry {
    pub name: String,
    pub value: Value,
    pub description: String,
    pub category: FlagCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlagCategory {
    Graphics,
    Network,
    Physics,
    Audio,
    Debug,
    #[allow(dead_code)]
    Custom,
}

impl FlagCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlagCategory::Graphics => "Graphics",
            FlagCategory::Network => "Network",
            FlagCategory::Physics => "Physics",
            FlagCategory::Audio => "Audio",
            FlagCategory::Debug => "Debug",
            FlagCategory::Custom => "Custom",
        }
    }
}

/// Built-in preset groups
pub struct Preset {
    pub name: &'static str,
    pub description: &'static str,
    pub flags: Vec<(&'static str, Value)>,
}

pub fn get_presets() -> Vec<Preset> {
    vec![
        Preset {
            name: "Max FPS / Performance",
            description: "Maximise framerate, disable eye-candy",
            flags: vec![
                ("FFlagDebugGraphicsDisableMetal", json!(false)),
                ("FFlagTaskSchedulerLimitTargetFpsTo2402", json!(false)),
                ("DFIntTaskSchedulerTargetFps", json!(9999)),
                ("FFlagGraphicsGLTextureReductionAmount", json!(true)),
                ("FFlagRenderGpuTextureCompressor", json!(true)),
                ("DFIntTextureCompositorActiveJobs", json!(0)),
                ("FFlagDebugGraphicsDisableOpengl", json!(false)),
                ("FIntRenderShadowIntensity", json!(0)),
                ("DFIntRenderClampRoughnessMax", json!(-640000000)),
                ("FFlagNewLightAttenuation", json!(true)),
                ("FFlagFixGraphicsQuality", json!(true)),
                ("FIntDebugForceMSAASamples", json!(-1)),
                ("DFIntDebugFRMQualityLevelOverride", json!(1)),
                ("FFlagDebugDisableTelemetryEphemeralCounter", json!(true)),
                ("FFlagDebugDisableTelemetryEphemeralStat", json!(true)),
                ("FFlagDebugDisableTelemetryEventIngest", json!(true)),
                ("FFlagDebugDisableTelemetryPoint", json!(true)),
                ("FFlagDebugDisableTelemetryV2Counter", json!(true)),
                ("FFlagDebugDisableTelemetryV2Event", json!(true)),
                ("FFlagDebugDisableTelemetryV2Stat", json!(true)),
            ],
        },
        Preset {
            name: "Low Latency Network",
            description: "Reduce ping and packet buffering",
            flags: vec![
                ("DFIntConnectionMTUSize", json!(900)),
                ("DFIntMaxMissedWorldStepsRemembered", json!(1)),
                ("DFIntOptimizeSendDataPacketsPerStep", json!(1)),
                ("DFIntDataSendRate", json!(40)),
                ("DFIntS2PhysicsSendRate", json!(60)),
                ("FFlagDebugDisableTelemetryEphemeralCounter", json!(true)),
                ("DFIntHttpMaxConcurrentRequests", json!(256)),
            ],
        },
        Preset {
            name: "Balanced (1080p Quality)",
            description: "Good visuals while keeping performance",
            flags: vec![
                ("DFIntTaskSchedulerTargetFps", json!(144)),
                ("FFlagTaskSchedulerLimitTargetFpsTo2402", json!(false)),
                ("DFIntDebugFRMQualityLevelOverride", json!(4)),
                ("FIntRenderShadowIntensity", json!(1)),
                ("FFlagNewLightAttenuation", json!(true)),
            ],
        },
        Preset {
            name: "Disable Telemetry / Analytics",
            description: "Stop Roblox from sending analytics data",
            flags: vec![
                ("FFlagDebugDisableTelemetryEphemeralCounter", json!(true)),
                ("FFlagDebugDisableTelemetryEphemeralStat", json!(true)),
                ("FFlagDebugDisableTelemetryEventIngest", json!(true)),
                ("FFlagDebugDisableTelemetryPoint", json!(true)),
                ("FFlagDebugDisableTelemetryV2Counter", json!(true)),
                ("FFlagDebugDisableTelemetryV2Event", json!(true)),
                ("FFlagDebugDisableTelemetryV2Stat", json!(true)),
            ],
        },
        Preset {
            name: "Unlock All Textures / Full Quality",
            description: "Force maximum texture and render quality",
            flags: vec![
                ("DFIntDebugFRMQualityLevelOverride", json!(21)),
                ("FFlagCommitToGraphicsQualityFix", json!(true)),
                ("FFlagFixGraphicsQuality", json!(true)),
                ("FIntDebugForceMSAASamples", json!(4)),
                ("DFIntRenderClampRoughnessMax", json!(0)),
            ],
        },
    ]
}

/// Returns all well-known individual flags with metadata.
pub fn get_known_flags() -> Vec<FlagEntry> {
    vec![
        // ── Graphics ──────────────────────────────────────────────────────
        FlagEntry {
            name: "DFIntTaskSchedulerTargetFps".into(),
            value: json!(9999),
            description: "Target FPS cap (9999 = unlimited)".into(),
            category: FlagCategory::Graphics,
        },
        FlagEntry {
            name: "FFlagTaskSchedulerLimitTargetFpsTo2402".into(),
            value: json!(false),
            description: "Remove internal 2402 FPS cap".into(),
            category: FlagCategory::Graphics,
        },
        FlagEntry {
            name: "DFIntDebugFRMQualityLevelOverride".into(),
            value: json!(1),
            description: "Force render quality level (1=lowest, 21=highest)".into(),
            category: FlagCategory::Graphics,
        },
        FlagEntry {
            name: "FIntRenderShadowIntensity".into(),
            value: json!(0),
            description: "Shadow intensity (0 = disabled)".into(),
            category: FlagCategory::Graphics,
        },
        FlagEntry {
            name: "FFlagNewLightAttenuation".into(),
            value: json!(true),
            description: "New light attenuation model".into(),
            category: FlagCategory::Graphics,
        },
        FlagEntry {
            name: "FIntDebugForceMSAASamples".into(),
            value: json!(-1),
            description: "MSAA samples (-1 = off, 1/2/4/8)".into(),
            category: FlagCategory::Graphics,
        },
        FlagEntry {
            name: "DFIntRenderClampRoughnessMax".into(),
            value: json!(-640000000),
            description: "Clamp material roughness for performance".into(),
            category: FlagCategory::Graphics,
        },
        FlagEntry {
            name: "FFlagRenderGpuTextureCompressor".into(),
            value: json!(true),
            description: "GPU texture compression".into(),
            category: FlagCategory::Graphics,
        },
        FlagEntry {
            name: "FFlagGraphicsGLTextureReductionAmount".into(),
            value: json!(true),
            description: "Reduce texture resolution for performance".into(),
            category: FlagCategory::Graphics,
        },
        // ── Network ───────────────────────────────────────────────────────
        FlagEntry {
            name: "DFIntConnectionMTUSize".into(),
            value: json!(900),
            description: "Network MTU size".into(),
            category: FlagCategory::Network,
        },
        FlagEntry {
            name: "DFIntMaxMissedWorldStepsRemembered".into(),
            value: json!(1),
            description: "Max missed physics steps before correction".into(),
            category: FlagCategory::Network,
        },
        FlagEntry {
            name: "DFIntOptimizeSendDataPacketsPerStep".into(),
            value: json!(1),
            description: "Data packets per game step".into(),
            category: FlagCategory::Network,
        },
        FlagEntry {
            name: "DFIntDataSendRate".into(),
            value: json!(40),
            description: "Network data send rate (Hz)".into(),
            category: FlagCategory::Network,
        },
        FlagEntry {
            name: "DFIntS2PhysicsSendRate".into(),
            value: json!(60),
            description: "Server→client physics send rate".into(),
            category: FlagCategory::Network,
        },
        FlagEntry {
            name: "DFIntHttpMaxConcurrentRequests".into(),
            value: json!(256),
            description: "Max concurrent HTTP requests".into(),
            category: FlagCategory::Network,
        },
        // ── Physics ───────────────────────────────────────────────────────
        FlagEntry {
            name: "DFIntSimRadiusScale".into(),
            value: json!(0),
            description: "Physics simulation radius scale".into(),
            category: FlagCategory::Physics,
        },
        FlagEntry {
            name: "FFlagDebugSimIntegratorEulerMode".into(),
            value: json!(false),
            description: "Use Euler integrator (legacy physics)".into(),
            category: FlagCategory::Physics,
        },
        // ── Audio ─────────────────────────────────────────────────────────
        FlagEntry {
            name: "FFlagDebugRomarkMockAudioDevices".into(),
            value: json!(false),
            description: "Mock audio devices (disable audio)".into(),
            category: FlagCategory::Audio,
        },
        FlagEntry {
            name: "DFIntAudioNumChannels".into(),
            value: json!(32),
            description: "Number of concurrent audio channels".into(),
            category: FlagCategory::Audio,
        },
        // ── Telemetry / Debug ─────────────────────────────────────────────
        FlagEntry {
            name: "FFlagDebugDisableTelemetryEphemeralCounter".into(),
            value: json!(true),
            description: "Disable ephemeral telemetry counters".into(),
            category: FlagCategory::Debug,
        },
        FlagEntry {
            name: "FFlagDebugDisableTelemetryEphemeralStat".into(),
            value: json!(true),
            description: "Disable ephemeral telemetry stats".into(),
            category: FlagCategory::Debug,
        },
        FlagEntry {
            name: "FFlagDebugDisableTelemetryEventIngest".into(),
            value: json!(true),
            description: "Disable event ingestion telemetry".into(),
            category: FlagCategory::Debug,
        },
        FlagEntry {
            name: "FFlagDebugDisableTelemetryPoint".into(),
            value: json!(true),
            description: "Disable telemetry point".into(),
            category: FlagCategory::Debug,
        },
        FlagEntry {
            name: "FFlagDebugDisableTelemetryV2Counter".into(),
            value: json!(true),
            description: "Disable telemetry v2 counter".into(),
            category: FlagCategory::Debug,
        },
        FlagEntry {
            name: "FFlagDebugDisableTelemetryV2Event".into(),
            value: json!(true),
            description: "Disable telemetry v2 event".into(),
            category: FlagCategory::Debug,
        },
        FlagEntry {
            name: "FFlagDebugDisableTelemetryV2Stat".into(),
            value: json!(true),
            description: "Disable telemetry v2 stat".into(),
            category: FlagCategory::Debug,
        },
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// File I/O
// ─────────────────────────────────────────────────────────────────────────────

pub fn find_roblox_versions_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // Standard localappdata path
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let roblox_versions = PathBuf::from(&local).join("Roblox").join("Versions");
        if roblox_versions.exists() {
            if let Ok(entries) = fs::read_dir(&roblox_versions) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let exe = path.join("RobloxPlayerBeta.exe");
                        if exe.exists() {
                            dirs.push(path);
                        }
                    }
                }
            }
        }
    }

    // Bloxstrap path
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let blox = PathBuf::from(&local)
            .join("Bloxstrap")
            .join("Versions");
        if blox.exists() {
            if let Ok(entries) = fs::read_dir(&blox) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        dirs.push(path);
                    }
                }
            }
        }
    }

    dirs
}

pub fn get_client_settings_path(version_dir: &PathBuf) -> PathBuf {
    version_dir
        .join("ClientSettings")
        .join("ClientAppSettings.json")
}

/// Read the current flags JSON from a version directory.
pub fn read_flags(version_dir: &PathBuf) -> Result<Map<String, Value>> {
    let path = get_client_settings_path(version_dir);
    if !path.exists() {
        return Ok(Map::new());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Cannot read {}", path.display()))?;
    let v: Value = serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON in {}", path.display()))?;
    match v {
        Value::Object(map) => Ok(map),
        _ => Ok(Map::new()),
    }
}

/// Write flags to a version directory (merge with existing).
pub fn write_flags(
    version_dir: &PathBuf,
    new_flags: &HashMap<String, Value>,
) -> Result<()> {
    let settings_dir = version_dir.join("ClientSettings");
    fs::create_dir_all(&settings_dir)
        .with_context(|| format!("Cannot create {}", settings_dir.display()))?;

    let path = get_client_settings_path(version_dir);

    // Read existing, then merge
    let mut existing = read_flags(version_dir).unwrap_or_default();
    for (k, v) in new_flags {
        existing.insert(k.clone(), v.clone());
    }

    let json_str = serde_json::to_string_pretty(&Value::Object(existing))
        .context("JSON serialization failed")?;
    fs::write(&path, json_str)
        .with_context(|| format!("Cannot write {}", path.display()))?;
    Ok(())
}

/// Remove specific flags from a version directory.
#[allow(dead_code)]
pub fn remove_flags(version_dir: &PathBuf, keys: &[String]) -> Result<()> {
    let mut existing = read_flags(version_dir).unwrap_or_default();
    for k in keys {
        existing.remove(k);
    }
    let path = get_client_settings_path(version_dir);
    let settings_dir = version_dir.join("ClientSettings");
    fs::create_dir_all(&settings_dir)?;
    let json_str = serde_json::to_string_pretty(&Value::Object(existing))?;
    fs::write(&path, json_str)?;
    Ok(())
}

/// Wipe all custom flags (reset to vanilla Roblox).
pub fn clear_all_flags(version_dir: &PathBuf) -> Result<()> {
    let path = get_client_settings_path(version_dir);
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("Cannot remove {}", path.display()))?;
    }
    Ok(())
}

/// Export current flags to a human-readable JSON file.
pub fn export_flags(version_dir: &PathBuf, output_path: &PathBuf) -> Result<usize> {
    let flags = read_flags(version_dir)?;
    let count = flags.len();
    let json_str = serde_json::to_string_pretty(&Value::Object(flags))?;
    fs::write(output_path, json_str)?;
    Ok(count)
}

/// Import flags from a JSON file (merges with existing).
pub fn import_flags(version_dir: &PathBuf, input_path: &PathBuf) -> Result<usize> {
    let content = fs::read_to_string(input_path)
        .with_context(|| format!("Cannot read {}", input_path.display()))?;
    let v: Value = serde_json::from_str(&content)?;
    let map = match v {
        Value::Object(m) => m,
        _ => anyhow::bail!("Expected a JSON object"),
    };
    let count = map.len();
    let hm: HashMap<String, Value> = map.into_iter().collect();
    write_flags(version_dir, &hm)?;
    Ok(count)
}

/// Apply a preset to all detected Roblox version directories.
pub fn apply_preset(preset_index: usize) -> Result<(usize, Vec<String>)> {
    let presets = get_presets();
    if preset_index >= presets.len() {
        anyhow::bail!("Invalid preset index");
    }
    let preset = &presets[preset_index];
    let flags: HashMap<String, Value> = preset
        .flags
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();

    let dirs = find_roblox_versions_dirs();
    let mut applied_to: Vec<String> = Vec::new();

    if dirs.is_empty() {
        anyhow::bail!("No Roblox installation found. Install Roblox first.");
    }

    for dir in &dirs {
        write_flags(dir, &flags)?;
        applied_to.push(dir.display().to_string());
    }

    Ok((flags.len(), applied_to))
}

/// Set a single custom flag across all Roblox dirs.
pub fn set_single_flag(name: &str, value: Value) -> Result<Vec<String>> {
    let dirs = find_roblox_versions_dirs();
    if dirs.is_empty() {
        anyhow::bail!("No Roblox installation found.");
    }
    let mut map = HashMap::new();
    map.insert(name.to_string(), value);
    let mut written = Vec::new();
    for dir in &dirs {
        write_flags(dir, &map)?;
        written.push(dir.display().to_string());
    }
    Ok(written)
}
