# DevDeck

**One workspace. Every developer tool.** DevDeck is a local-first developer workspace for projects, runtime processes, Git, environment validation, and API work.

## Status

This repository contains the first native-core milestone in progress. The polished shell remains intact, and a Tauri 2/Rust host has been added with typed project, Git, and port commands. The source integration is present, but native compilation and desktop runtime execution have **not** yet been verified in this environment. The browser preview remains an explicit fallback and does not pretend to have operating-system access.

## Development

Requirements: Node.js 20+ and npm (or pnpm). From the project directory:

```bash
npm install
npm run dev       # local preview
npm run test      # unit tests
npm run lint      # strict TypeScript check
npm run build     # production bundle
npm run tauri:dev # native development (requires Rust and Tauri system dependencies)
```

Use **Register project** in a Chromium-based browser to choose a local folder. Metadata is stored in browser localStorage for this milestone. No project contents or secrets are uploaded.

## Architecture

`src/domain.ts` contains pure browser project detection logic. `src/native.ts` is the typed frontend/native adapter. `src-tauri/src/storage.rs` owns SQLite project metadata and `src-tauri/src/commands.rs` exposes explicit project, Git, and listening-port commands. The native capability file grants only core and directory-dialog permissions; no shell or unrestricted filesystem capability is granted. SQLite creates the `projects` table in the application data directory with a unique canonical path.

## Native setup

Install Rust through the official `rustup` installer and install the Tauri 2 prerequisites for your platform. On Debian/Ubuntu, the Tauri prerequisites include GTK/WebKitGTK development packages, `librsvg2-dev`, `libssl-dev`, and `patchelf`. Then run:

```bash
npm install
npm run tauri:dev
npm run tauri:build
```

The current OmniRush execution environment is Debian under WSL2 (`Linux ... microsoft-standard-WSL2`), not a Windows-native terminal. `powershell.exe`, `rustc`, `cargo`, and `rustup` are not available to this session. The Windows toolchain reported by the project owner could not be reached from this OmniRush process.

## Verification status

Verified in the current environment:

- `npm run lint`
- `npm run test` — 3 frontend tests passed
- `npm run build`
- `npm install --offline` — recovered cached platform packages
- `npm install` — completed successfully in the current environment

Not verified because the toolchain/dependencies are unavailable:

- `cargo fmt`
- Rust unit tests
- `cargo build`
- `npm run tauri:build`

The current native checks stopped at `cargo fmt` because Cargo is unavailable. Consequently, `cargo test`, `cargo build`, and `npm run tauri:build` were not executed in this run. This is not a Windows build result. A Windows build must be run from PowerShell on Windows with the Windows Rust target/toolchain and Windows Tauri CLI binding installed.

The native source includes focused Rust tests for SQLite CRUD, duplicate paths, invalid directories, project classification, and Linux TCP parsing. These require Rust/Cargo to execute.

## v0.2 implementation status

The current source includes safe native EnvSync inspection through `inspect_environment`. It returns variable names and validation metadata only; environment values never cross the Rust/frontend boundary. RequestBox supports explicit requests, query-string construction, editable headers/body, cancellation, response-size limiting, JSON formatting, and browser CORS limitations. Appearance settings now apply to the application and persist locally.

Verified in this WSL session:

```text
npm run lint  PASS
npm run test  PASS (6 tests)
npm run build PASS
```

Rust formatting, Rust tests, and Rust compilation were attempted but are unavailable in this environment because `cargo` is not installed. Native Windows GUI behavior, SQLite persistence, Windows Git inspection, and Windows port discovery must be verified from the already-working Windows PowerShell environment.

Command History now has a bounded, local browser persistence helper (`src/history.ts`) with credential-like summary redaction and a 100-entry retention limit. The visible history screen reads and clears those safe records. Native operation recording remains a follow-up integration task; this is intentionally not presented as complete until each operation records its actual result.

## Runtime integration note

The Tauri frontend bridge uses static Vite-resolvable imports from `@tauri-apps/api/core` and `@tauri-apps/plugin-dialog`. It must not use `@vite-ignore` dynamic imports for these packages: those leave bare module specifiers for the WebView to resolve at runtime and cause `Failed to resolve module specifier` errors. Browser preview still loads the same bundle, but only invokes native commands when Tauri internals are present.

The Rust test module syntax and formatting issues reported by the Windows build were corrected, and a valid multi-resolution Windows ICO was added at `src-tauri/icons/icon.ico`. The Tauri bundle configuration now references that icon. The `package metadata does not exist` message is informational Cargo metadata output; it is not a source configuration failure.

The browser preview was additionally checked by starting Vite and requesting `http://127.0.0.1:5173/`; it returned successfully. This verifies the preview server only, not native commands.

Static native review findings: command names and payload field names are aligned between `src/native.ts` and the Rust handlers; the dialog plugin is initialized and permission-scoped; filesystem paths are canonicalized before registration and Git inspection; Git arguments are fixed and read-only; and port discovery does not terminate processes or claim unavailable process attribution. Native compilation remains an unverified finding because the required toolchains were not present.

On Windows, PortPilot uses a fixed-argument `netstat -ano -p tcp` implementation and returns PID attribution when Windows reports it. Process names are intentionally left unavailable until a separately permission-reviewed lookup is added. macOS currently returns an explicit unsupported error rather than reporting fabricated port data.

## Roadmap

1. Complete native toolchain setup and wire the native adapter into all registration flows.
2. Add platform-specific process attribution and Git integration tests.
3. EnvSync safe template comparison and RequestBox local HTTP collections.
4. Cross-platform CI, packaging, contribution guide, security policy, and accessibility pass.

## License

License selection is pending repository ownership and release policy; no license is asserted by this foundation yet.
