# SmartEqualizer APO Presets Manager

A desktop application for managing Equalizer APO presets, built with SvelteKit and Tauri.

## Overview

SmartEqualizer APO Presets Manager provides an intuitive graphical interface for applying, syncing, and editing Equalizer APO configuration files, allowing for rapid switching of audio presets.
It also recognizes convolution presets backed by WAV impulse responses, so you can import `.wav` files, update their linked paths, and reveal them in Explorer from the editor.

## App stack

* **Svelte 5**
* **Tailwind 4**
* **TypeScript**
* **Rust**
* **Tauri V2**

## Getting Started

### Prerequisites

* Node.js (and npm/yarn/pnpm)
* Rust (for Tauri backend, if applicable)

### Installation

1. Clone the repository and navigate to the project directory:

   ```bash
   cd SmartEqualizerAPOPresetsManager
   ```

2. Install dependencies:

   ```bash
   npm install
   ```

3. Run the development server:

   ```bash
   npm run tauri dev
   ```

   *Note: Using `npm run tauri dev` will start the Tauri app in development mode along with the SvelteKit frontend server.*

### Building

To build the application for production:

```bash
npm run tauri build
```

The resulting executables will be located in the `src-tauri/target/release/` directory.
