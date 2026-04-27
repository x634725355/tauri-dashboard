import type { Metric } from "../types";

export default function VolumeSection({
  volume,
  onChange,
}: {
  volume: Metric<number>;
  onChange: (value: number) => void;
}) {
  return (
    <section className="dashboard-card">
      <h2>系统主音量</h2>
      <div className="metric-row">
        <span className="metric-label">当前</span>
        <span className="metric-value">
          {volume.value ?? "—"}%
        </span>
        {volume.note ? (
          <span className="metric-note" title={volume.note}>
            {volume.note}
          </span>
        ) : null}
      </div>
      <label className="mt-2.5 block">
        <input
          type="range"
          min={0}
          max={100}
          value={volume.value ?? 0}
          onChange={(e) => onChange(Number(e.target.value))}
          className="w-full cursor-pointer accent-accent-500
                     focus:outline-none focus:ring-2 focus:ring-accent-300 focus:ring-offset-2
                     dark:focus:ring-offset-neutral-800"
          aria-label="系统主音量"
        />
      </label>
    </section>
  );
}
