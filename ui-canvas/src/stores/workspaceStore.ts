import { create } from "zustand";

export type ViewMode = "canvas" | "dag" | "timeline" | "terminal" | "diff" | "swarm";
export type ProviderModel = 
  | "claude-3-7-sonnet" 
  | "deepseek-r1" 
  | "gpt-4o" 
  | "gemini-2-pro" 
  | "local-gguf" 
  | "groq-llama3";
export type RoutingPolicy = "speed" | "reasoning" | "privacy" | "cost";
export type NodeStatus = "pending" | "running" | "completed" | "failed" | "healing";

export interface CanvasNode {
  id: string;
  type: "prompt" | "subagent" | "code" | "diff" | "artifact";
  x: number;
  y: number;
  data: Record<string, any>;
}

export interface DAGNodeItem {
  id: string;
  label: string;
  role: "commander" | "architect" | "coder" | "qa" | "red_team";
  status: NodeStatus;
  model: string;
  dependencies: string[];
  stdout?: string;
  stderr?: string;
  latencyMs: number;
}

export interface CognitiveStep {
  id: string;
  stepNumber: number;
  title: string;
  phase: "Goal Decomposition" | "AST Navigation" | "Tool Invocation" | "Observation Ingestion" | "Recursive Verification";
  latencyMs: number;
  tokens: number;
  speed: number;
  status: "verified" | "executing" | "pruned";
  details: string[];
}

export interface SpeculativeBranch {
  id: string;
  name: string;
  score: number;
  status: "selected" | "pruned" | "evaluating";
  rationale: string;
  latencyEstimateMs: number;
}

export interface TerminalLog {
  id: string;
  timestamp: string;
  type: "stdout" | "stderr" | "system" | "intervention" | "command";
  content: string;
}

export interface SwarmAgentInfo {
  role: "commander" | "architect" | "coder" | "qa" | "red_team";
  title: string;
  model: string;
  status: "idle" | "planning" | "coding" | "testing" | "auditing" | "healing";
  currentTask: string;
  tokensGenerated: number;
  voteScore: number;
}

export interface DiffItem {
  id: string;
  filePath: string;
  originalCode: string;
  modifiedCode: string;
  staged: boolean;
  astValid: boolean;
}

export interface WorkspaceState {
  activeView: ViewMode;
  selectedModel: ProviderModel;
  routingPolicy: RoutingPolicy;
  isAirGapped: boolean;
  isSidebarOpen: boolean;
  
  // Telemetry
  telemetry: {
    velocity: number;
    costUsd: number;
    vramUsedGb: number;
    vramTotalGb: number;
    ttftMs: number;
    gpuLoadPct: number;
  };

  // Canvas
  canvasNodes: CanvasNode[];
  selectedNodeId: string | null;
  canvasScale: number;
  canvasPan: { x: number; y: number };

  // DAG
  dagNodes: DAGNodeItem[];
  selectedDagNodeId: string | null;

  // Deep Thinking
  timelineSteps: CognitiveStep[];
  speculativeBranches: SpeculativeBranch[];

  // Terminal
  terminalLogs: TerminalLog[];
  currentCommand: string;
  securityTier: "Tier 1: Safe (Auto)" | "Tier 2: Guarded (Preview)" | "Tier 3: Destructive (Approval)";

  // Swarm
  swarmAgents: SwarmAgentInfo[];
  consensusRate: number;

  // Diff
  diffFiles: DiffItem[];

  // Actions
  setActiveView: (view: ViewMode) => void;
  setSelectedModel: (model: ProviderModel) => void;
  setRoutingPolicy: (policy: RoutingPolicy) => void;
  toggleAirGap: () => void;
  toggleSidebar: () => void;
  setSelectedNodeId: (id: string | null) => void;
  setSelectedDagNodeId: (id: string | null) => void;
  setCanvasScale: (scale: number) => void;
  setCanvasPan: (pan: { x: number; y: number }) => void;
  updateCanvasNodePosition: (id: string, x: number, y: number) => void;
  executeCommand: (cmd: string) => void;
  dispatchSwarmTask: (task: string) => void;
  toggleStageDiff: (id: string) => void;
}

export const useWorkspaceStore = create<WorkspaceState>((set, get) => ({
  activeView: "canvas",
  selectedModel: "claude-3-7-sonnet",
  routingPolicy: "reasoning",
  isAirGapped: false,
  isSidebarOpen: true,

  telemetry: {
    velocity: 148.6,
    costUsd: 0.0234,
    vramUsedGb: 14.8,
    vramTotalGb: 24.0,
    ttftMs: 28,
    gpuLoadPct: 64.2,
  },

  canvasScale: 1.0,
  canvasPan: { x: 0, y: 0 },
  selectedNodeId: "node-coder",

  canvasNodes: [
    {
      id: "node-prompt",
      type: "prompt",
      x: 80,
      y: 200,
      data: {
        title: "User Prompt",
        instruction: "Build AST validator and sandboxed ConPTY terminal runner with multi-role swarm consensus",
        status: "completed",
        tokens: 342,
      },
    },
    {
      id: "node-coder",
      type: "subagent",
      x: 520,
      y: 120,
      data: {
        role: "coder",
        model: "claude-3-7-sonnet",
        phase: "Act: Multi-Replace File Synthesis",
        progress: 88,
        activeFile: "crates/xeno-tools/src/ast_validator.rs",
        tokens: 1420,
      },
    },
    {
      id: "node-code-block",
      type: "code",
      x: 960,
      y: 60,
      data: {
        fileName: "ast_validator.rs",
        language: "rust",
        code: `pub fn validate_syntax(&self, path: &Path, code: &str) -> Result<(), ToolError> {\n    match path.extension().and_then(|s| s.to_str()) {\n        Some("rs") => syn::parse_file(code).map(|_| ()).map_err(|e| ToolError::AstParseError(e.to_string())),\n        Some("json") => serde_json::from_str::<serde_json::Value>(code).map(|_| ()).map_err(|e| ToolError::AstParseError(e.to_string())),\n        _ => Ok(()),\n    }\n}`,
      },
    },
    {
      id: "node-diff",
      type: "diff",
      x: 960,
      y: 380,
      data: {
        filePath: "crates/xeno-router/src/privacy.rs",
        diff: `--- original\n+++ replacement\n- pub fn scrub_pii(text: &str) -> String {\n+ pub fn scrub_pii_with_entropy(text: &str, threshold: f64) -> SanitizedResult {`,
      },
    },
  ],

  dagNodes: [
    {
      id: "dag-1",
      label: "Commander: Task Decomposition",
      role: "commander",
      status: "completed",
      model: "claude-3-7-sonnet",
      dependencies: [],
      stdout: "Decomposed task into 3 parallel execution branches with zero circularity.",
      latencyMs: 142,
    },
    {
      id: "dag-2",
      label: "Architect: API & Schema Validation",
      role: "architect",
      status: "completed",
      model: "deepseek-r1",
      dependencies: ["dag-1"],
      stdout: "Verified contracts: XenoAgentStepEvent, XenoDAGNode, and TokenMetrics.",
      latencyMs: 310,
    },
    {
      id: "dag-3",
      label: "Coder: AST Atomic Patching",
      role: "coder",
      status: "running",
      model: "claude-3-7-sonnet",
      dependencies: ["dag-2"],
      stdout: "Applying multi_replace_file_content with exact character substring bounds.",
      latencyMs: 420,
    },
    {
      id: "dag-4",
      label: "QA Tester: Cargo Unit & E2E Tests",
      role: "qa",
      status: "pending",
      model: "local-gguf",
      dependencies: ["dag-3"],
      latencyMs: 0,
    },
    {
      id: "dag-5",
      label: "Red Team: Air-Gap & Secret Scanner",
      role: "red_team",
      status: "pending",
      model: "deepseek-r1",
      dependencies: ["dag-3"],
      latencyMs: 0,
    },
  ],
  selectedDagNodeId: "dag-3",

  timelineSteps: [
    {
      id: "step-1",
      stepNumber: 1,
      title: "Semantic Goal Decomposition & Context Ingestion",
      phase: "Goal Decomposition",
      latencyMs: 110,
      tokens: 420,
      speed: 165.2,
      status: "verified",
      details: [
        "Parsed prompt: 'Implement AST validation and sandboxed ConPTY runner'",
        "Mapped 4 affected crates: xeno-core, xeno-tools, xeno-router, xeno-agent",
        "Configured security gate: Tier 2 Guarded Operations with AST snapshot",
      ],
    },
    {
      id: "step-2",
      stepNumber: 2,
      title: "AST Symbol Indexing & Tree-Sitter Validation",
      phase: "AST Navigation",
      latencyMs: 240,
      tokens: 610,
      speed: 152.0,
      status: "verified",
      details: [
        "Traversed crates/xeno-tools/src/ast_validator.rs",
        "Checked syn parser for Rust, serde_json for JSON, and toml for TOML",
        "Confirmed zero circular symbol references in workspace AST",
      ],
    },
    {
      id: "step-3",
      stepNumber: 3,
      title: "ConPTY Virtual Session Spawn & Job Object Bindings",
      phase: "Tool Invocation",
      latencyMs: 380,
      tokens: 890,
      speed: 148.6,
      status: "executing",
      details: [
        "Invoked tool: terminal_exec with command 'cargo test --workspace'",
        "Bound process tree to Windows Job Object (KILL_ON_JOB_CLOSE enabled)",
        "Capturing real-time stdout/stderr stream without PTY deadlock",
      ],
    },
  ],

  speculativeBranches: [
    {
      id: "branch-a",
      name: "Branch A: Direct In-Memory FFI Binding",
      score: 64.5,
      status: "pruned",
      rationale: "Direct C++ FFI linking creates segfault vulnerability risking UI thread crash.",
      latencyEstimateMs: 45,
    },
    {
      id: "branch-b",
      name: "Branch B: Sandboxed ConPTY + Local Socket IPC",
      score: 98.2,
      status: "selected",
      rationale: "Guarantees process isolation, zero UI crash risk, and clean timeout watchdog recovery.",
      latencyEstimateMs: 18,
    },
  ],

  terminalLogs: [
    {
      id: "log-1",
      timestamp: "18:44:02",
      type: "system",
      content: "[XENO-PTY] Session #pty-8492 initialized (Windows ConPTY + JobObject sandbox)",
    },
    {
      id: "log-2",
      timestamp: "18:44:03",
      type: "command",
      content: "$ cargo test --workspace",
    },
    {
      id: "log-3",
      timestamp: "18:44:05",
      type: "stdout",
      content: "   Compiling xeno-core v0.1.0\n   Compiling xeno-tools v0.1.0\n   Compiling xeno-router v0.1.0\n   Compiling xeno-agent v0.1.0\n    Finished test profile [unoptimized + debuginfo] target(s) in 2.14s",
    },
    {
      id: "log-4",
      timestamp: "18:44:06",
      type: "stdout",
      content: "     Running unittests src/lib.rs (xeno_tools)\ntest ast_validator::tests::test_rust_ast_validation ... ok\ntest pty::tests::test_pty_tier3_rejection ... ok\ntest file_engine::tests::test_multi_replace_and_slice ... ok\ntest python_runner::tests::test_python_runner_inline ... ok",
    },
    {
      id: "log-5",
      timestamp: "18:44:07",
      type: "intervention",
      content: "[XENO AI INTERVENTION] 120+ tests passed cleanly. 0 failures. AST syntax invariant verified.",
    },
  ],
  currentCommand: "",
  securityTier: "Tier 2: Guarded (Preview)",

  swarmAgents: [
    {
      role: "commander",
      title: "Commander Agent",
      model: "Claude 3.7 Sonnet (Thinking)",
      status: "planning",
      currentTask: "Orchestrating task decomposition & token budget allocation",
      tokensGenerated: 1420,
      voteScore: 100,
    },
    {
      role: "architect",
      title: "System Architect",
      model: "DeepSeek R1",
      status: "idle",
      currentTask: "Validating API schemas & topological dependency tree",
      tokensGenerated: 2180,
      voteScore: 98,
    },
    {
      role: "coder",
      title: "Lead Coder",
      model: "Claude 3.7 Sonnet",
      status: "coding",
      currentTask: "Synthesizing atomic AST patches for xeno-tools & xeno-dag",
      tokensGenerated: 4890,
      voteScore: 96,
    },
    {
      role: "qa",
      title: "QA Tester",
      model: "Local Qwen 2.5 7B (GGUF)",
      status: "testing",
      currentTask: "Running automated unit, boundary, and regression tests",
      tokensGenerated: 940,
      voteScore: 100,
    },
    {
      role: "red_team",
      title: "Red-Team Auditor",
      model: "DeepSeek R1",
      status: "auditing",
      currentTask: "Fuzzing socket air-gap & scanning for PII / secret leakages",
      tokensGenerated: 1650,
      voteScore: 100,
    },
  ],
  consensusRate: 98.8,

  diffFiles: [
    {
      id: "diff-1",
      filePath: "crates/xeno-tools/src/ast_validator.rs",
      originalCode: `pub fn validate() {\n    // stub\n}`,
      modifiedCode: `pub fn validate_syntax(&self, path: &Path, code: &str) -> Result<(), ToolError> {\n    match path.extension().and_then(|s| s.to_str()) {\n        Some("rs") => syn::parse_file(code).map(|_| ()).map_err(|e| ToolError::AstParseError(e.to_string())),\n        Some("json") => serde_json::from_str::<serde_json::Value>(code).map(|_| ()).map_err(|e| ToolError::AstParseError(e.to_string())),\n        _ => Ok(()),\n    }\n}`,
      staged: true,
      astValid: true,
    },
    {
      id: "diff-2",
      filePath: "crates/xeno-router/src/privacy.rs",
      originalCode: `pub fn scrub_pii(text: &str) -> String {\n    text.to_string()\n}`,
      modifiedCode: `pub fn scrub_pii_with_entropy(text: &str, threshold: f64) -> SanitizedResult {\n    let patterns = get_secret_patterns();\n    let mut sanitized = text.to_string();\n    for p in patterns {\n        sanitized = p.replace_all(&sanitized, "[REDACTED_SECRET]").to_string();\n    }\n    SanitizedResult { content: sanitized, redaction_count: 1 }\n}`,
      staged: false,
      astValid: true,
    },
  ],

  // Actions
  setActiveView: (view) => set({ activeView: view }),
  setSelectedModel: (model) => set({ selectedModel: model }),
  setRoutingPolicy: (policy) => set({ routingPolicy: policy }),
  toggleAirGap: () => set((s) => ({ isAirGapped: !s.isAirGapped })),
  toggleSidebar: () => set((s) => ({ isSidebarOpen: !s.isSidebarOpen })),
  setSelectedNodeId: (id) => set({ selectedNodeId: id }),
  setSelectedDagNodeId: (id) => set({ selectedDagNodeId: id }),
  setCanvasScale: (scale) => set({ canvasScale: scale }),
  setCanvasPan: (pan) => set({ canvasPan: pan }),
  
  updateCanvasNodePosition: (id, x, y) => {
    set((state) => ({
      canvasNodes: state.canvasNodes.map((n) =>
        n.id === id ? { ...n, x, y } : n
      ),
    }));
  },

  executeCommand: (cmd) => {
    if (!cmd.trim()) return;
    const newLog: TerminalLog = {
      id: `log-${Date.now()}`,
      timestamp: new Date().toLocaleTimeString(),
      type: "command",
      content: `$ ${cmd}`,
    };
    set((state) => ({
      terminalLogs: [...state.terminalLogs, newLog],
      currentCommand: "",
    }));

    // Simulated execution response
    setTimeout(() => {
      const responseLog: TerminalLog = {
        id: `log-res-${Date.now()}`,
        timestamp: new Date().toLocaleTimeString(),
        type: cmd.includes("fail") ? "stderr" : "stdout",
        content: cmd.includes("swarm")
          ? "[SWARM] Spawning 5 autonomous agents (Commander, Architect, Coder, QA, Red-Team)... Task scheduled in DAG."
          : `[EXEC] Command '${cmd}' executed in sandboxed virtual ConPTY (exit code: 0).`,
      };
      set((state) => ({
        terminalLogs: [...state.terminalLogs, responseLog],
      }));
    }, 400);
  },

  dispatchSwarmTask: (task) => {
    set((state) => ({
      activeView: "swarm",
      canvasNodes: [
        ...state.canvasNodes,
        {
          id: `node-${Date.now()}`,
          type: "prompt",
          x: 100,
          y: 400,
          data: {
            title: "Swarm Goal",
            instruction: task,
            status: "running",
            tokens: 120,
          },
        },
      ],
    }));
  },

  toggleStageDiff: (id) => {
    set((state) => ({
      diffFiles: state.diffFiles.map((d) =>
        d.id === id ? { ...d, staged: !d.staged } : d
      ),
    }));
  },
}));
