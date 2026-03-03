---
title: UI Style Guide
label: kroki-rs-nxt.developer-guide.ui-style-guide
---

# UI Style Guide

This guide defines the shared visual direction for web-facing surfaces (`apps/web-app`, server playground, desktop web UI).

## Visual Direction

- Tone: futuristic, professional, high-tech.
- Default theme: deep-space dark mode.
- Support light theme as a secondary option.

## Color Strategy (60-30-10)

Use colors with asymmetric weight to preserve readability and hierarchy.

- `60%` Primary background: deep bluish-purple.
  - Base: `#120f2d` (or `#1a1b4b` / `#240046` by context).
- `30%` Secondary UI/gradient support: orangish yellow.
  - Base: `#ffb30f`.
- `10%` Accent/action: flamingo red.
  - Base: `#ff6f61`.

## Token Baseline

```css
:root {
  --bg-deep: #120f2d;
  --bg-card: rgba(45, 40, 85, 0.4);
  --accent-primary: #ff6f61;
  --accent-secondary: #ffb30f;
  --grad-main: linear-gradient(135deg, var(--accent-secondary), var(--accent-primary));
  --glass-surface: blur(10px) saturate(180%);
  --glow: 0 0 20px rgba(255, 111, 97, 0.5);
}
```

## Core Components

- Glow cards:
  - Glassmorphism surface + rounded corners (`12-16px`) + subtle gradient borders.
- Gradient headers:
  - Clip `--grad-main` inside headline text.
- Primary CTA:
  - Pill shape, flamingo red fill, glow on hover/focus.
- Secondary CTA:
  - Transparent/ghost with orangish-yellow border.
- Floating navbar:
  - Sticky "island" style with glass backdrop.

## Layout and Interaction

- Layout:
  - Prefer Bento-style CSS Grid blocks over long single-column sections.
- Motion:
  - Use meaningful hover lift (`translateY(-4px/-5px)`) and reveal transitions.
- Whitespace:
  - Keep generous spacing; avoid dense packing.

## Typography

- Heading family: geometric sans-serif (for example Space Grotesk, Outfit).
- Body family: highly readable sans-serif.
- Avoid decorative sci-fi fonts for body copy.

## Accessibility and Usability

- Maintain contrast and legibility over gradients and glow effects.
- Keep keyboard focus states visible.
- Do not encode meaning with color alone.

## Current Reference Implementation

- Server Lit playground route: `/playground` in `apps/server`.
- Shared theme tokens stylesheet: `shared/design-system/src/theme.css`.
- Playground includes a runtime theme switch (`dark` / `light`) backed by `localStorage`.
- This page is the baseline style source for future shared components in `shared/design-system`.
