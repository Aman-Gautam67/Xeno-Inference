# Original User Request

## Initial Request — 2026-08-20T17:46:04+05:30

# Teamwork Project Prompt — XENO INFERENCE

Build **XENO INFERENCE** — an omni-modal autonomous inference engine, agentic harness (PAORV), real-time execution DAG, sandboxed terminal runtime, and dual-surface interface (desktop spatial canvas and terminal TUI) according to `XENO_INFERENCE_BLUEPRINT.md`.

Working directory: D:/PROJECTS/OM
Integrity mode: development

## Environment & Tooling Constraints
- **Python Invocation**: Always invoke Python via `python.exe` explicitly (never bare `python`). Verified path: `C:\msys64\ucrt64\bin\python.exe`.
- **Rust Toolchain**: Rust 1.97.0 (Cargo workspace).
- **Node Environment**: Node v26.3.0 / npm 11.17.0.
- **Hardware Profile**: NVIDIA RTX 2050 (4GB VRAM, CUDA 12.7) / Windows 11 x64.
- **Privacy & Telemetry**: Observable telemetry only (timings, token rates, tool I/O, verification gates). Do not expose or fake private model chain-of-thought.

## Requirements

### R1. Core Primitives & Unified Inference Router (`xeno-core`, `xeno-router`)
Implement the core data contracts (`XenoAgentStepEvent`, `XenoDAGNode`, `TokenMetrics`), error models, and unified `InferenceProvider` abstraction with support for mock testing, local OpenAI-compatible endpoints, and cloud providers (Anthropic, OpenAI, Gemini, Groq, DeepSeek). Provide a Tokio async streaming token bus with real-time velocity and cost tracking.

### R2. Sandboxed PTY & Atomic AST File System Engine (`xeno-tools`)
Implement an isolated virtual PTY terminal manager using `portable-pty` (Windows ConPTY / Job Objects) with Tier 1/2/3 command safety permissions, alongside an atomic character-exact file editing engine with diff snapshots and rollback capabilities.

### R3. Autonomous Agent Harness (PAORV) & MCP Tool Runtime (`xeno-agent`)
Implement the Plan-Act-Observe-Reflect-Verify continuous state machine, hierarchical swarm roles (Commander, Architect, Coder, QA Tester, Red-Team Auditor), and native Model Context Protocol (MCP) host with standard tool integrations.

### R4. Real-Time Execution DAG & Observable Timeline Inspector
Implement dynamic graph state tracking (`petgraph`) with streaming status transitions, edge data flow telemetry, and structured cognitive step logging.

### R5. Dual User Interface: Terminal TUI (`xeno-cli`) & Desktop Spatial Canvas (`xeno-tauri` / `ui-canvas`)
Implement the standalone Rust terminal UI (`ratatui` + `crossterm`) with Unicode DAG and telemetry, and scaffold the Tauri v2 + React 19 spatial canvas workspace.

## Acceptance Criteria

### Verification & Test Suite
- [ ] `cargo test --workspace` compiles and passes all unit and integration tests cleanly.
- [ ] End-to-end vertical slice test demonstrates: prompt routing -> streaming tokens -> PTY/file tool invocation -> observation parsing -> verified task completion.
- [ ] Sandboxed terminal execution safely runs commands, handles timeouts, and captures exit codes/stdout/stderr without orphan processes.
- [ ] Atomic file replacements correctly validate substring matches and prevent corrupt edits.
- [ ] Observable telemetry strictly reports execution metrics and discrete step statuses without leaking internal CoT.
- [ ] Python scripts and tools strictly use `python.exe`.
