import type { DiskRow } from "../types";

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

export default function DiskSection({ disks }: { disks: DiskRow[] }) {
  return (
    <section className="dashboard-card wide">
      <h2>磁盘</h2>
      <div className="flex flex-col gap-0">
        <div className="disk-table-header">
          <span>卷</span>
          <span>挂载点</span>
          <span>可移动</span>
          <span>用量</span>
        </div>
        {disks.map((d) => (
          <div key={`${d.mount}-${d.name}`} className="disk-table-row">
            <span className="truncate" title={d.name}>{d.name}</span>
            <span className="font-mono text-xs">{d.mount}</span>
            <span>{d.removable ? "是" : "否"}</span>
            <span>
              {formatBytes(d.used_bytes)} / {formatBytes(d.total_bytes)}
            </span>
          </div>
        ))}
      </div>
    </section>
  );
}
