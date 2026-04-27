import type { Metric } from "../types";
import MetricLine from "./MetricLine";

export default function CpuGpuSection({
  cpuTemp,
  gpuTemp,
  refreshHz,
}: {
  cpuTemp: Metric<number>;
  gpuTemp: Metric<number>;
  refreshHz: Metric<number>;
}) {
  return (
    <section className="dashboard-card">
      <h2>温度与显示</h2>
      <MetricLine label="CPU 温度" metric={cpuTemp} unit=" °C" decimals={1} />
      <MetricLine label="主 GPU 温度" metric={gpuTemp} unit=" °C" decimals={1} />
      <MetricLine label="主显示器刷新率" metric={refreshHz} unit=" Hz" />
    </section>
  );
}
