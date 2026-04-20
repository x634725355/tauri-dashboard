# 桌面系统仪表盘 — 实现说明

> 对应需求：`docs/desktop-system-dashboard-requirements.md`  
> 技术栈：**Tauri 2**（Rust）+ **React 19** + **Vite 7**  
> 本文档描述当前仓库中的**实际实现**、与需求的映射关系及已知边界。

---

## 1. 目标与范围

在单仓库内实现「主窗口仪表盘」：约 **1 次/秒** 刷新系统指标，支持 **Windows** 与 **macOS**；指标缺失时以 **`Metric` 的 `value: null` + `note`** 降级说明（符合需求中的自用容错）。

**不在本文档范围**：移动端、Linux 桌面（未作为目标平台实现）。

---

## 2. 代码结构

| 路径 | 职责 |
|------|------|
| `src-tauri/src/lib.rs` | Tauri 入口：注册插件、`manage` 网络采样状态、注册 `invoke` 命令、启动时尝试开启自启动。 |
| `src-tauri/src/snapshot_types.rs` | `DashboardSnapshot` 及各子结构体的 **serde 序列化** 定义（前后端契约）。 |
| `src-tauri/src/dashboard.rs` | 汇总 `sysinfo` 与 `platform` 结果，计算网速差分，温度启发式分类，暴露 Tauri 命令。 |
| `src-tauri/src/platform/mod.rs` | 平台分发；非 macOS/Windows 目标返回明确「不支持」说明。 |
| `src-tauri/src/platform/macos.rs` | macOS：刷新率、音量（AppleScript）、亮度（IOKit）。 |
| `src-tauri/src/platform/windows.rs` | Windows：刷新率（GDI）、音量（WASAPI）、亮度（`brightness` crate）、可选 GPU 温度（NVML）、远程盘过滤辅助。 |
| `src/App.tsx` | 仪表盘 UI：定时 `invoke` 拉快照，音量/亮度滑条调用设置命令。 |
| `src-tauri/capabilities/default.json` | 能力集：含 `autostart:default` 以允许自启动插件相关权限。 |

---

## 3. Tauri 命令（前后端契约）

| 命令名 | 参数 | 返回 / 行为 |
|--------|------|-------------|
| `get_dashboard_snapshot` | 无 | `DashboardSnapshot` JSON，见下节。 |
| `set_system_volume_cmd` | `percent: u32`（0–100） | `Result<(), String>` |
| `set_brightness_cmd` | `id: String`, `percent: u32` | `Result<(), String>`；`id` 与快照中某块屏的 `BrightnessDisplay.id` 一致。 |

应用状态：`NetCounters` 以 `parking_lot::Mutex` 托管，用于**跨次 invoke**保存上次网络累计字节与时间戳，以计算速率。

---

## 4. 快照模型 `DashboardSnapshot`

定义见 `src-tauri/src/snapshot_types.rs`，字段语义如下：

| 字段 | 含义 |
|------|------|
| `cpu_temp_c` / `gpu_temp_c` | 摄氏度；`Metric` 可带 `note`（例如传感器名称或降级原因）。 |
| `primary_refresh_hz` | 主显示器当前模式的刷新率（Hz），非游戏帧率。 |
| `memory_*` | `sysinfo` 的已用/总量及百分比。 |
| `network_up_bps` / `network_down_bps` | 所有**非环回**网卡累计收发字节的差分速率（字节/秒）；**第一次采样为 0**。 |
| `disks` | 本地与可移动卷；排除网络类文件系统及 Windows 远程映射盘（见实现节）。 |
| `brightness_displays` | 多块屏时多条；`id` 用于 `set_brightness_cmd`。 |
| `volume_percent` | 系统主音量 0–100。 |

---

## 5. 各能力实现要点

### 5.1 温度（CPU / 主 GPU）

- **数据源**：`sysinfo::Components::new_with_refreshed_list()`。
- **CPU**：优先匹配标签中的关键字（如 `cpu`、`package`、`soc`、`die`、`apple m` 等）；有多个候选时取**最高温**作为代表性读数，并在 `note` 中标注传感器名。若无匹配，则在排除 GPU 关键字后取**最高温**启发式，仍无则 `null`。
- **GPU**：优先匹配 GPU 相关关键字（如 `gpu`、`graphics`、`geforce`、`radeon` 等）；**Windows** 上若仍无，则尝试 **`nvml-wrapper`** 读取 **索引 0** 的 NVIDIA GPU 温度，成功则在 `note` 中标明 `NVIDIA NVML`。
- **与需求对齐**：仅展示「一块主 GPU」逻辑——当前实现为**首个匹配的 GPU 传感器**或 **NVML device 0**，非多 GPU 列表。

### 5.2 主显示器刷新率（Hz）

- **macOS**：`core-graphics` 的 `CGDisplay::main()` + `display_mode()` + `refresh_rate()`；若为 0 则记为不可用（可变刷新率或未报告等情况）。
- **Windows**：`EnumDisplaySettingsW` + `ENUM_CURRENT_SETTINGS` 读取 `dmDisplayFrequency`。

### 5.3 内存占比

- `System::new()` + `refresh_memory()`，使用 `used_memory` / `total_memory` 计算百分比（与部分系统监视器可能存在小幅口径差异，需求已允许）。

### 5.4 磁盘列表

- `Disks::new_with_refreshed_list()`。
- **排除网络卷**：文件系统名含 `nfs` / `smb` / `cifs` / `webdav` / `afp` 等则跳过；挂载路径含 `//` 的启发式跳过。
- **Windows 额外**：对形如 `X:\` 的根路径调用 `GetDriveTypeW`，**`DRIVE_REMOTE`** 视为网络映射盘并跳过（与需求「不含网络映射盘」一致）。
- **可移动**：保留 `sysinfo` 的 `is_removable()` 字段供 UI 展示。

### 5.5 网络上下行（系统总吞吐）

- 遍历 `Networks`，跳过环回接口（名称含 `loopback` 或以 `lo` 开头等启发式）。
- 对 `total_received` / `total_transmitted` **全量求和**，用 `NetCounters` 与上次快照做差除以时间间隔得 **B/s**。

### 5.6 系统主音量

- **macOS**：`osascript` 读取/设置 `output volume`。
- **Windows**：COM `CoInitializeEx` + `MMDeviceEnumerator` 默认播放端点 + `IAudioEndpointVolume` 的标量音量读写。

### 5.7 屏幕亮度（多屏尽量覆盖）

- **macOS**：IOKit，匹配服务 **`IODisplayConnect`** 与 **`AppleBacklightDisplay`**；对支持 `IODisplayGetFloatParameter` / `IODisplaySetFloatParameter`（`brightness` 键）的设备生成条目。`id` 格式为 **`服务名|枚举序号`**，设置时按相同枚举次序重匹配。
- **Windows**：`brightness` crate 的 **blocking** 设备迭代；`id` 为 **`win|序号`**。
- 无设备时返回占位条目（`id: none`），前端不渲染滑条。

### 5.8 登录自启动

- 插件：`tauri-plugin-autostart`，macOS 使用 **`MacosLauncher::LaunchAgent`**（与 `init` 一致）。
- 在 `.setup` 中调用 `app.autolaunch().enable()`；失败仅 **`eprintln!`**，不阻止应用启动（避免未签名包等环境直接崩溃）。
- 需在 `capabilities/default.json` 中声明 **`autostart:default`**。

---

## 6. 前端行为

- 进入页面后立即拉取一次快照，之后 **`setInterval(..., 1000)`** 与需求「约 1Hz」一致。
- 音量与各显示器亮度使用 **受控** `input[type=range]`，变更时调用对应 `invoke` 并再次刷新快照。
- 样式：`src/App.css`，支持浅色/暗色偏好。

---

## 7. 已知限制与风险（验收时请关注）

1. **温度读数**强依赖系统与驱动；Windows 上 WMI 可能只有「整机」类单值；非 NVIDIA 或无 NVML 时 GPU 常为 **N/A**。
2. **网速**第一次为 0；极短间隔内调用 `get_dashboard_snapshot` 也会使速率波动。
3. **macOS 外接屏亮度**可能始终不可用（系统/硬件限制），与需求文档中的「多屏尽量覆盖 + 接受不可用」一致。
4. **自启动**在 macOS 上受签名、路径、用户「登录项」策略影响；失败时需用户在本机「系统设置」中排查。
5. **Windows 实现**在开发机以外未做完整 CI 验证；若 `cargo tauri build` 报链接或 feature 错误，需按目标机环境补全 `windows` crate 特性或 SDK。

---

## 8. 本地构建与运行

```bash
pnpm install
pnpm tauri dev
```

发布构建：

```bash
pnpm tauri build
```

---

## 9. 修订记录

| 日期 | 说明 |
|------|------|
| 2026-04-20 | 初版：与当前仓库实现同步归档。 |
