# Cryn

**C**ufe D**ry** Ru**n** — a cross-platform timetable planner for Cairo University Faculty of Engineering (CUFE) students. Rewritten in Rust from the original [C# version](https://github.com/MRKDaGods/CUFE-Dry-Run) for native cross-platform support and WebAssembly deployment.

## Features

- Timetable grid with a layout engine that handles overlapping slots and multi-period courses
- Clash detection between selected lectures and tutorials
- Searchable course list with multi-query filtering (comma-separated)
- Import selections by pasting a course summary (e.g. `[CSEN401] Course Name (1/2)`) with live preview and warnings
- Screenshot export from the timetable view

## Platforms

| Platform | Renderer | Status |
|----------|----------|--------|
| Windows (x86_64) | Glow (OpenGL) | Supported |
| Web (WASM) | WebGPU/WebGL | Supported |
| Linux / macOS | Glow (OpenGL) | Should work (untested) |

## Building

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)

### Desktop

```sh
cargo run
```

Release build:

```sh
cargo run --release
```

### Web (WASM)

Install [Trunk](https://trunkrs.dev/):

```sh
cargo install trunk
```

Build and serve:

```sh
trunk serve
```

The app will be available at `http://127.0.0.1:8080`.

## Architecture

```
src/
├── main.rs              # Entry point, platform dispatch
├── app.rs               # CrynApp (eframe::App), context, font setup
├── platform/            # Desktop (native) and Web (WASM) runners
├── models/              # CourseDefinition, CourseRecord, CourseSpan, CourseSummary, events
├── services/
│   ├── course_manager   # Central course state, selection logic, clash detection
│   └── parsers/         # HTML table parser, summary text parser
├── views/
│   ├── timetable_view/  # Grid renderer, layout engine, colors, landing page, navbar extras
│   ├── courses_view     # Searchable course table with selection toggles
│   └── placeholder_view # Stub for unimplemented views (Settings)
├── windows/
│   ├── main_window/     # Title bar, navbar, view container, desktop resize/drag
│   └── import_window    # Modal for pasting and importing course summaries
├── utils/               # Logging, signals, UI helpers
└── icons.rs             # Segoe MDL2 icon codepoints
```

## Roadmap

- [ ] Dynamic loading of course data (currently uses embedded sample data)
- [ ] Auto-updating
- [ ] Settings view
- [ ] Linux and macOS testing
