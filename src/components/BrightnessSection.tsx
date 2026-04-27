import type { BrightnessDisplay } from "../types";

export default function BrightnessSection({
  displays,
  onChange,
}: {
  displays: BrightnessDisplay[];
  onChange: (id: string, value: number) => void;
}) {
  return (
    <section className="dashboard-card wide">
      <h2>屏幕亮度（按显示器）</h2>
      {displays.map((b) => (
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
            <label className="mt-2 block">
              <input
                type="range"
                min={0}
                max={100}
                value={b.percent}
                onChange={(e) => onChange(b.id, Number(e.target.value))}
                className="w-full cursor-pointer accent-accent-500
                           focus:outline-none focus:ring-2 focus:ring-accent-300 focus:ring-offset-2
                           dark:focus:ring-offset-neutral-800"
                aria-label={`${b.label} 亮度`}
              />
            </label>
          ) : null}
        </div>
      ))}
    </section>
  );
}
