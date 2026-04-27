# 桌面系统仪表盘

Tauri 2 + React 19 + Vite 7 构建的系统监控仪表盘，支持 Windows 与 macOS。

指标包括：CPU/GPU 温度、显示器刷新率、内存占比、磁盘容量、网络吞吐、系统音量与屏幕亮度控制。

---

## 快速开始

```bash
pnpm install
pnpm tauri dev
```

生产构建：

```bash
pnpm tauri build
```

---

## 主题定制

仪表盘使用 **Tailwind CSS 3**，通过 `tailwind.config.js` 中的自定义颜色变量实现主题化。

### 颜色系统

| Token | 用途 | 默认色 |
|-------|------|--------|
| `primary` | 主色（品牌色、标题高亮） | `#FFE97D` |
| `secondary` | 辅色（次要元素、背景区块） | `#FFEF9F` |
| `accent` | 强调色（滑条、交互控件） | `#E13F7C` |
| `neutral` | 中性色（文字、边框、背景） | 灰阶色板 |
| `success` | 成功状态 | `#2dd4a0` |
| `warning` | 警告状态 | `#FFB74D` |
| `error` | 错误/危险状态 | `#FF537B` |
| `info` | 信息提示 | `#5b9aff` |

每个颜色均包含 `50`–`900` 共 10 个色阶，`500` 为基准色。

### 替换参考图片并重新生成色阶

1. 从参考图片中提取主色、辅色、强调色与功能色的十六进制值。
2. 修改 `scripts/generate-palette.js` 中的基准色值。
3. 运行脚本生成色板并自动更新 `src/theme-palette.json`：

```bash
node scripts/generate-palette.js
```

4. 将 `tailwind.config.js` 中对应颜色的 `50`–`900` 色阶替换为脚本输出的新值。

### 在组件中使用主题色

```tsx
{/* 主色背景 */}
<div className="bg-primary-500 text-white">...</div>

{/* 强调色边框与文字 */}
<button className="border-accent-500 text-accent-600 hover:bg-accent-50">...</button>

{/* 功能色 */}
<span className="text-error-500">错误提示</span>
<span className="text-success-600 bg-success-100">操作成功</span>
```

深色模式自动通过 `dark:` 前缀适配：

```tsx
<div className="bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100">
  ...
</div>
```

### 设计规范

- 文字与背景对比度遵循 **WCAG 2.1 AA** 标准（≥ 4.5:1 常规文本，≥ 3:1 大文本）。
- 浅色/深色模式通过 `@media (prefers-color-scheme: dark)` 自动切换。
- 所有交互控件具备 `focus:ring` 焦点环，支持键盘导航与屏幕阅读器。

---

## 项目结构

```
src/
├── components/
│   ├── MetricLine.tsx          # 指标行（标签 + 值 + 备注）
│   ├── CpuGpuSection.tsx       # 温度与显示卡片
│   ├── MemorySection.tsx       # 内存卡片（进度条 + 数值）
│   ├── NetworkSection.tsx      # 网络吞吐卡片
│   ├── DiskSection.tsx         # 磁盘列表卡片
│   ├── VolumeSection.tsx       # 音量滑条卡片
│   ├── BrightnessSection.tsx   # 多屏亮度控制卡片
│   └── ErrorBanner.tsx         # 错误横幅
├── types.ts                    # TypeScript 类型定义
├── index.css                   # Tailwind 入口 + @apply 组件类
├── App.tsx                     # 主布局与数据获取
├── main.tsx                    # React 入口
└── theme-palette.json          # 导出的色板 JSON
scripts/
├── generate-palette.js         # 主题色生成脚本
└── visual-regression.test.js   # 可视化回归测试
tailwind.config.js              # Tailwind 配置（含自定义色板）
postcss.config.js               # PostCSS 配置
```

---

## 可视化回归测试

```bash
# 安装 Playwright
pnpm add -D playwright @playwright/test
npx playwright install chromium

# 启动开发服务器后运行
pnpm dev &
node scripts/visual-regression.test.js
```

测试覆盖 20 个关键状态：加载态、默认态、各项指标正常/异常、明暗主题、4 种分辨率（320px / 768px / 1440px / 4K）。

---

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2 |
| 前端框架 | React 19 |
| 构建工具 | Vite 7 |
| CSS 框架 | Tailwind CSS 3 |
| 后端语言 | Rust |

## 推荐 IDE 配置

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
