//! Windows：默认音频端点音量、主显示器刷新率、物理显示器亮度、驱动器类型过滤。

use crate::snapshot_types::BrightnessDisplay;
use brightness::blocking::{Brightness, BrightnessDevice};
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::Nvml;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::Once;
use windows::core::Interface;
use windows::Win32::Graphics::Gdi::{
    DEVMODEW, ENUM_CURRENT_SETTINGS, EnumDisplaySettingsW,
};
use windows::core::PCWSTR;
use windows::Win32::Media::Audio::{
    eConsole, eRender, Endpoints::IAudioEndpointVolume, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::Storage::FileSystem::{GetDriveTypeW, DRIVE_REMOTE};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};

static COM_INIT: Once = Once::new();

fn ensure_com() -> Result<(), String> {
    let mut err = Ok(());
    COM_INIT.call_once(|| {
        let r = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if r.is_err() {
            err = Err(format!("CoInitializeEx 失败: {r:?}"));
        }
    });
    err
}

pub fn system_volume_percent() -> (Option<u32>, Option<String>) {
    if let Err(e) = ensure_com() {
        return (None, Some(e));
    }
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                Ok(e) => e,
                Err(x) => return (None, Some(format!("创建设备枚举器失败: {x:?}"))),
            };
        let device = match enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
            Ok(d) => d,
            Err(x) => return (None, Some(format!("无默认播放设备: {x:?}"))),
        };
        let volume: IAudioEndpointVolume = match device.Activate(CLSCTX_ALL, None) {
            Ok(v) => v,
            Err(x) => return (None, Some(format!("无法激活音量接口: {x:?}"))),
        };
        let scalar = match volume.GetMasterVolumeLevelScalar() {
            Ok(s) => s,
            Err(x) => return (None, Some(format!("读取音量失败: {x:?}"))),
        };
        let pct = (scalar * 100.0).round().clamp(0.0, 100.0) as u32;
        (Some(pct), None)
    }
}

pub fn set_system_volume(percent: u32) -> Result<(), String> {
    ensure_com()?;
    let level = (percent.min(100) as f32 / 100.0).clamp(0.0, 1.0);
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| format!("创建设备枚举器失败: {e:?}"))?;
        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .map_err(|e| format!("无默认播放设备: {e:?}"))?;
        let volume: IAudioEndpointVolume = device
            .Activate(CLSCTX_ALL, None)
            .map_err(|e| format!("无法激活音量接口: {e:?}"))?;
        volume
            .SetMasterVolumeLevelScalar(level, std::ptr::null())
            .map_err(|e| format!("设置音量失败: {e:?}"))?;
    }
    Ok(())
}

pub fn primary_refresh_hz_hz() -> (Option<u32>, Option<String>) {
    let mut dev = DEVMODEW {
        dmSize: std::mem::size_of::<DEVMODEW>() as u16,
        ..Default::default()
    };
    let ok = unsafe { EnumDisplaySettingsW(PCWSTR::null(), ENUM_CURRENT_SETTINGS, &mut dev) };
    if !ok.as_bool() {
        return (None, Some("EnumDisplaySettingsW 失败".into()));
    }
    let hz = dev.dmDisplayFrequency as u32;
    if hz == 0 || hz == 1 {
        return (
            None,
            Some("dmDisplayFrequency 为 0/1（驱动未报告或可变刷新率）".into()),
        );
    }
    (Some(hz), None)
}

pub fn list_brightness_displays() -> Vec<BrightnessDisplay> {
    let mut out = Vec::new();
    let mut idx = 0usize;
    for res in brightness::blocking::brightness_devices() {
        match res {
            Ok(dev) => match collect_one_display(&dev, idx) {
                Ok(row) => {
                    out.push(row);
                    idx += 1;
                }
                Err(_) => {}
            },
            Err(_) => {}
        }
    }
    if out.is_empty() {
        out.push(BrightnessDisplay {
            id: "none".into(),
            label: "未找到可调亮度显示器".into(),
            percent: None,
            note: Some("部分外接屏需 DDC/驱动支持".into()),
        });
    }
    out
}

fn collect_one_display(dev: &BrightnessDevice, idx: usize) -> Result<BrightnessDisplay, brightness::Error> {
    let name = dev.friendly_device_name()?;
    let pct = dev.get()?;
    Ok(BrightnessDisplay {
        id: format!("win|{idx}"),
        label: name,
        percent: Some(pct),
        note: None,
    })
}

pub fn set_brightness_percent(id: &str, percent: u32) -> Result<(), String> {
    if id == "none" {
        return Err("无可调亮度设备".into());
    }
    let idx: usize = id
        .strip_prefix("win|")
        .ok_or_else(|| "无效的亮度 id".to_string())?
        .parse()
        .map_err(|_| "无效的亮度序号".to_string())?;
    let mut i = 0usize;
    for res in brightness::blocking::brightness_devices() {
        let mut dev = res.map_err(|e| e.to_string())?;
        if i == idx {
            dev.set(percent.min(100)).map_err(|e| e.to_string())?;
            return Ok(());
        }
        i += 1;
    }
    Err("未找到对应亮度设备".into())
}

/// 尝试读取 NVIDIA 独显 GPU 温度（摄氏度）；无 NVIDIA 或驱动不可用时返回 `None`。
pub fn try_nvidia_gpu_temp_c() -> Option<f32> {
    let nvml = Nvml::init().ok()?;
    let dev = nvml.device_by_index(0).ok()?;
    dev.temperature(TemperatureSensor::Gpu).ok().map(|t| t as f32)
}

/// Windows 下排除网络映射盘（`GetDriveTypeW == DRIVE_REMOTE`）。
pub fn is_network_drive(mount: &std::path::Path) -> bool {
    let Some(s) = mount.to_str() else {
        return false;
    };
    let root = if s.len() >= 2 {
        let b0 = s.as_bytes()[0];
        let b1 = s.as_bytes()[1];
        if b1 == b':' && b0.is_ascii_alphabetic() {
            format!("{}:\\", s.chars().next().unwrap())
        } else {
            return false;
        }
    } else {
        return false;
    };
    let wide: Vec<u16> = OsStr::new(&root).encode_wide().chain(Some(0)).collect();
    let t = unsafe { GetDriveTypeW(windows::core::PCWSTR(wide.as_ptr())) };
    t == DRIVE_REMOTE
}
