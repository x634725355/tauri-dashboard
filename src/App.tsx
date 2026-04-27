import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import type { DashboardSnapshot } from "./types";
import ErrorBanner from "./components/ErrorBanner";
import CpuGpuSection from "./components/CpuGpuSection";
import MemorySection from "./components/MemorySection";
import NetworkSection from "./components/NetworkSection";
import DiskSection from "./components/DiskSection";
import VolumeSection from "./components/VolumeSection";
import BrightnessSection from "./components/BrightnessSection";

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
    <div className="min-h-screen bg-neutral-50 px-5 pb-10 pt-5 text-neutral-900
                    dark:bg-neutral-900 dark:text-neutral-100">
      <header>
        <h1 className="m-0 text-[1.45rem] font-semibold leading-tight tracking-tight">
          系统仪表盘
        </h1>
        <p className="m-0 mt-1 text-sm text-neutral-400 dark:text-neutral-500">
          约 1 秒刷新 · Windows / macOS
        </p>
      </header>

      {err ? <ErrorBanner message={err} /> : null}

      {!snap ? (
        <p className="mt-4 text-neutral-400 dark:text-neutral-500">正在读取…</p>
      ) : (
        <div className="mt-4 grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
          <CpuGpuSection
            cpuTemp={snap.cpu_temp_c}
            gpuTemp={snap.gpu_temp_c}
            refreshHz={snap.primary_refresh_hz}
          />
          <MemorySection
            usedBytes={snap.memory_used_bytes}
            totalBytes={snap.memory_total_bytes}
            percent={snap.memory_percent}
          />
          <NetworkSection upBps={snap.network_up_bps} downBps={snap.network_down_bps} />
          <DiskSection disks={snap.disks} />
          <VolumeSection volume={snap.volume_percent} onChange={onVolume} />
          <BrightnessSection displays={snap.brightness_displays} onChange={onBrightness} />
        </div>
      )}
    </div>
  );
}
