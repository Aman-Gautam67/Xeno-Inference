# XENO INFERENCE — E2E Test Infrastructure & Test Suite Specification (`TEST_INFRA.md`)

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│    _  ________   ______     _____   ____________________  ______   ____________        │
│   | |/ / ____/ | / / __ \   /  _/ | / / ____/ ____/ __ \/ ____/ | / / ____/ ____/      │
│   |   / __/ /  |/ / / / /   / //  |/ / /_  / __/ / /_/ / __/ /  |/ / /   / __/         │
│  /   / /___/ /|  / /_/ /  _/ // /|  / __/ / /___/ _, _/ /___/ /|  / /___/ /___         │
│ /_/|_\____/_/ |_/\____/  /___/_/ |_/_/   /_____/_/ |_/_____/_/ |_/\____/_____/         │
│                                                                                        │
│                [ 4-TIER E2E TEST INFRASTRUCTURE & VALIDATION SUITE ]                   │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

**Document Version**: 1.0.0-PROD-SPEC  
**Author**: `test_writer_1` (E2E Test Suite Architect & Writer)  
**Integrity Mode**: Development / Sovereign Zero-Regression  
**Target Environment**: Rust 1.97.0 (Cargo Workspace) / Windows 11 x64 / `python.exe` (`C:\msys64\ucrt64\bin\python.exe`)  

---

## 1. Executive Summary & Testing Philosophy

The **XENO INFERENCE** test infrastructure is engineered around an **opaque-box, requirement-driven, 4-tier testing methodology**. The suite validates the entire surface area of the system across all 48 distinct features (**F01–F48**) spanning 5 foundational pillars (R1–R5):
1. **R1: Core Primitives & Unified Inference Router** (`xeno-core`, `xeno-router`)
2. **R2: Sandboxed PTY & Atomic AST File System Engine** (`xeno-tools`)
3. **R3: Autonomous Agent Harness (PAORV) & Native MCP Runtime** (`xeno-agent`)
4. **R4: Real-Time Execution DAG & Observable Telemetry** (`xeno-dag`, `xeno-telemetry`)
5. **R5: Dual User Interface: Terminal TUI & Desktop Spatial Canvas** (`xeno-cli`, `xeno-tauri` / `ui-canvas`)

### 1.1 Core Principles of the E2E Test Track

- **Opaque-Box Verification**: Tests interact strictly through public crate API contracts, serialization schemas (Serde JSON), asynchronous trait interfaces (`InferenceProvider`, `XenoTool`), and observable event streams without coupling to private internal data structures.
- **Requirement-Driven Traceability**: Every single test case traces directly to an authoritative requirement in `ORIGINAL_REQUEST.md`, `PROJECT.md`, or `XENO_INFERENCE_BLUEPRINT.md`.
- **Progressive Testability & Strict Isolation**: Tests are self-contained and idempotent. Each test initializes its own isolated in-memory or `tempfile::TempDir` workspace, establishes independent Tokio async channels, and cleanly cleans up all resources (Job Objects, child processes, temp files).
- **Zero Orphan Process Guarantee**: Sandboxed terminal tests verify that Win32 Job Objects (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) and process tree reapers completely terminate child processes on completion or timeout.
- **Strict Python Runtime Verification**: All Python execution tests strictly enforce invocation via `python.exe` at the verified path `C:\msys64\ucrt64\bin\python.exe`. Bare `python` invocations are rejected.
- **Privacy & Telemetry Boundary**: Observable telemetry tests verify that metrics, token velocities, and tool I/O are captured cleanly while strictly prohibiting exposure or leakage of private model chain-of-thought (CoT).

---

## 2. 4-Tier Test Architecture

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                               TIER 4: REAL-WORLD WORKLOADS                             │
│   • Autonomous Code Refactoring Loop • Self-Healing Bug Fixer • Swarm Multi-Tool Exec  │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                          TIER 3: CROSS-FEATURE COMBINATIONS                            │
│   • Router + Tool Engine • PTY + File Rollback • DAG + Telemetry • Swarm + Consensus   │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                       TIER 2: BOUNDARY & CORNER CASES (ADVERSARIAL)                    │
│   • Malformed JSON • Huge Payloads • Unicode/ANSI • Process Timeouts • Socket Air-Gap  │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                           TIER 1: FEATURE COVERAGE (F01 - F48)                         │
│   • >=5 Tests per Subsystem • 48 Feature Matrix • Public API Contract Invariants       │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Tier 1: Feature Coverage Matrix (F01–F48)

Tier 1 provides exhaustive verification of individual functional units across all 7 milestones and 48 features, requiring at least 5 targeted test cases per major architectural area.

| Feature ID | Feature Name | Primary Contract / Interface | Test Target | Minimum Test Cases |
| :--- | :--- | :--- | :--- | :--- |
| **F01** | Core Event Models | `XenoAgentStepEvent`, `ThinkingPayload`, `ObservationPayload` | `xeno-core::events` | 5 |
| **F02** | DAG Node Contract | `XenoDAGNode`, `NodeType`, `NodeStatus`, `ModelAssignment` | `xeno-core::types` | 5 |
| **F03** | Telemetry & Token Metrics | `TokenMetrics`, `HardwareStats`, `calculate_tps` | `xeno-core::metrics` | 5 |
| **F04** | Inference Request/Response | `InferenceRequest`, `InferenceResponse`, `ChatMessage` | `xeno-core::contracts` | 5 |
| **F05** | Error Taxonomy | `XenoError`, `InferenceError`, `ToolError`, `AgentError` | `xeno-core::errors` | 5 |
| **F06** | Mock Provider | `MockProvider`, configurable latency/chunks/errors | `xeno-router::providers::mock` | 5 |
| **F07** | Local OpenAI Provider | `LocalOpenAIProvider` SSE streaming client | `xeno-router::providers::local_openai` | 5 |
| **F08** | Anthropic Messages Provider | `AnthropicProvider` Messages API v1 with Thinking blocks | `xeno-router::providers::anthropic` | 5 |
| **F09** | OpenAI Provider | `OpenAIProvider` reasoning tokens (`o1`, `o3-mini`) | `xeno-router::providers::openai` | 5 |
| **F10** | Google Gemini Provider | `GeminiProvider` REST/SSE `streamGenerateContent` | `xeno-router::providers::gemini` | 5 |
| **F11** | Groq LPU Provider | `GroqProvider` ultra-low latency inference | `xeno-router::providers::groq` | 5 |
| **F12** | DeepSeek Provider | `DeepSeekProvider` `reasoning_content` delta parser | `xeno-router::providers::deepseek` | 5 |
| **F13** | Streaming Token Bus | `TokenBus`, monotonic TTFT computation, async stream | `xeno-router::token_bus` | 5 |
| **F14** | Token Velocity Calculator | Rolling window & EMA tokens/second tracker | `xeno-router::velocity` | 5 |
| **F15** | Cost Estimation Engine | Real-time USD pricing tracker based on token counts | `xeno-router::pricing` | 5 |
| **F16** | Secret & PII Sanitizer | Regex scrubber for AWS, OpenAI, GitHub PAT, JWT, SSH | `xeno-router::privacy` | 5 |
| **F17** | Air-Gap Enforcer | Socket-level non-loopback network blocker | `xeno-router::privacy` | 5 |
| **F18** | Semantic Intent Router | Speed, Reasoning, Privacy, Cost policy routing | `xeno-router::router` | 5 |
| **F19** | Provider Trait & Factory | Dynamic provider registry and factory instantiation | `xeno-router::provider` | 5 |
| **F20** | Virtual ConPTY Session | `portable-pty` session manager on Windows ConPTY | `xeno-tools::pty` | 5 |
| **F21** | Win32 Job Objects Sandboxing | Job Object container with `KILL_ON_JOB_CLOSE` | `xeno-tools::pty` | 5 |
| **F22** | Process Tree Reaper | Recursive process termination, zero orphan child guarantee | `xeno-tools::pty` | 5 |
| **F23** | Command Security Tiers | Tier 1 (Safe), Tier 2 (Guarded), Tier 3 (Destructive) | `xeno-tools::safety` | 5 |
| **F24** | Python Runtime Sanitizer | Strict validation of `C:\msys64\ucrt64\bin\python.exe` | `xeno-tools::python_runner` | 5 |
| **F25** | Multi-Replace File Content | Character-exact line-bounded substring replacement | `xeno-tools::file_engine` | 5 |
| **F26** | In-Memory AST Validation | Syntax validation for Rust, Python, TS, JSON, TOML | `xeno-tools::ast_validator` | 5 |
| **F27** | Diff Snapshot & Rollback | Unified diff generation and in-memory rollback stack | `xeno-tools::file_engine` | 5 |
| **F28** | Atomic File Writer | Safe temp file creation and atomic rename swap | `xeno-tools::file_engine` | 5 |
| **F29** | File Read Slice | 1-indexed line slice reader with UTF-8 safety | `xeno-tools::file_engine` | 5 |
| **F30** | Fuzzy Glob & Ripgrep | High-speed `.gitignore`-aware directory search | `xeno-tools::search` | 5 |
| **F31** | Tool Trait & Context | Standardized `XenoTool` async trait and execution ctx | `xeno-tools::tool_trait` | 5 |
| **F32** | Native MCP Tool Runtime | MCP client-server bridge over STDIO and SSE | `xeno-tools::mcp` | 5 |
| **F33** | Execution DAG State Tracker | `petgraph`-backed DAG managing node dependencies | `xeno-dag::graph` | 5 |
| **F34** | Dynamic Subgraph Grafting | Dynamic insertion of self-healing and tool nodes | `xeno-dag::graph` | 5 |
| **F35** | Streaming Graph Event Bus | Tokio broadcast channel streaming status transitions | `xeno-dag::events` | 5 |
| **F36** | Observable Telemetry Engine | Privacy-safe telemetry metrics without leaking CoT | `xeno-telemetry::collector` | 5 |
| **F37** | Step Duration & Velocity | Discrete step timing, token rate, and cost recording | `xeno-telemetry::metrics` | 5 |
| **F38** | PAORV State Machine | Plan-Act-Observe-Reflect-Verify continuous loop | `xeno-agent::paorv` | 5 |
| **F39** | Swarm Role Council | Commander, Architect, Coder, QA Tester, Red-Team | `xeno-agent::swarm` | 5 |
| **F40** | Multi-Tier Memory Manager | L1 working context window & L2 episodic session store | `xeno-agent::memory` | 5 |
| **F41** | Recursive Self-Healing Loop | Automated patch synthesis and retry on anomalies | `xeno-agent::self_healing` | 5 |
| **F42** | Cross-Model Consensus | 3-way consensus evaluator for critical verifications | `xeno-agent::swarm` | 5 |
| **F43** | Ratatui TUI Application | Interactive terminal UI binary with keyboard navigation | `xeno-cli::app` | 5 |
| **F44** | TUI ASCII DAG & HUD | Terminal Braille/ASCII graphs for DAG nodes & VRAM | `xeno-cli::ui::dag_view` | 5 |
| **F45** | TUI Diff Viewer & Omni-Bar | Live AST diff preview pane and interactive command bar | `xeno-cli::ui::diff_view` | 5 |
| **F46** | Desktop Spatial Canvas | Tauri v2 + React 19 infinite canvas workspace | `xeno-tauri` / `ui-canvas` | 5 |
| **F47** | Canvas Polymorphic Nodes | Prompt, code preview, diff view, artifact projections | `ui-canvas::components` | 5 |
| **F48** | End-to-End Vertical Slice | Full integration: Prompt -> Routing -> Tool -> Verify | `tests/e2e_vertical_slice.rs` | 5 |

---

### 2.2 Tier 2: Boundary & Corner Cases (Adversarial Verification)

Tier 2 exercises extreme input parameters, fault injection, and security boundary violations to ensure resilience:

1. **Empty & Zero-Length Payloads**:
   - Zero-token prompts, empty string file modifications, 0-line file slices.
   - Empty JSON objects in tool arguments, empty DAG graphs with zero nodes.
2. **Extreme Payload Sizes**:
   - Large context payloads (100,000+ tokens / 5MB payload).
   - Massive search queries returning 1,000+ files with strict truncation limits (cap at 200 matches).
   - Multi-megabyte file writes verifying atomic swap memory efficiency.
3. **Malformed & Adversarial JSON / AST**:
   - Corrupted JSON-RPC messages to MCP server.
   - Invalid Rust/Python syntax mutations rejected by AST validation before disk modification.
   - Ambiguous duplicate substrings in `multi_replace_file_content` with `AllowMultiple: false` properly erroring.
4. **Timeouts & Deadlock Resilience**:
   - PTY command execution exceeding timeout (e.g. `sleep 10` with 500ms timeout) cleanly killed without hanging.
   - Rapid async token bus cancellation during active streaming.
   - Broadcast buffer overflow handling with `RecvError::Lagged` graceful recovery.
5. **Unicode, Multi-Byte & Line Ending Integrity**:
   - Multi-byte UTF-8 slicing across character boundaries (Japanese Kanji, Emoji, Cyrillic).
   - Mixed CRLF (`\r\n`) and LF (`\n`) file content replacement preserving original line endings.
   - ANSI truecolor escape sequences in terminal output correctly stripped in telemetry.
6. **Air-Gap & Secret Scrubber Adversarial Cases**:
   - Prompts containing mixed AWS keys (`AKIA...`), GitHub PAT (`ghp_...`), JWTs, and private SSH keys scrubbed before cloud dispatch.
   - Non-loopback socket connection attempts blocked in Air-Gap mode.

---

### 2.3 Tier 3: Cross-Feature Combinations

Tier 3 validates integration across subsystem boundaries:

1. **Semantic Router + Sandboxed Tool Execution (`xeno-router` + `xeno-tools`)**:
   - Semantic Intent Router selects local provider based on privacy constraints, streams tool invocation chunks, dispatches to `XenoTool`, and parses observation.
2. **Virtual ConPTY + Atomic File Rollback (`xeno-tools`)**:
   - An agent executes a build command via ConPTY. When compiler errors are reported in stderr, the file engine rolls back to the previous snapshot using `RollbackHistory`.
3. **Execution DAG + Observable Telemetry (`xeno-dag` + `xeno-telemetry`)**:
   - As DAG nodes transition (`Pending` -> `Running` -> `Success`), events are broadcast on Tokio channel and telemetry aggregator updates token rate and duration metrics.
4. **Swarm Multi-Agent + 3-Way Consensus (`xeno-agent`)**:
   - Commander decomposes task -> Coder generates patch -> QA and Red-Team evaluate -> 3-way consensus engine aggregates results and approves commit.
5. **Multi-Tier Memory + Token Bus Stream (`xeno-agent` + `xeno-router`)**:
   - L1 working memory compacts historical messages when approaching context threshold while streaming token chunks from the async token bus.

---

### 2.4 Tier 4: Real-World Workload Applications

Tier 4 simulates complete, production-grade end-to-end user workflows:

1. **Workload 1: Autonomous Code Refactoring Loop**:
   - Goal: Refactor a function in a Rust module to optimize throughput.
   - Flow: Agent reads file slice -> identifies target signature -> calls `multi_replace_file_content` -> AST validator verifies syntax -> runs `cargo test` in ConPTY -> verifies 0 errors -> records telemetry.
2. **Workload 2: Self-Healing Debug Loop**:
   - Goal: Fix a deliberately injected syntax bug in a Python script.
   - Flow: Agent attempts execution using `python.exe` (`C:\msys64\ucrt64\bin\python.exe`) -> observes exit code 1 with traceback -> reflects on error line -> applies atomic patch -> re-executes `python.exe` -> verifies exit code 0.
3. **Workload 3: Multi-Tool Swarm Collaboration**:
   - Goal: Scaffold and verify a new multi-file module.
   - Flow: Commander initializes DAG -> Architect specifies schemas -> Coder writes files using `atomic_write_file` -> QA creates unit tests -> Red-Team audits secrets -> DAG completes with `AllSuccess`.
4. **Workload 4: Air-Gapped Zero-Leakage Audit**:
   - Goal: Process a proprietary codebase in Air-Gap mode.
   - Flow: Air-gap mode enabled -> prompt with confidential tokens and API keys sanitized -> local GGUF mock provider invoked -> verify zero external socket connections made.

---

## 3. Test Directory Layout & Integration Test Suite

The integration test suite is located in `tests/`:

```
D:/PROJECTS/OM/
├── Cargo.toml
├── crates/
│   ├── xeno-core/
│   ├── xeno-router/
│   ├── xeno-tools/
│   ├── xeno-dag/
│   ├── xeno-telemetry/
│   ├── xeno-agent/
│   ├── xeno-cli/
│   └── xeno-tauri/
└── tests/
    ├── e2e_opaque_box.rs       # Tier 1 Opaque-Box contracts & 48-feature interface tests
    ├── e2e_boundary_tests.rs   # Tier 2 Boundary, corner cases, error recovery & stress tests
    ├── e2e_cross_feature.rs    # Tier 3 Cross-feature integrations (Router+Tools, DAG+Telemetry, Swarm+Consensus)
    ├── e2e_workloads.rs        # Tier 4 Real-world end-to-end task workflows
    └── e2e_vertical_slice.rs   # Full vertical slice: Prompt -> Routing -> Tool -> Verify -> Complete
```

---

## 4. Test Execution & CI Automation Commands

### 4.1 Running the Full Test Suite
To run all unit and integration tests across the workspace:
```powershell
cargo test --workspace -- --nocapture
```

### 4.2 Running Specific Test Tiers
To execute individual E2E test suites:
```powershell
# Tier 1: Opaque-Box & Contract Tests
cargo test --test e2e_opaque_box -- --nocapture

# Tier 2: Boundary & Corner Cases
cargo test --test e2e_boundary_tests -- --nocapture

# Tier 3: Cross-Feature Combinations
cargo test --test e2e_cross_feature -- --nocapture

# Tier 4: Real-World Workload Applications
cargo test --test e2e_workloads -- --nocapture

# Tier 5 / Milestone Acceptance: Vertical Slice
cargo test --test e2e_vertical_slice -- --nocapture
```

### 4.3 Python Path Verification Command
To verify that Python execution strictly utilizes the UCRT64 binary:
```powershell
& "C:\msys64\ucrt64\bin\python.exe" --version
```

### 4.4 Environment Variables for Test Harnesses
| Variable | Value | Description |
| :--- | :--- | :--- |
| `XENO_INTEGRITY_MODE` | `development` | Enables deterministic mock clock and seeded RNG for reproducible tests. |
| `XENO_PYTHON_PATH` | `C:\msys64\ucrt64\bin\python.exe` | Explicit Python binary path checked by `xeno-tools`. |
| `XENO_AIRGAP_MODE` | `1` | Forces air-gapped isolation mode and socket blocking. |
| `RUST_LOG` | `info,xeno_router=debug,xeno_tools=debug` | Tracing log level for diagnostic capture during test execution. |

---

## 5. Verification & Acceptance Criteria

A test run is considered **PASSED (100% Ready)** when:
1. All 48 features (F01–F48) have corresponding passing tests in Tiers 1–4.
2. `cargo test --workspace` completes with **0 failed tests, 0 warnings, and 0 memory leaks**.
3. All PTY and subprocess processes terminate cleanly without lingering orphan child processes in Windows Task Manager.
4. All file replacements validate exact character substrings and maintain rollback safety.
5. Observable telemetry reports metrics accurately without any CoT leakage.
