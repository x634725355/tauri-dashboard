import type { Metric } from "../types";

export default function MetricLine({
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
  const v = metric.value == null ? "—" : metric.value.toFixed(decimals) + unit;
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
