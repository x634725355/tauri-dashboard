#!/usr/bin/env node

import { writeFileSync } from "node:fs";
import { resolve } from "node:path";

const hexToRgb = (hex) => {
  const h = hex.replace("#", "");
  return {
    r: parseInt(h.substring(0, 2), 16),
    g: parseInt(h.substring(2, 4), 16),
    b: parseInt(h.substring(4, 6), 16),
  };
};

const rgbToHex = (r, g, b) => {
  const toHex = (n) => Math.max(0, Math.min(255, Math.round(n))).toString(16).padStart(2, "0");
  return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
};

const lerp = (a, b, t) => a + (b - a) * t;

const mixRgb = (c1, c2, t) => ({
  r: lerp(c1.r, c2.r, t),
  g: lerp(c1.g, c2.g, t),
  b: lerp(c1.b, c2.b, t),
});

const generateShades = (hex) => {
  const base = hexToRgb(hex);
  const white = hexToRgb("#ffffff");
  const black = hexToRgb("#000000");

  const shades = {};

  const lightSteps = [
    { shade: 50, t: 0.05 },
    { shade: 100, t: 0.15 },
    { shade: 200, t: 0.30 },
    { shade: 300, t: 0.55 },
    { shade: 400, t: 0.75 },
  ];
  for (const { shade, t } of lightSteps) {
    const m = mixRgb(white, base, t);
    shades[shade] = rgbToHex(m.r, m.g, m.b);
  }

  shades[500] = hex;

  const darkSteps = [
    { shade: 600, t: 0.85 },
    { shade: 700, t: 0.65 },
    { shade: 800, t: 0.45 },
    { shade: 900, t: 0.25 },
  ];
  for (const { shade, t } of darkSteps) {
    const m = mixRgb(black, base, t);
    shades[shade] = rgbToHex(m.r, m.g, m.b);
  }

  return shades;
};

const palette = {
  primary: generateShades("#FFE97D"),
  secondary: generateShades("#FFEF9F"),
  accent: generateShades("#E13F7C"),
  neutral: {
    50: "#f8f9fb",
    100: "#eef0f5",
    200: "#d9dde9",
    300: "#b7bfd3",
    400: "#8e99b5",
    500: "#6b7897",
    600: "#55617d",
    700: "#475065",
    800: "#3d4455",
    900: "#363b49",
  },
  success: generateShades("#2dd4a0"),
  warning: generateShades("#FFB74D"),
  error: generateShades("#FF537B"),
  info: generateShades("#5b9aff"),
};

const outputPath = resolve(import.meta.dirname, "..", "src", "theme-palette.json");
writeFileSync(outputPath, JSON.stringify(palette, null, 2), "utf-8");
console.log(`Palette written to ${outputPath}`);
