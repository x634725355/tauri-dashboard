//! 前后端共用的快照数据结构（serde 输出给前端）。

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Metric<T> {
    pub value: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrightnessDisplay {
    pub id: String,
    pub label: String,
    pub percent: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskRow {
    pub name: String,
    pub mount: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub removable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSnapshot {
    pub cpu_temp_c: Metric<f32>,
    pub gpu_temp_c: Metric<f32>,
    pub primary_refresh_hz: Metric<u32>,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub memory_percent: f32,
    pub network_up_bps: u64,
    pub network_down_bps: u64,
    pub disks: Vec<DiskRow>,
    pub brightness_displays: Vec<BrightnessDisplay>,
    pub volume_percent: Metric<u32>,
}
