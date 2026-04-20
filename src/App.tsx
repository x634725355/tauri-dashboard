import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import "./App.css";

type Metric<T> = { value: T | null; note?: string };

type BrightnessDisplay = {
  id: string;
  label: string;
  percent: number | null;
  note?: string;
};

type DiskRow = {
  name: string;
  mount: string;
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  removable: boolean;
};

type DashboardSnapshot = {
  cpu_temp_c: Metric<number>;
  gpu_temp_c: Metric<number>;
  primary_refresh_hz: Metric<number>;
  memory_used_bytes: number;
  memory_total_bytes: number;
  memory_percent: number;
  network_up_bps: number;
  network_down_bps: number;
  disks: DiskRow[];
  brightness_displays: BrightnessDisplay[];
  volume_percent: Metric<number>;
};

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const u = ["KB", "MB", "GB", "TB"];
  let v = n;
  let i = 0;
  while (v >= 1024 && i < u.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(i === 0 ? 0 : 1)} ${u[i]}`;
}

function formatBps(n: number): string {
  if (n < 1000) return `${n} B/s`;
  const u = ["KB/s", "MB/s", "GB/s"];
  let v = n;
  let i = 0;
  while (v >= 1000 && i < u.length - 1) {
    v /= 1000;
    i++;
  }
  return `${v.toFixed(1)} ${u[i]}`;
}

function MetricLine({
  label,
  metric,
  unit,
  decimals = 0,
}: {
  label: string;
  metric: Metric<number>;
  unit: string;
  decimals?: number;
}) {
  const v =
    metric.value == null ? "—" : metric.value.toFixed(decimals) + unit;
  return (
    <div className="metric-row">
      <span className="metric-label">{label}</span>
      <span className="metric-value">{v}</span>
      {metric.note ? (
        <span className="metric-note" title={metric.note}>
          {metric.note}
        </span>
      ) : null}
    </div>
  );
}

export default function App() {
  const [snap, setSnap] = useState<DashboardSnapshot | null>(null);
  const [err, setErr] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const data = await invoke<DashboardSnapshot>("get_dashboard_snapshot");
      setSnap(data);
      setErr(null);
    } catch (e) {
      setErr(String(e));
    }
  }, []);

  useEffect(() => {
    void refresh();
    const id = window.setInterval(() => void refresh(), 1000);
    return () => window.clearInterval(id);
  }, [refresh]);

  const onVolume = async (v: number) => {
    try {
      await invoke("set_system_volume_cmd", { percent: v });
      void refresh();
    } catch (e) {
      setErr(String(e));
    }
  };

  const onBrightness = async (id: string, v: number) => {
    try {
      await invoke("set_brightness_cmd", { id, percent: v });
      void refresh();
    } catch (e) {
      setErr(String(e));
    }
  };

  return (
    <div className="app">
      <header className="header">
        <h1>系统仪表盘</h1>
        <p className="sub">约 1 秒刷新 · Windows / macOS</p>
      </header>

      {err ? <div className="banner error">{err}</div> : null}

      {!snap ? (
        <p className="muted">正在读取…</p>
      ) : (
        <div className="grid">
          <section className="card">
            <h2>温度与显示</h2>
            <MetricLine
              label="CPU 温度"
              metric={snap.cpu_temp_c}
              unit=" °C"
              decimals={1}
            />
            <MetricLine
              label="主 GPU 温度"
              metric={snap.gpu_temp_c}
              unit=" °C"
              decimals={1}
            />
            <MetricLine
              label="主显示器刷新率"
              metric={snap.primary_refresh_hz}
              unit=" Hz"
              decimals={0}
            />
          </section>

          <section className="card">
            <h2>内存</h2>
            <div className="mem-bar-wrap">
              <div
                className="mem-bar"
                style={{ width: `${Math.min(100, snap.memory_percent)}%` }}
              />
            </div>
            <div className="metric-row">
              <span className="metric-label">占用</span>
              <span className="metric-value">
                {snap.memory_percent.toFixed(1)}% ·{" "}
                {formatBytes(snap.memory_used_bytes)} /{" "}
                {formatBytes(snap.memory_total_bytes)}
              </span>
            </div>
          </section>

          <section className="card">
            <h2>网络（全接口合计）</h2>
            <div className="metric-row">
              <span className="metric-label">下行</span>
              <span className="metric-value">
                {formatBps(snap.network_down_bps)}
              </span>
            </div>
            <div className="metric-row">
              <span className="metric-label">上行</span>
              <span className="metric-value">
                {formatBps(snap.network_up_bps)}
              </span>
            </div>
            <p className="muted small">
              首次采样速率为 0；约 1 秒后显示增量估算。
            </p>
          </section>

          <section className="card wide">
            <h2>磁盘</h2>
            <div className="disk-table">
              <div className="disk-head">
                <span>卷</span>
                <span>挂载点</span>
                <span>可移动</span>
                <span>用量</span>
              </div>
              {snap.disks.map((d) => (
                <div key={`${d.mount}-${d.name}`} className="disk-row">
                  <span title={d.name}>{d.name}</span>
                  <span className="mono">{d.mount}</span>
                  <span>{d.removable ? "是" : "否"}</span>
                  <span>
                    {formatBytes(d.used_bytes)} / {formatBytes(d.total_bytes)}
                  </span>
                </div>
              ))}
            </div>
          </section>

          <section className="card">
            <h2>系统主音量</h2>
            <div className="metric-row">
              <span className="metric-label">当前</span>
              <span className="metric-value">
                {snap.volume_percent.value ?? "—"}%
              </span>
              {snap.volume_percent.note ? (
                <span className="metric-note" title={snap.volume_percent.note}>
                  {snap.volume_percent.note}
                </span>
              ) : null}
            </div>
            <label className="slider-label">
              <input
                type="range"
                min={0}
                max={100}
                value={snap.volume_percent.value ?? 0}
                onChange={(e) => onVolume(Number(e.target.value))}
              />
            </label>
          </section>

          <section className="card wide">
            <h2>屏幕亮度（按显示器）</h2>
            {snap.brightness_displays.map((b) => (
              <div key={b.id} className="brightness-block">
                <div className="metric-row">
                  <span className="metric-label">{b.label}</span>
                  <span className="metric-value">
                    {b.percent == null ? "—" : `${b.percent}%`}
                  </span>
                  {b.note ? (
                    <span className="metric-note" title={b.note}>
                      {b.note}
                    </span>
                  ) : null}
                </div>
                {b.id !== "none" && b.percent != null ? (
                  <label className="slider-label">
                    <input
                      type="range"
                      min={0}
                      max={100}
                      value={b.percent}
                      onChange={(e) =>
                        onBrightness(b.id, Number(e.target.value))
                      }
                    />
                  </label>
                ) : null}
              </div>
            ))}
          </section>
        </div>
      )}
    </div>
  );
}
