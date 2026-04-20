//! 汇总系统指标并生成 `DashboardSnapshot`。

use crate::platform;
use crate::snapshot_types::{BrightnessDisplay, DashboardSnapshot, DiskRow, Metric};
use parking_lot::Mutex;
use std::time::Instant;
use sysinfo::{Components, Disk, Disks, Networks, System};
use tauri::State;

#[derive(Default)]
pub struct NetCounters {
    last_rx: u64,
    last_tx: u64,
    last_at: Option<Instant>,
}

fn is_loopback_iface(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("loopback") || n == "lo" || n.starts_with("lo")
}

fn is_network_fs(fs: &std::ffi::OsStr) -> bool {
    let s = fs.to_string_lossy().to_lowercase();
    s.contains("nfs")
        || s.contains("smb")
        || s.contains("cifs")
        || s.contains("webdav")
        || s.contains("afp")
}

#[cfg(windows)]
fn skip_disk(d: &Disk) -> bool {
    if is_network_fs(d.file_system()) {
        return true;
    }
    crate::platform::windows::is_network_drive(d.mount_point())
}

#[cfg(not(windows))]
fn skip_disk(d: &Disk) -> bool {
    is_network_fs(d.file_system())
        || d
            .mount_point()
            .to_string_lossy()
            .to_lowercase()
            .contains("//")
}

fn sum_network_bytes(nets: &Networks) -> (u64, u64) {
    let mut rx = 0u64;
    let mut tx = 0u64;
    for (name, data) in nets {
        if is_loopback_iface(name) {
            continue;
        }
        rx = rx.saturating_add(data.total_received());
        tx = tx.saturating_add(data.total_transmitted());
    }
    (rx, tx)
}

fn classify_temps(components: &Components) -> (Metric<f32>, Metric<f32>) {
    let mut gpu_val: Option<f32> = None;
    let mut cpu_candidates: Vec<(String, f32)> = Vec::new();

    let gpu_pat = [
        "gpu", "graphics", "geforce", "radeon", "nvidia", "adreno", "iris",
    ];
    let cpu_pat = [
        "cpu", "package", "soc", "die", "core", "apple m", "p-core", "e-core",
        "heat", "processor",
    ];

    for c in components.list() {
        let Some(t) = c.temperature() else {
            continue;
        };
        let label = c.label().to_lowercase();
        if gpu_pat.iter().any(|p| label.contains(p)) {
            if gpu_val.is_none() {
                gpu_val = Some(t);
            }
            continue;
        }
        if cpu_pat.iter().any(|p| label.contains(p)) || label == "computer" {
            cpu_candidates.push((c.label().to_string(), t));
        }
    }

    let cpu = if cpu_candidates.is_empty() {
        // 退回：取所有可读温度的最大值（排除已判为 GPU 的项无法区分时）
        let mut best: Option<(String, f32)> = None;
        for c in components.list() {
            if let Some(t) = c.temperature() {
                let lbl = c.label();
                let l = lbl.to_lowercase();
                if gpu_pat.iter().any(|p| l.contains(p)) {
                    continue;
                }
                if best.as_ref().map(|b| t > b.1).unwrap_or(true) {
                    best = Some((lbl.to_string(), t));
                }
            }
        }
        match best {
            Some((l, t)) => Metric {
                value: Some(t),
                note: Some(format!("启发式聚合: {l}")),
            },
            None => Metric {
                value: None,
                note: Some("未读取到 CPU 温度传感器".into()),
            },
        }
    } else {
        // 取 CPU 候选中的最高温作为「代表性」读数
        let (l, t) = cpu_candidates
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .unwrap();
        Metric {
            value: Some(t),
            note: Some(format!("代表性传感器: {l}")),
        }
    };

    let gpu = if gpu_val.is_none() {
        #[cfg(windows)]
        {
            if let Some(t) = platform::windows::try_nvidia_gpu_temp_c() {
                Metric {
                    value: Some(t),
                    note: Some("NVIDIA NVML".into()),
                }
            } else {
                Metric {
                    value: None,
                    note: Some("未读取到主 GPU 温度（非 NVIDIA 或无 NVML）".into()),
                }
            }
        }
        #[cfg(not(windows))]
        Metric {
            value: None,
            note: Some("未识别到 GPU 温度传感器".into()),
        }
    } else {
        Metric {
            value: gpu_val,
            note: None,
        }
    };

    (cpu, gpu)
}

pub fn build_snapshot(counters: &Mutex<NetCounters>) -> DashboardSnapshot {
    let mut sys = System::new();
    sys.refresh_memory();

    let networks = Networks::new_with_refreshed_list();
    let (rx, tx) = sum_network_bytes(&networks);

    let mut guard = counters.lock();
    let now = Instant::now();
    let (down_bps, up_bps) = if let Some(prev) = guard.last_at {
        let dt = now.duration_since(prev).as_secs_f64().max(0.05);
        let drx = rx.saturating_sub(guard.last_rx);
        let dtx = tx.saturating_sub(guard.last_tx);
        ((drx as f64 / dt) as u64, (dtx as f64 / dt) as u64)
    } else {
        (0u64, 0u64)
    };
    guard.last_rx = rx;
    guard.last_tx = tx;
    guard.last_at = Some(now);
    drop(guard);

    let disks_list = Disks::new_with_refreshed_list();
    let mut disks_out: Vec<DiskRow> = Vec::new();
    for d in disks_list.list() {
        if skip_disk(d) {
            continue;
        }
        let total = d.total_space();
        if total == 0 {
            continue;
        }
        let avail = d.available_space();
        let used = total.saturating_sub(avail);
        disks_out.push(DiskRow {
            name: d.name().to_string_lossy().into_owned(),
            mount: d.mount_point().to_string_lossy().into_owned(),
            total_bytes: total,
            used_bytes: used,
            available_bytes: avail,
            removable: d.is_removable(),
        });
    }

    let components = Components::new_with_refreshed_list();
    let (cpu_temp, gpu_temp) = classify_temps(&components);

    let (hz, hz_note) = platform::primary_refresh_hz_hz();
    let primary_refresh_hz = Metric {
        value: hz,
        note: hz_note,
    };

    let (vol, vol_note) = platform::system_volume_percent();
    let volume_percent = Metric {
        value: vol,
        note: vol_note,
    };

    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let mem_pct = if total_mem > 0 {
        (used_mem as f64 / total_mem as f64 * 100.0) as f32
    } else {
        0.0
    };

    let brightness_displays: Vec<BrightnessDisplay> = platform::list_brightness_displays();

    DashboardSnapshot {
        cpu_temp_c: cpu_temp,
        gpu_temp_c: gpu_temp,
        primary_refresh_hz,
        memory_used_bytes: used_mem,
        memory_total_bytes: total_mem,
        memory_percent: mem_pct,
        network_up_bps: up_bps,
        network_down_bps: down_bps,
        disks: disks_out,
        brightness_displays,
        volume_percent,
    }
}

#[tauri::command]
pub fn get_dashboard_snapshot(state: State<'_, Mutex<NetCounters>>) -> DashboardSnapshot {
    build_snapshot(&state)
}

#[tauri::command]
pub fn set_system_volume_cmd(percent: u32) -> Result<(), String> {
    platform::set_system_volume(percent)
}

#[tauri::command]
pub fn set_brightness_cmd(id: String, percent: u32) -> Result<(), String> {
    platform::set_brightness_percent(&id, percent)
}
