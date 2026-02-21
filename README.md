# 🌌 SOLARA

> *The solar system, forged in Rust, alive in your browser.*

A **real-time, interactive solar system simulator** running entirely in the browser via **Rust + WebAssembly + WebGL2**. No install, no backend, no account — just the cosmos.

![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange)
![WASM](https://img.shields.io/badge/WebAssembly-target-blue)
![License](https://img.shields.io/badge/License-MIT-green)

## ✨ Features

- **8 planets** with real NASA orbital data (semi-major axes, periods, inclinations)
- **Kepler orbital mechanics** — all planets orbit at physically correct relative speeds
- **Real-time simulation** — configurable time scale (default: 1 second = 1 Earth day)
- **Orbital camera** — mouse drag to rotate, scroll to zoom, touch support for mobile
- **3000+ star** background with twinkling shader animation
- **Phong shading** with atmospheric rim lighting on all planets
- **Saturn's rings** rendered as a translucent annulus
- **Sun glow** — self-illuminated central star
- **60fps** on mid-range hardware
- **105KB** optimized WASM binary

## 🏗️ Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust 2024 Edition |
| Compile target | `wasm32-unknown-unknown` via `wasm-pack` |
| Rendering | WebGL2 via `web-sys` |
| Math | `glam` (vectors, matrices) |
| Dev server | Vite |
| Build tooling | `wasm-pack` + `wasm-opt` |

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (stable, 1.85+)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Node.js](https://nodejs.org/) (18+)

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack
```

### Build & Run

```bash
# Clone the repository
git clone https://github.com/npiron/SolarSysteme.git
cd SolarSysteme

# Install npm dependencies
npm install

# Build WASM (dev mode) and start development server
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

### Production Build

```bash
# Build optimized WASM + bundle
npm run build

# Preview the production build
npm run preview
```

### Run Tests

```bash
# Unit tests (simulation logic — pure Rust, no browser needed)
cargo test

# Clippy lints
cargo clippy --all-targets
```

## 📁 Project Structure

```
solara/
├── src/
│   ├── lib.rs              # WASM entry point + unit tests
│   ├── simulation/
│   │   ├── mod.rs          # Simulation orchestrator
│   │   ├── body.rs         # CelestialBody struct + orbital position computation
│   │   ├── orbit.rs        # Orbit path geometry generation
│   │   └── time.rs         # Simulation clock & speed control
│   ├── renderer/
│   │   ├── mod.rs          # WebGL2 renderer + shaders (inline GLSL)
│   │   └── camera.rs       # Orbital camera controller
│   ├── data/
│   │   └── solar_system.rs # NASA planetary data (distances, periods, radii, colors)
│   └── input/
│       └── mod.rs          # Mouse / touch / keyboard input handling
├── www/
│   ├── index.html          # Minimal HTML shell
│   ├── style.css           # Dark space theme
│   └── bootstrap.js        # WASM loader
├── Cargo.toml
├── package.json
├── vite.config.js
└── README.md
```

## 🎨 Design Decisions

- **Raw WebGL2 via `web-sys`** instead of `wgpu` — keeps the binary small (105KB vs ~2MB+), ensures 100% browser compatibility (WebGL2 is universal), and avoids `wgpu`'s heavy dependency tree.
- **Circular Kepler orbits** for V1 — accurate enough for visual correctness, trivial to compute at 60fps. Elliptical refinement can come in V2.
- **Log-scaled planet sizes** — true scale would make Mercury invisible next to Jupiter. We use `log10(radius_km)` scaling so all planets remain visible while maintaining relative ordering.
- **Inline GLSL shaders** — no external shader files to load. All 6 shader programs are compiled from `&str` constants at initialization time.
- **Spherical coordinate camera** — simple, intuitive orbital camera that always looks at the Sun. No gimbal lock thanks to phi clamping.

## 🌍 Planet Data (NASA)

| Planet   | Semi-major axis (AU) | Period (days) | Color |
|----------|---------------------|---------------|-------|
| Mercury  | 0.387               | 87.97         | #b5b5b5 |
| Venus    | 0.723               | 224.70        | #e8cda0 |
| Earth    | 1.000               | 365.25        | #4fa3e0 |
| Mars     | 1.524               | 687.00        | #c1440e |
| Jupiter  | 5.203               | 4,332.59      | #c88b3a |
| Saturn   | 9.537               | 10,759.22     | #e4d191 |
| Uranus   | 19.191              | 30,688.50     | #7de8e8 |
| Neptune  | 30.069              | 60,182.00     | #3f54ba |

## 🚢 Deployment (GitHub Pages)

```bash
# Build for production
npm run build

# The `dist/` folder contains fully static files
# Deploy to any static host: GitHub Pages, Vercel, Cloudflare Pages, Netlify
```

For GitHub Pages, push the contents of `dist/` to a `gh-pages` branch or configure GitHub Actions.

## 📋 Roadmap (V2)

- [ ] Click-to-focus with smooth camera animation
- [ ] Time controls (pause/play, speed slider)
- [ ] NASA texture maps on planets
- [ ] Earth's Moon + Jupiter's Galilean moons
- [ ] HUD with simulation date and planet info
- [ ] Asteroid belt (particle system)

## 📄 License

MIT
