# Project: XENO INFERENCE

## Architecture & System Overview
XENO INFERENCE is an omni-modal autonomous inference engine, agentic harness (PAORV), real-time execution DAG, sandboxed terminal runtime, and dual-surface interface (desktop spatial canvas and terminal TUI).

### Workspace Crates & Packages
- `crates/xeno-core`: Core data contracts (`XenoAgentStepEvent`, `XenoDAGNode`, `TokenMetrics`, `InferenceRequest`, `InferenceResponse`, `ChatMessage`, `ContentBlock`, `ToolDefinition`, `StreamChunk`, `ProviderConfig`, `PrivacyFilter`), error taxonomy, Serde schemas.
- `crates/xeno-router`: Unified `InferenceProvider` async trait with 7 provider backends (Mock, Local OpenAI-compatible, Anthropic Messages API v1, OpenAI, Gemini, Groq, DeepSeek), Tokio async streaming token bus, real-time TTFT / velocity (tok/s) tracker, cost estimator, pre-flight privacy regex scrubber, and socket air-gap enforcer.
- `crates/xeno-tools`: Isolated virtual PTY terminal manager using `portable-pty` (Windows ConPTY / Job Objects) with Tier 1/2/3 command safety permissions, process tree reaper, atomic character-exact file editing engine (`multi_replace_file_content`, `atomic_write_file`, `file_read_slice`), `fuzzy_glob_ripgrep`, AST syntax validation (Rust, Python, TS/JS, JSON, TOML), and strict `python.exe` invocation.
- `crates/xeno-dag`: Dynamic graph state tracking using `petgraph`, streaming status transitions (`pending`, `running`, `success`, `failed`, `self_healing`), dynamic subgraph grafting, and typed event bus.
- `crates/xeno-telemetry`: Observable telemetry engine recording execution metrics, durations, token rates, and discrete step statuses without leaking private model chain-of-thought.
- `crates/xeno-agent`: Autonomous agent harness implementing the Plan-Act-Observe-Reflect-Verify (PAORV) continuous state machine, hierarchical swarm roles (Commander, Architect, Coder, QA Tester, Red-Team Auditor), multi-tier memory (L1 working, L2 episodic session store), and native Model Context Protocol (MCP) host with standard tool integrations.
- `crates/xeno-cli`: Standalone high-performance terminal UI (`ratatui` + `crossterm`) with ASCII banner, real-time HUD (VRAM/tokens/cost), Unicode DAG graph, active diff viewer, and interactive prompt bar.
- `ui-canvas` & `crates/xeno-tauri`: Desktop spatial canvas workspace scaffold (Tauri v2 + React 19 / Vite, polymorphic nodes, visual diffs, WebGL viewport).

---

## Feature Inventory

| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| F01 | Core Event Models | `XenoAgentStepEvent` schema with Serde serialization matching Blueprint §13 | M1 | survey_core_1 |
| F02 | DAG Node Contract | `XenoDAGNode` strongly-typed data structures and Serde derivations | M1 | survey_core_1 |
| F03 | Telemetry & Token Metrics | `TokenMetrics` structure tracking TTFT, prompt/completion tokens, VRAM, and cost | M1 | survey_core_1 |
| F04 | Inference Request/Response | Generic chat messages, multimodal content blocks, and tool definitions | M1 | survey_core_1 |
| F05 | Error Taxonomy | Hierarchical `XenoError` definitions with typed variants across modules | M1 | survey_core_1 |
| F06 | Mock Provider | Deterministic test provider with configurable latency, stream chunks, and error injection | M2 | survey_core_1 |
| F07 | Local OpenAI Provider | SSE streaming client for llama.cpp, vLLM, and Ollama `/v1/chat/completions` | M2 | survey_core_1 |
| F08 | Anthropic Messages Provider | Anthropic Messages API v1 client with thinking/CoT blocks and event streaming | M2 | survey_core_1 |
| F09 | OpenAI Provider | Chat completions client supporting reasoning tokens (`o1`, `o3-mini`, `gpt-4o`) | M2 | survey_core_1 |
| F10 | Google Gemini Provider | Gemini REST / SSE `streamGenerateContent` client | M2 | survey_core_1 |
| F11 | Groq LPU Provider | Ultra-low latency inference client for Groq endpoints | M2 | survey_core_1 |
| F12 | DeepSeek Provider | DeepSeek API client with `reasoning_content` delta and `<think>` parsing | M2 | survey_core_1 |
| F13 | Streaming Token Bus | Tokio async stream wrapper with monotonic clock TTFT calculation | M2 | survey_core_1 |
| F14 | Token Velocity Calculator | Rolling window & Exponential Moving Average (EMA) tokens/sec tracker | M2 | survey_core_1 |
| F15 | Cost Estimation Engine | Real-time USD pricing tracker based on prompt and completion tokens | M2 | survey_core_1 |
| F16 | Secret & PII Sanitizer | Regex scrubber for AWS, OpenAI, GitHub PAT, JWT, SSH keys | M2 | survey_core_1 |
| F17 | Air-Gap Enforcer | Socket-level non-loopback network blocker in air-gapped isolation mode | M2 | survey_core_1 |
| F18 | Semantic Intent Router | Speed, Reasoning, Privacy, and Cost optimization routing policies | M2 | survey_core_1 |
| F19 | Provider Trait & Factory | Dynamic provider registry and factory instantiation from config | M2 | survey_core_1 |
| F20 | Virtual ConPTY Session | `portable-pty` session manager on Windows ConPTY | M3 | survey_tools_1 |
| F21 | Win32 Job Objects Sandboxing | Job Object container with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` | M3 | survey_tools_1 |
| F22 | Process Tree Reaper | Recursive process termination ensuring zero orphan child processes | M3 | survey_tools_1 |
| F23 | Command Security Tiers | Tier 1 (Safe), Tier 2 (Guarded Snapshot), Tier 3 (Destructive Approval) | M3 | survey_tools_1 |
| F24 | Python Runtime Sanitizer | Strict validation enforcing `C:\msys64\ucrt64\bin\python.exe` | M3 | survey_tools_1 |
| F25 | Multi-Replace File Content | Character-exact line-bounded substring replacement | M3 | survey_tools_1 |
| F26 | In-Memory AST Validation | Syntax validation for Rust, Python, TS/JS, JSON, TOML before write | M3 | survey_tools_1 |
| F27 | Diff Snapshot & Rollback | Unified diff generation and in-memory rollback stack | M3 | survey_tools_1 |
| F28 | Atomic File Writer | Safe temp file creation and atomic rename swap | M3 | survey_tools_1 |
| F29 | File Read Slice | 1-indexed line slice reader with UTF-8 safety and truncation limits | M3 | survey_tools_1 |
| F30 | Fuzzy Glob & Ripgrep | High-speed `.gitignore`-aware directory search and file traversal | M3 | survey_tools_1 |
| F31 | Tool Trait & Execution Context | Standardized `XenoTool` async trait and execution environment | M3 | survey_tools_1 |
| F32 | Native MCP Tool Runtime | Model Context Protocol client-server bridge over STDIO and SSE | M3 | survey_tools_1 |
| F33 | Execution DAG State Tracker | `petgraph`-backed directed acyclic graph managing node dependencies | M4 | survey_agent_ui_1 |
| F34 | Dynamic Subgraph Grafting | Dynamic insertion of self-healing and tool execution nodes | M4 | survey_agent_ui_1 |
| F35 | Streaming Graph Event Bus | Tokio broadcast channel streaming node status transitions | M4 | survey_agent_ui_1 |
| F36 | Observable Telemetry Engine | Privacy-safe telemetry metrics without leaking private CoT | M4 | survey_agent_ui_1 |
| F37 | Step Duration & Velocity Telemetry | Discrete step timing, token rate, and cost recording | M4 | survey_agent_ui_1 |
| F38 | PAORV State Machine | Plan-Act-Observe-Reflect-Verify continuous autonomous execution loop | M5 | survey_agent_ui_1 |
| F39 | Swarm Role Council | Commander, Architect, Coder, QA Tester, Red-Team Auditor roles | M5 | survey_agent_ui_1 |
| F40 | Multi-Tier Memory Manager | L1 working memory context window & L2 episodic session store | M5 | survey_agent_ui_1 |
| F41 | Recursive Self-Healing Loop | Automated patch synthesis and retry on tool/verification anomalies | M5 | survey_agent_ui_1 |
| F42 | Cross-Model Consensus Checker | 3-way consensus evaluator for critical code/logic verification | M5 | survey_agent_ui_1 |
| F43 | Ratatui TUI Application | Interactive terminal UI binary with ASCII header and keyboard navigation | M6 | survey_agent_ui_1 |
| F44 | TUI ASCII DAG & HUD | Terminal Braille/ASCII graphs for DAG nodes, VRAM, and token speed | M6 | survey_agent_ui_1 |
| F45 | TUI Diff Viewer & Omni-Bar | Live AST diff preview pane and interactive command input bar | M6 | survey_agent_ui_1 |
| F46 | Desktop Spatial Canvas Scaffold | Tauri v2 + React 19 infinite canvas workspace scaffold | M6 | survey_agent_ui_1 |
| F47 | Canvas Polymorphic Nodes | Prompt, code preview, diff view, and artifact projection components | M6 | survey_agent_ui_1 |
| F48 | End-to-End Vertical Slice | Full integration: Prompt -> Routing -> Tool Exec -> Verify -> Complete | M7 | All |

---

## Milestones

| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Core Primitives & Data Contracts (`xeno-core`) | Data contracts, events, errors, metrics, Serde schemas (F01-F05) | none | DONE |
| M2 | Unified Inference Router & Token Bus (`xeno-router`) | 7 providers, streaming bus, velocity/cost, privacy scrubber, airgap (F06-F19) | M1 | DONE |
| M3 | Sandboxed PTY & Atomic AST File Engine (`xeno-tools`) | ConPTY/Job Objects, Tier 1/2/3 safety, atomic edits, AST validation, python.exe, MCP runtime (F20-F32) | M1 | DONE |
| M4 | Execution DAG & Observable Telemetry (`xeno-dag`, `xeno-telemetry`) | Petgraph DAG, dynamic grafting, event bus, privacy-safe telemetry (F33-F37) | M1 | DONE |
| M5 | Autonomous Agent Harness & Swarm (`xeno-agent`) | PAORV state machine, Swarm council, L1/L2 memory, self-healing, consensus (F38-F42) | M1, M2, M3, M4 | DONE |
| M6 | Dual User Interface: Terminal TUI & Canvas (`xeno-cli`, `xeno-tauri` / `ui-canvas`) | Ratatui TUI + Tauri v2 / React 19 spatial canvas scaffold (F43-F47) | M1, M2, M3, M4, M5 | DONE |
| M7 | End-to-End Vertical Slice & System Verification | E2E test suite pass (Tiers 1-4) & adversarial coverage hardening (Tier 5) (F48) | M1-M6, Test Track | DONE |

---

## Interface Contracts

### `xeno-core` ↔ `xeno-router`
- `InferenceProvider` trait:
  ```rust
  #[async_trait]
  pub trait InferenceProvider: Send + Sync {
      fn provider_type(&self) -> ProviderType;
      async fn infer(&self, request: &InferenceRequest) -> Result<InferenceResponse, XenoError>;
      async fn infer_stream(
          &self,
          request: &InferenceRequest,
      ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, XenoError>> + Send>>, XenoError>;
  }
  ```

### `xeno-core` ↔ `xeno-tools`
- `XenoTool` trait:
  ```rust
  #[async_trait]
  pub trait XenoTool: Send + Sync {
      fn name(&self) -> &str;
      fn description(&self) -> &str;
      fn parameters_schema(&self) -> serde_json::Value;
      fn security_tier(&self) -> ToolSecurityTier;
      async fn execute(
          &self,
          args: serde_json::Value,
          ctx: &ToolExecutionContext,
      ) -> Result<ToolObservation, ToolError>;
  }
  ```

### `xeno-core` ↔ `xeno-dag`
- `DAGNodeEvent` & `DAGGraph`:
  ```rust
  pub struct XenoDAGGraph { ... }
  impl XenoDAGGraph {
      pub fn add_node(&mut self, node: XenoDAGNode) -> Result<String, DAGError>;
      pub fn update_status(&mut self, node_id: &str, status: NodeStatus) -> Result<(), DAGError>;
      pub fn subscribe(&self) -> broadcast::Receiver<DAGNodeEvent>;
  }
  ```

### `xeno-core` / `xeno-tools` / `xeno-router` ↔ `xeno-agent`
- `AgentHarness`:
  ```rust
  pub struct XenoAgentHarness { ... }
  impl XenoAgentHarness {
      pub async fn execute_goal(
          &mut self,
          goal: &str,
      ) -> Result<AgentExecutionResult, AgentError>;
  }
  ```

---

## Code Layout
```
D:/PROJECTS/OM/
├── Cargo.toml
├── crates/
│   ├── xeno-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── contracts.rs
│   │       ├── events.rs
│   │       ├── errors.rs
│   │       ├── metrics.rs
│   │       └── types.rs
│   ├── xeno-router/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── provider.rs
│   │       ├── providers/
│   │       │   ├── mod.rs
│   │       │   ├── mock.rs
│   │       │   ├── local_openai.rs
│   │       │   ├── anthropic.rs
│   │       │   ├── openai.rs
│   │       │   ├── gemini.rs
│   │       │   ├── groq.rs
│   │       │   └── deepseek.rs
│   │       ├── token_bus.rs
│   │       ├── velocity.rs
│   │       ├── pricing.rs
│   │       ├── privacy.rs
│   │       └── router.rs
│   ├── xeno-tools/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pty.rs
│   │       ├── safety.rs
│   │       ├── file_engine.rs
│   │       ├── ast_validator.rs
│   │       ├── search.rs
│   │       ├── python_runner.rs
│   │       ├── mcp.rs
│   │       └── tool_trait.rs
│   ├── xeno-dag/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── graph.rs
│   │       ├── node.rs
│   │       └── events.rs
│   ├── xeno-telemetry/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── collector.rs
│   │       ├── metrics.rs
│   │       └── privacy_guard.rs
│   ├── xeno-agent/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── harness.rs
│   │       ├── paorv.rs
│   │       ├── swarm.rs
│   │       ├── memory.rs
│   │       └── self_healing.rs
│   ├── xeno-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs
│   │       ├── tui.rs
│   │       ├── ui/
│   │       │   ├── mod.rs
│   │       │   ├── banner.rs
│   │       │   ├── hud.rs
│   │       │   ├── dag_view.rs
│   │       │   ├── diff_view.rs
│   │       │   └── prompt_bar.rs
│   │       └── event_loop.rs
│   └── xeno-tauri/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── ui-canvas/
│   ├── package.json
│   ├── vite.config.ts
│   ├── index.html
│   └── src/
│       ├── App.tsx
│       ├── main.tsx
│       └── components/
└── tests/
    ├── e2e_vertical_slice.rs
    ├── router_integration.rs
    ├── tools_sandboxing.rs
    └── agent_swarm_test.rs
```
