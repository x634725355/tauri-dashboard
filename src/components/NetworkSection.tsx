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

export default function NetworkSection({
  upBps,
  downBps,
}: {
  upBps: number;
  downBps: number;
}) {
  return (
    <section className="dashboard-card">
      <h2>网络（全接口合计）</h2>
      <div className="metric-row">
        <span className="metric-label">下行</span>
        <span className="metric-value">{formatBps(downBps)}</span>
      </div>
      <div className="metric-row">
        <span className="metric-label">上行</span>
        <span className="metric-value">{formatBps(upBps)}</span>
      </div>
      <p className="mt-2 text-xs text-neutral-400 dark:text-neutral-500">
        首次采样速率为 0；约 1 秒后显示增量估算。
      </p>
    </section>
  );
}
