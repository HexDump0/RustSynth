# RustSynth

RustSynth is a modern rewrite of Structure Synth for generating 3D scenes and models from code.

(P.S Take a look at the [docs](https://github.com/HexDump0/RustSynth/blob/main/docs/docs.md) for more info)

![app](https://raw.githubusercontent.com/HexDump0/RustSynth/refs/heads/main/img/app.png)

## Installation

A web version is hosted here: http://rustsynth.hexdump0.pw (Using WASM)

Desktop build are also available in the [releases](https://github.com/HexDump0/RustSynth/releases)

---

## Build/Run from source

### Requirements

- Rust
- Node.js 
- Tauri v2

### Build the CLI (Needed for both the desktop and web version)

```bash
git clone https://github.com/HexDump0/RustSynth/releases
cd src
cargo build
```

### Run desktop app (Tauri)

```bash
cd src/crates/rustsynth_app_tauri
npm install
npx tauri dev
```

### Run the web version

```bash
cd src/crates/rustsynth_app_tauri
npm install
npm run web:dev
```

---

## CLI Usage

```bash
cd src
cargo run -p rustsynth_cli -- --help
```

Build a script as OBJ (+ MTL):

```bash
cargo run -p rustsynth_cli -- export-obj path/to/model.es -o out/model.obj
```

Build scene JSON from script:

```bash
cargo run -p rustsynth_cli -- build path/to/model.es
```

Options:
- `--seed <u64>`
- `--max-objects <usize>`
- `--max-generations <u32>`
- `--mode bfs|dfs`

## Optimizations
- Using Rust
    Even though using typescript for the compilation part would be natural I decided to use Rust coz why the hell not. It is very much fast compared to the original version which was written in cpp.

    It takes `0.599s` to compile and export [tree_3d.es](https://raw.githubusercontent.com/HexDump0/RustSynth/refs/heads/main/src/crates/rustsynth_app_tauri/src/examples/Trees%203d.es) while it took `~3.2s` on StructureSynth

- Instanced rendering
    Objects of the same type are batched into one draw group, reducing draw calls from many per object to a few per group which is how it was done in StructureSynth. (Also using modern browser tech which is already wayy better at rendering stuff)

## Architecture

- Frontend stack: React + Vite + Three.js
- Desktop shell: Tauri v2
- Core pipeline: preprocess -> parse -> resolve -> validate -> evaluate -> export


## License
GPL-3.0