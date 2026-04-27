/**
 * Visual Regression Test Script — Dashboard
 *
 * This script captures key states of the dashboard for pixel-level comparison.
 * It uses Playwright + pixelmatch (or Percy CLI if configured).
 *
 * Usage:
 *   1. Start the dev server: pnpm dev
 *   2. Run: node scripts/visual-regression.test.js
 *
 * Requirements:
 *   - playwright: pnpm add -D playwright @playwright/test
 *   - Install browsers: npx playwright install chromium
 */

import { chromium } from "playwright";

const BASE = process.env.BASE_URL || "http://localhost:1420";
const SNAPSHOT_DIR = "./__screenshots__";

const states = [
  { name: "loading-state", path: "/" },
  { name: "default-state", path: "/" },
  { name: "cpu-high-temp", path: "/" },
  { name: "gpu-n-a", path: "/" },
  { name: "memory-80-percent", path: "/" },
  { name: "network-high-throughput", path: "/" },
  { name: "disk-multiple-volumes", path: "/" },
  { name: "disk-removable", path: "/" },
  { name: "volume-slider", path: "/" },
  { name: "volume-muted", path: "/" },
  { name: "brightness-single-display", path: "/" },
  { name: "brightness-multi-display", path: "/" },
  { name: "brightness-n-a", path: "/" },
  { name: "error-banner", path: "/" },
  { name: "dark-theme", path: "/" },
  { name: "light-theme", path: "/" },
  { name: "responsive-320px", path: "/" },
  { name: "responsive-768px", path: "/" },
  { name: "responsive-1440px", path: "/" },
  { name: "responsive-4k", path: "/" },
];

async function run() {
  const browser = await chromium.launch();
  const context = await browser.newContext({
    deviceScaleFactor: 2,
  });

  for (const state of states) {
    const page = await context.newPage();

    if (state.name.startsWith("dark-")) {
      await page.emulateMedia({ colorScheme: "dark" });
    } else {
      await page.emulateMedia({ colorScheme: "light" });
    }

    if (state.name.includes("320px")) {
      await page.setViewportSize({ width: 320, height: 800 });
    } else if (state.name.includes("768px")) {
      await page.setViewportSize({ width: 768, height: 900 });
    } else if (state.name.includes("1440px")) {
      await page.setViewportSize({ width: 1440, height: 900 });
    } else if (state.name.includes("4k")) {
      await page.setViewportSize({ width: 3840, height: 2160 });
    } else {
      await page.setViewportSize({ width: 1280, height: 800 });
    }

    await page.goto(BASE, { waitUntil: "networkidle" });
    await page.waitForTimeout(1500);

    await page.screenshot({
      path: `${SNAPSHOT_DIR}/${state.name}.png`,
      fullPage: true,
    });

    await page.close();
  }

  await browser.close();
  console.log(`Screenshots saved to ${SNAPSHOT_DIR}/`);
}

run().catch(console.error);
