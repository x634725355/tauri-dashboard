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

export default function MemorySection({
  usedBytes,
  totalBytes,
  percent,
}: {
  usedBytes: number;
  totalBytes: number;
  percent: number;
}) {
  return (
    <section className="dashboard-card">
      <h2>内存</h2>
      <div className="progress-bar-track">
        <div className="progress-bar-fill" style={{ width: `${Math.min(100, percent)}%` }} />
      </div>
      <div className="metric-row">
        <span className="metric-label">占用</span>
        <span className="metric-value">
          {percent.toFixed(1)}% · {formatBytes(usedBytes)} / {formatBytes(totalBytes)}
        </span>
      </div>
    </section>
  );
}
