//! 平台相关能力：刷新率、音量、亮度（实现按 OS 拆分）。

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "macos")]
pub use macos::{
    list_brightness_displays, primary_refresh_hz_hz, set_brightness_percent, set_system_volume,
    system_volume_percent,
};
#[cfg(windows)]
pub use windows::{
    list_brightness_displays, primary_refresh_hz_hz, set_brightness_percent, set_system_volume,
    system_volume_percent,
};

#[cfg(not(any(target_os = "macos", windows)))]
pub fn primary_refresh_hz_hz() -> (Option<u32>, Option<String>) {
    (None, Some("当前构建目标不支持".to_string()))
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn system_volume_percent() -> (Option<u32>, Option<String>) {
    (None, Some("当前构建目标不支持".to_string()))
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn set_system_volume(_percent: u32) -> Result<(), String> {
    Err("当前构建目标不支持".to_string())
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn list_brightness_displays() -> Vec<crate::snapshot_types::BrightnessDisplay> {
    vec![]
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn set_brightness_percent(_id: &str, _percent: u32) -> Result<(), String> {
    Err("当前构建目标不支持".to_string())
}
