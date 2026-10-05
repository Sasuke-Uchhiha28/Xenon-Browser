---
name: Tactile Cyber-Onyx
colors:
  surface: '#131318'
  surface-dim: '#131318'
  surface-bright: '#39383e'
  surface-container-lowest: '#0e0e13'
  surface-container-low: '#1b1b20'
  surface-container: '#1f1f24'
  surface-container-high: '#2a292f'
  surface-container-highest: '#35343a'
  on-surface: '#e4e1e9'
  on-surface-variant: '#d1c1d9'
  inverse-surface: '#e4e1e9'
  inverse-on-surface: '#303035'
  outline: '#9a8ca2'
  outline-variant: '#4e4356'
  surface-tint: '#dfb7ff'
  primary: '#dfb7ff'
  on-primary: '#4b007e'
  primary-container: '#9d00ff'
  on-primary-container: '#f7e5ff'
  inverse-primary: '#8c00e5'
  secondary: '#e8b3ff'
  on-secondary: '#500075'
  secondary-container: '#a800ef'
  on-secondary-container: '#fbe5ff'
  tertiary: '#c7c3e3'
  on-tertiary: '#2f2e47'
  tertiary-container: '#696783'
  on-tertiary-container: '#ece8ff'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#f1daff'
  primary-fixed-dim: '#dfb7ff'
  on-primary-fixed: '#2d004f'
  on-primary-fixed-variant: '#6b00b0'
  secondary-fixed: '#f6d9ff'
  secondary-fixed-dim: '#e8b3ff'
  on-secondary-fixed: '#310049'
  on-secondary-fixed-variant: '#7200a4'
  tertiary-fixed: '#e3dfff'
  tertiary-fixed-dim: '#c7c3e3'
  on-tertiary-fixed: '#1a1931'
  on-tertiary-fixed-variant: '#46445f'
  background: '#131318'
  on-background: '#e4e1e9'
  surface-variant: '#35343a'
typography:
  display-xl:
    fontFamily: Space Grotesk
    fontSize: 56px
    fontWeight: '700'
    lineHeight: 64px
    letterSpacing: -0.03em
  display-xl-mobile:
    fontFamily: Space Grotesk
    fontSize: 36px
    fontWeight: '700'
    lineHeight: 44px
    letterSpacing: -0.02em
  headline-lg:
    fontFamily: Space Grotesk
    fontSize: 32px
    fontWeight: '600'
    lineHeight: 40px
    letterSpacing: -0.02em
  headline-lg-mobile:
    fontFamily: Space Grotesk
    fontSize: 26px
    fontWeight: '600'
    lineHeight: 34px
    letterSpacing: -0.01em
  headline-md:
    fontFamily: Space Grotesk
    fontSize: 22px
    fontWeight: '600'
    lineHeight: 28px
    letterSpacing: -0.01em
  title-sm:
    fontFamily: Geist
    fontSize: 16px
    fontWeight: '600'
    lineHeight: 22px
  body-lg:
    fontFamily: Geist
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  body-md:
    fontFamily: Geist
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  body-sm:
    fontFamily: Geist
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 16px
  label-code:
    fontFamily: JetBrains Mono
    fontSize: 13px
    fontWeight: '500'
    lineHeight: 18px
    letterSpacing: 0.02em
  label-badge:
    fontFamily: JetBrains Mono
    fontSize: 11px
    fontWeight: '600'
    lineHeight: 14px
    letterSpacing: 0.06em
rounded:
  sm: 0.25rem
  DEFAULT: 0.5rem
  md: 0.75rem
  lg: 1rem
  xl: 1.5rem
  full: 9999px
spacing:
  gutter: 1rem
  gutter-compact: 0.5rem
  margin: 1.5rem
  margin-mobile: 1rem
  space-xs: 0.25rem
  space-sm: 0.5rem
  space-md: 0.875rem
  space-lg: 1.25rem
  space-xl: 2rem
---

## Brand & Style

This design system blends **dark tactile neomorphism** with an aggressive, precision-engineered **cyber-minimalism**. Designed for power users, developers, and privacy-conscious web navigators, the aesthetic rejects flat, lifeless software surfaces in favor of extruded, physical slate planes, micro-beveled metallic edges, and reactive neon luminescence.

Key aesthetic pillars:
- **Tactile Matte Onyx:** Layered matte composites with fine physical depth, dual-directional ambient lighting, and subtle chamfers that mimic milled graphite and brushed titanium hardware.
- **Electric Luminescence:** Concentrated ultraviolet and electric violet core emissions that slice through deep charcoal surfaces, signaling activity, encryption strength, and focus states.
- **Engineered Precision:** Tight spacing, crisp telemetry labels, recessed control bays, and physical toggle switches designed to feel authoritative, mechanical, and low-latency.

## Colors

The palette revolves around deep, light-absorbing onyx and charcoal grounds punctuated by high-frequency electric purple light.

### Color Tokens & Roles
- **Canvas Base (`#0D0D12`):** Ultra-deep charcoal base canvas that swallows light and allows neomorphic shadow extrusions to retain dimensional distinction.
- **Surface Layer 1 (`#14141B`):** Structural panels, sidebars, URL navigation docks, and tab bar framing.
- **Surface Layer 2 (`#1B1B26`):** Interactive card tiles, floating omnibox pods, tactile buttons, and elevated modular widgets.
- **Primary Violet Accent (`#9D00FF`):** Core brand energetic glow, active browser security indicators, focused input halos, and primary action fills.
- **Secondary Electric Glow (`#B829FF`):** High-energy edge highlights, active tab line sweeps, hover state pings, and notification pips.
- **Gunmetal Specular Highlight (`#2A2A38`):** The physical top/left reflection line that defines extruded 3D bevels against dark backgrounds.
- **Shadow Trap (`#060608`):** Deep bottom/right ambient and cast occlusion shadows creating the neomorphic lift.
- **Text & Foreground:**
  - **High-Contrast Text (`#F2F2F7`):** Tab titles, URL hostnames, header readouts.
  - **Muted Telemetry Text (`#8C8B9E`):** URL path protocols, tracker block counters, shortcut key caps.
  - **Subtle Metadata (`#58576A`):** Inactive tabs, disabled states, divider notches.

## Typography

The type system balances technical sci-fi angularity with hyper-legible software UI performance.

- **Headlines (`Space Grotesk`):** Delivers aggressive, geometric authority for new tab dashboard headers, privacy summary metrics, and modal titles.
- **Body & Controls (`Geist`):** Crisp, neutral, and engineered specifically for high screen density; ideal for tab titles, context menus, and documentation settings.
- **Labels & Telemetry (`JetBrains Mono`):** Monospaced precision for the Omnibox URL engine, cipher information, connection latency counters, memory consumption badges, and keyboard shortcuts.

## Layout & Spacing

The structural layout uses an adjustable docked-shell architecture optimized for browser workflows:

- **Browser Chrome Top/Side Dock:** Flexible framing supporting both horizontal top tabs and vertical side-dock trees. A standard `0.5rem` micro-gutter segregates the viewport from browser controls.
- **Grid Structure:** A 12-column dynamic grid with responsive gutters for the dashboard and new-tab views, collapsing to 4 columns on mobile viewports.
- **Rhythm & Gaps:** Tight internal spacing (`space-xs` and `space-sm`) maintains physical density within toolbars and tab bars, preventing touch targets from feeling detached while preserving distinct bevel silhouettes.

## Elevation & Depth

Tactile neomorphism requires simulated multi-directional lighting rather than standard vertical drop shadows:

1. **The Primary Key Light (Top-Left):** Casts a sharp, metallic rim highlight along the upper and left edges using `#2A2A38` at low opacity, reinforcing mechanical chamfering.
2. **The Ambient Occlusion (Bottom-Right):** Casts a deep, soft shadow into `#060608` with a spread factor that physically pushes elements upward.
3. **Surface Tiers:**
   - **Recessed / Inset (Troughs & Address Bars):** Uses dual inner shadows (`inset 2px 2px 4px #060608`, `inset -2px -2px 4px #262536`) to visually carve the component into the chassis.
   - **Level 1 Extruded (Inactive Tabs, Standard Buttons):** Soft outward extrusion (`3px 3px 6px #07070B`, `-2px -2px 5px #222230`).
   - **Level 2 Floating (Active Tabs, Popover Cards, Privacy Shields):** Elevated extrusion coupled with a faint purple undertone glow (`0 8px 24px rgba(157, 0, 255, 0.18)` and `0 2px 6px rgba(0, 0, 0, 0.7)`).
4. **Active/Engaged Glow:** Interactive elements transition their top highlight to `#B829FF` with a diffused `0 0 12px rgba(184, 41, 255, 0.45)` laser bloom upon focus or active execution.

## Shapes

The shape system employs disciplined, rounded-chiseled geometries (`roundedness: 2`, 8px standard radius) simulating precision-cut industrial polymer and dark metal components.

- **Standard Elements (8px / 0.5rem):** Action buttons, input modules, Omnibox bar, tab items, dropdown lists.
- **Large Panels & Cards (16px / 1rem):** Privacy telemetry modules, security dashboard widgets, popover sheets.
- **Pill Indicators (9999px):** Status badges, tracker counters, and incognito privacy shields.
- **Chamfer Micro-Bevels:** 1px borders colored with directional linear gradients (`to bottom right, #2E2D40, #101016`) accentuate hard silhouette definition across all tactile layers.

## Components

### Omnibox & URL Field
- **Structure:** Carved, inset neomorphic dock container set inside the top chrome.
- **Styling:** Recessed `#0D0D12` background with double inset shadows. Monospaced path styling where the domain protocol is muted (`#58576A`), the root hostname is bold high-contrast (`#F2F2F7`), and parameters are subtle.
- **Security Lock Shield:** Integrated pill badge at left edge; displays green-violet gradient border with an illuminated shield icon when SSL/DNS-over-HTTPS is verified.

### Navigation Tabs (Horizontal & Vertical)
- **Inactive Tab:** Flat extruded plate with flush dark matte surface (`#14141B`), subtle top bevel highlight, and muted text.
- **Active Tab:** Raised tactile body (`#1B1B26`), framed by an electric violet under-glow sweep (`#B829FF`), white high-contrast label, and a recessed circular close button that extrudes on hover.

### Tactile Action Buttons
- **Primary:** Neon-infused violet core (`linear-gradient(135deg, #B829FF 0%, #9D00FF 100%)`) with dark gunmetal 1px bounding rim, high-contrast white text, and an ambient purple shadow plume.
- **Tactile Secondary:** Raised matte charcoal pill (`#1B1B26`), top-left specular highlight, depressing into an inset inner shadow state on `:active` clicks.

### Privacy Shields & Telemetry Cards
- **Structure:** Level 2 elevated widgets featuring modular telemetry readouts (trackers blocked, scripts stopped, canvas fingerprint protection status).
- **Styling:** Matte dark card with perimeter gradient stroke. Includes a neon circular radar/gauge component using `#9D00FF` progress tracks.

### Checkboxes, Switches & Radio Controls
- **Switches:** Recessed track (`inset 0 2px 4px #060608`) with an extruded tactile knob that glides along the horizontal axis, shifting from dark gunmetal to glowing electric violet when activated.
- **Checkboxes:** Square extruded bevels (`6px` radius); checked states display an inset purple core stamped with a sharp metallic checkmark.

### Context Menus & Tooltips
- **Floating Panels:** Deep `#14141B` surfaces rimmed with a 1px metallic stroke (`#2A2A38`), isolated with a heavy ambient drop shadow. Hovered menu items display an extruded highlight bar with an electric violet indicator pip on the left margin.