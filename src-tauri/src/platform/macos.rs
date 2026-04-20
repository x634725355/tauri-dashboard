//! macOS：主屏刷新率、系统音量（AppleScript）、亮度（IOKit）。

use crate::snapshot_types::BrightnessDisplay;
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_graphics::display::CGDisplay;
use mach2::kern_return::{kern_return_t, KERN_SUCCESS};
use std::ffi::CString;
use std::io::Read;
use std::process::{Command, Stdio};

type IoObject = u32;
type IoIterator = u32;

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOServiceMatching(name: *const libc::c_char) -> *mut libc::c_void;
    fn IOServiceGetMatchingServices(
        master_port: libc::mach_port_t,
        matching: *mut libc::c_void,
        existing: *mut IoIterator,
    ) -> kern_return_t;
    fn IOIteratorNext(iterator: IoIterator) -> IoObject;
    fn IOObjectRelease(obj: IoObject) -> kern_return_t;
    fn IODisplayGetFloatParameter(
        service: IoObject,
        flags: i32,
        key: core_foundation::string::CFStringRef,
        value: *mut f32,
    ) -> kern_return_t;
    fn IODisplaySetFloatParameter(
        service: IoObject,
        flags: i32,
        key: core_foundation::string::CFStringRef,
        value: f32,
    ) -> kern_return_t;
}

const K_IO_MASTER_PORT_DEFAULT: libc::mach_port_t = 0;

fn run_osascript(script: &str) -> Result<String, String> {
    let mut child = Command::new("osascript")
        .args(["-e", script])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("无法启动 osascript: {e}"))?;
    let mut out = String::new();
    child
        .stdout
        .take()
        .ok_or_else(|| "osascript 无 stdout".to_string())?
        .read_to_string(&mut out)
        .map_err(|e| format!("读取 osascript 输出失败: {e}"))?;
    let status = child.wait().map_err(|e| format!("osascript 退出异常: {e}"))?;
    if !status.success() {
        return Err(format!("osascript 失败 (code={:?})", status.code()));
    }
    Ok(out.trim().to_string())
}

pub fn system_volume_percent() -> (Option<u32>, Option<String>) {
    match run_osascript("output volume of (get volume settings)") {
        Ok(s) => match s.parse::<u32>() {
            Ok(v) if v <= 100 => (Some(v), None),
            Ok(v) => (None, Some(format!("音量值异常: {v}"))),
            Err(_) => (None, Some(format!("无法解析音量: {s}"))),
        },
        Err(e) => (None, Some(e)),
    }
}

pub fn set_system_volume(percent: u32) -> Result<(), String> {
    let p = percent.min(100);
    run_osascript(&format!("set volume output volume {p}"))?;
    Ok(())
}

pub fn primary_refresh_hz_hz() -> (Option<u32>, Option<String>) {
    let main = CGDisplay::main();
    let mode = match main.display_mode() {
        Some(m) => m,
        None => return (None, Some("无法读取当前显示模式".to_string())),
    };
    let hz = mode.refresh_rate().round() as u32;
    if hz == 0 {
        return (
            None,
            Some("刷新率为 0（可能是可变刷新率或未报告）".to_string()),
        );
    }
    (Some(hz), None)
}

fn brightness_key() -> CFString {
    CFString::new("brightness")
}

fn brightness_for_service(svc: &str, out: &mut Vec<BrightnessDisplay>) {
    let Ok(cname) = CString::new(svc) else {
        return;
    };
    let matching = unsafe { IOServiceMatching(cname.as_ptr()) };
    if matching.is_null() {
        return;
    }
    let mut iter: IoIterator = 0;
    let kr = unsafe {
        IOServiceGetMatchingServices(K_IO_MASTER_PORT_DEFAULT, matching, &mut iter)
    };
    if kr != KERN_SUCCESS || iter == 0 {
        if iter != 0 {
            unsafe {
                let _ = IOObjectRelease(iter);
            }
        }
        return;
    }
    let key = brightness_key();
    let key_ref = key.as_concrete_TypeRef();
    let mut idx = 0usize;
    loop {
        let obj = unsafe { IOIteratorNext(iter) };
        if obj == 0 {
            break;
        }
        let mut v: f32 = 0.0;
        let kr = unsafe { IODisplayGetFloatParameter(obj, 0, key_ref, &mut v) };
        if kr == KERN_SUCCESS {
            let pct = (v.clamp(0.0, 1.0) * 100.0).round() as u32;
            out.push(BrightnessDisplay {
                id: format!("{svc}|{idx}"),
                label: format!("{svc} #{idx}"),
                percent: Some(pct),
                note: None,
            });
        }
        unsafe {
            let _ = IOObjectRelease(obj);
        }
        idx += 1;
    }
    unsafe {
        let _ = IOObjectRelease(iter);
    }
}

pub fn list_brightness_displays() -> Vec<BrightnessDisplay> {
    let mut list = Vec::new();
    brightness_for_service("IODisplayConnect", &mut list);
    brightness_for_service("AppleBacklightDisplay", &mut list);
    if list.is_empty() {
        list.push(BrightnessDisplay {
            id: "none".into(),
            label: "未找到可调亮度设备".into(),
            percent: None,
            note: Some("外接显示器在 macOS 上常需 DDC/专用驱动".into()),
        });
    }
    list
}

pub fn set_brightness_percent(id: &str, percent: u32) -> Result<(), String> {
    if id == "none" {
        return Err("无可调亮度设备".to_string());
    }
    let (svc, idx_str) = id
        .rsplit_once('|')
        .ok_or_else(|| "无效的亮度设备 id".to_string())?;
    let ordinal: usize = idx_str
        .parse()
        .map_err(|_| "无效的亮度设备序号".to_string())?;
    let cname = CString::new(svc).map_err(|_| "无效的服务名".to_string())?;
    let matching = unsafe { IOServiceMatching(cname.as_ptr()) };
    if matching.is_null() {
        return Err("IOServiceMatching 失败".to_string());
    }
    let mut iter: IoIterator = 0;
    let kr = unsafe {
        IOServiceGetMatchingServices(K_IO_MASTER_PORT_DEFAULT, matching, &mut iter)
    };
    if kr != KERN_SUCCESS {
        return Err(format!("IOServiceGetMatchingServices 失败: {kr}"));
    }
    let key = brightness_key();
    let key_ref = key.as_concrete_TypeRef();
    let target = (percent.min(100) as f32) / 100.0;
    let mut i = 0usize;
    let mut done = false;
    loop {
        let obj = unsafe { IOIteratorNext(iter) };
        if obj == 0 {
            break;
        }
        if i == ordinal {
            let kr = unsafe { IODisplaySetFloatParameter(obj, 0, key_ref, target) };
            unsafe {
                let _ = IOObjectRelease(obj);
            }
            if kr != KERN_SUCCESS {
                return Err(format!("IODisplaySetFloatParameter 失败: {kr}"));
            }
            done = true;
            // 排空迭代器，避免悬挂的 IO 对象
            loop {
                let rest = unsafe { IOIteratorNext(iter) };
                if rest == 0 {
                    break;
                }
                unsafe {
                    let _ = IOObjectRelease(rest);
                }
            }
            break;
        }
        unsafe {
            let _ = IOObjectRelease(obj);
        }
        i += 1;
    }
    unsafe {
        let _ = IOObjectRelease(iter);
    }
    if done {
        Ok(())
    } else {
        Err("未找到对应显示器（可能已热插拔）".to_string())
    }
}
