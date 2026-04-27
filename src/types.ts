export type Metric<T> = { value: T | null; note?: string };

export type BrightnessDisplay = {
  id: string;
  label: string;
  percent: number | null;
  note?: string;
};

export type DiskRow = {
  name: string;
  mount: string;
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  removable: boolean;
};

export type DashboardSnapshot = {
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
