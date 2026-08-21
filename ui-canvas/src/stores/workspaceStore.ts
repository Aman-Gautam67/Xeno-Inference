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
  isShortcutsOpen: boolean;
  isExportOpen: boolean;
  soundEnabled: boolean;
  
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

  // DAG & Execution
  dagNodes: DAGNodeItem[];
  selectedDagNodeId: string | null;

  // Timeline / Deep Thinking
  timelineSteps: CognitiveStep[];
  speculativeBranches: SpeculativeBranch[];

  // Terminal / ConPTY
  terminalLogs: TerminalLog[];
  currentCommand: string;
  securityTier: string;

  // Swarm Council
  swarmAgents: SwarmAgentInfo[];
  consensusRate: number;

  // AST Diff Studio
  diffFiles: DiffItem[];

  // Actions
  setActiveView: (view: ViewMode) => void;
  setSelectedModel: (model: ProviderModel) => void;
  setRoutingPolicy: (policy: RoutingPolicy) => void;
  toggleAirGap: () => void;
  toggleSidebar: () => void;
  toggleShortcuts: () => void;
  toggleExport: () => void;
  toggleSound: () => void;
  setSelectedNodeId: (id: string | null) => void;
  setSelectedDagNodeId: (id: string | null) => void;
  setCanvasScale: (scale: number) => void;
  setCanvasPan: (pan: { x: number; y: number }) => void;
  updateCanvasNodePosition: (id: string, x: number, y: number) => void;
  addCanvasNode: (type: "prompt" | "subagent" | "code" | "diff") => void;
  removeCanvasNode: (id: string) => void;
  executeCommand: (cmd: string) => void;
  dispatchSwarmTask: (task: string) => void;
  toggleStageDiff: (id: string) => void;
  exportSessionJson: () => string;
  importSessionJson: (jsonStr: string) => boolean;
}

export const useWorkspaceStore = create<WorkspaceState>((set, get) => ({
  activeView: "canvas",
  selectedModel: "claude-3-7-sonnet",
  routingPolicy: "reasoning",
  isAirGapped: false,
  isSidebarOpen: true,
  isShortcutsOpen: false,
  isExportOpen: false,
  soundEnabled: true,

  telemetry: {
    velocity: 84.6,
    costUsd: 0.0412,
    vramUsedGb: 14.8,
    vramTotalGb: 24.0,
    ttftMs: 142,
    gpuLoadPct: 78,
  },

  canvasNodes: [
    {
      id: "node-prompt",
      type: "prompt",
      x: 80,
      y: 120,
      data: {
        title: "User Directive",
        instruction: "Build AST validator in Rust with character-exact diff replacements",
        status: "completed",
        tokens: 340,
      },
    },
    {
      id: "node-coder",
      type: "subagent",
      x: 520,
      y: 100,
      data: {
        role: "Coder Agent",
        model: "Claude 3.7 Sonnet (Thinking)",
        task: "Synthesizing AST validation engine in xeno-tools",
        status: "running",
        progress: 82,
        tokensGenerated: 1420,
      },
    },
    {
      id: "node-code-block",
      type: "code",
      x: 980,
      y: 60,
      data: {
        fileName: "ast_validator.rs",
        language: "rust",
        code: `pub fn validate_syntax(path: &Path, code: &str) -> Result<(), ToolError> {\n    match path.extension().and_then(|s| s.to_str()) {\n        Some("rs") => syn::parse_file(code).map(|_| ()).map_err(|e| ToolError::AstParseError(e.to_string())),\n        Some("json") => serde_json::from_str::<serde_json::Value>(code).map(|_| ()).map_err(|e| ToolError::AstParseError(e.to_string())),\n        _ => Ok(()),\n    }\n}`,
      },
    },
    {
      id: "node-diff",
      type: "diff",
      x: 980,
      y: 380,
      data: {
        filePath: "crates/xeno-tools/src/ast_validator.rs",
        diff: `@@ -1,4 +1,8 @@\n-pub fn validate() {\n-    // stub\n-}\n+pub fn validate_syntax(&self, path: &Path, code: &str) -> Result<(), ToolError> {\n+    syn::parse_file(code).map(|_| ()).map_err(|e| ToolError::AstParseError(e.to_string()))\n+}`,
      },
    },
  ],
  selectedNodeId: "node-coder",
  canvasScale: 1.0,
  canvasPan: { x: 0, y: 0 },

  dagNodes: [
    {
      id: "dag-1",
      label: "Commander: Decompose Task",
      role: "commander",
      status: "completed",
      model: "Claude 3.7 Sonnet",
      dependencies: [],
      latencyMs: 310,
      stdout: "[Commander] Decomposed objective into 3 subtasks: AST schema design, tool implementation, and verification test.",
    },
    {
      id: "dag-2",
      label: "Architect: AST Validation Design",
      role: "architect",
      status: "completed",
      model: "DeepSeek R1",
      dependencies: ["dag-1"],
      latencyMs: 620,
      stdout: "[Architect] Syn parse_file contract confirmed with 0 new dependencies.",
    },
    {
      id: "dag-3",
      label: "Coder: Implement syn Parser",
      role: "coder",
      status: "running",
      model: "Claude 3.7 Sonnet",
      dependencies: ["dag-2"],
      latencyMs: 1420,
      stdout: "[Coder] Generated crates/xeno-tools/src/ast_validator.rs with full test coverage.",
    },
    {
      id: "dag-4",
      label: "QA Tester: Unit & Boundary Tests",
      role: "qa",
      status: "pending",
      model: "Qwen 2.5 72B Local",
      dependencies: ["dag-3"],
      latencyMs: 0,
      stdout: "",
    },
    {
      id: "dag-5",
      label: "Red-Team: Air-Gap & Fuzzing Audit",
      role: "red_team",
      status: "pending",
      model: "DeepSeek R1",
      dependencies: ["dag-3"],
      latencyMs: 0,
      stdout: "",
    },
  ],
  selectedDagNodeId: "dag-3",

  timelineSteps: [
    {
      id: "step-1",
      stepNumber: 1,
      title: "Goal Ingestion & Constraint Decomposition",
      phase: "Goal Decomposition",
      latencyMs: 180,
      tokens: 420,
      speed: 92.4,
      status: "verified",
      details: [
        "Ingested user goal: upgrade spatial canvas & execution DAG",
        "Pinned constraints: no breaking Rust API changes, zero CoT leakages, air-gap lock enforcement",
        "Formulated 5-step execution plan across xeno-tools and ui-canvas",
      ],
    },
    {
      id: "step-2",
      stepNumber: 2,
      title: "AST Character Replacement & Multi-Replace Engine",
      phase: "AST Navigation",
      latencyMs: 460,
      tokens: 1120,
      speed: 88.2,
      status: "verified",
      details: [
        "Scanned crates/xeno-tools/src/file_engine.rs for character replacement boundaries",
        "Verified line-bounded substring replacement with rollback stack capability",
        "Confirmed Syn AST validation prevents corrupt files from writing to disk",
      ],
    },
    {
      id: "step-3",
      stepNumber: 3,
      title: "PAORV State Loop & Subagent Dispatch",
      phase: "Tool Invocation",
      latencyMs: 780,
      tokens: 1840,
      speed: 84.6,
      status: "executing",
      details: [
        "Invoked multi_replace_file_content with character exact match",
        "Subscribed to Token Bus streaming chunks at 84.6 tok/s",
        "Streaming live diff projection directly to Spatial Canvas",
      ],
    },
  ],

  speculativeBranches: [
    {
      id: "branch-a",
      name: "Branch A: Pure Syn Parser AST validation",
      score: 96,
      status: "selected",
      rationale: "Eliminates syntax errors before file writes; zero external binary runtime dependencies.",
      latencyEstimateMs: 140,
    },
    {
      id: "branch-b",
      name: "Branch B: Regex Heuristic Pre-validation",
      score: 64,
      status: "pruned",
      rationale: "Pruned: Vulnerable to false positives on multi-line macros and raw string literals.",
      latencyEstimateMs: 45,
    },
  ],

  terminalLogs: [
    {
      id: "tlog-1",
      timestamp: "18:42:01",
      type: "system",
      content: "[SYSTEM] XENO Virtual PTY initialized (Windows ConPTY + Job Object Isolation).",
    },
    {
      id: "tlog-2",
      timestamp: "18:42:05",
      type: "command",
      content: "$ cargo test --workspace",
    },
    {
      id: "tlog-3",
      timestamp: "18:42:12",
      type: "stdout",
      content: "test result: ok. 120 passed; 0 failed; 0 ignored; finished in 1.42s",
    },
    {
      id: "tlog-4",
      timestamp: "18:42:15",
      type: "intervention",
      content: "[SAFETY GUARDIAN] Tier 2 Guarded Action Auto-Approved (Diff snapshot cached with instant rollback).",
    },
  ],
  currentCommand: "",
  securityTier: "Tier 1: Safe Read-Only",

  swarmAgents: [
    {
      role: "commander",
      title: "Council Commander",
      model: "Claude 3.7 Sonnet",
      status: "planning",
      currentTask: "Orchestrating sub-agent execution order in DAG",
      tokensGenerated: 2140,
      voteScore: 98,
    },
    {
      role: "architect",
      title: "System Architect",
      model: "DeepSeek R1",
      status: "planning",
      currentTask: "Verifying cross-crate dependency graphs and schemas",
      tokensGenerated: 1890,
      voteScore: 95,
    },
    {
      role: "coder",
      title: "Lead Coder",
      model: "Claude 3.7 Sonnet",
      status: "coding",
      currentTask: "Synthesizing character-exact AST diffs in xeno-tools",
      tokensGenerated: 4320,
      voteScore: 100,
    },
    {
      role: "qa",
      title: "QA Tester",
      model: "Qwen 2.5 72B Local",
      status: "testing",
      currentTask: "Running 120+ cargo tests and boundary condition sweeps",
      tokensGenerated: 1420,
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
  toggleShortcuts: () => set((s) => ({ isShortcutsOpen: !s.isShortcutsOpen })),
  toggleExport: () => set((s) => ({ isExportOpen: !s.isExportOpen })),
  toggleSound: () => set((s) => ({ soundEnabled: !s.soundEnabled })),
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

  addCanvasNode: (type) => {
    const id = `node-${type}-${Date.now()}`;
    let data: Record<string, any> = {};
    const pan = get().canvasPan;
    const scale = get().canvasScale;
    const x = (-pan.x + 300) / scale;
    const y = (-pan.y + 200) / scale;

    if (type === "prompt") {
      data = {
        title: "New Instruction",
        instruction: "Enter directive here...",
        status: "pending",
        tokens: 0,
      };
    } else if (type === "subagent") {
      data = {
        role: "Specialist Subagent",
        model: "Claude 3.7 Sonnet",
        task: "Autonomous task execution",
        status: "planning",
        progress: 0,
        tokensGenerated: 0,
      };
    } else if (type === "code") {
      data = {
        fileName: "new_file.rs",
        language: "rust",
        code: "// Write code here...\npub fn solve() {\n}\n",
      };
    } else if (type === "diff") {
      data = {
        filePath: "src/modified.rs",
        diff: "@@ -1,1 +1,2 @@\n-old code\n+new code",
      };
    }

    set((state) => ({
      canvasNodes: [
        ...state.canvasNodes,
        { id, type, x, y, data },
      ],
      selectedNodeId: id,
    }));
  },

  removeCanvasNode: (id) => {
    set((state) => ({
      canvasNodes: state.canvasNodes.filter((n) => n.id !== id),
      selectedNodeId: state.selectedNodeId === id ? null : state.selectedNodeId,
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

  exportSessionJson: () => {
    const s = get();
    const snapshot = {
      timestamp: new Date().toISOString(),
      activeView: s.activeView,
      selectedModel: s.selectedModel,
      routingPolicy: s.routingPolicy,
      isAirGapped: s.isAirGapped,
      telemetry: s.telemetry,
      canvasNodes: s.canvasNodes,
      dagNodes: s.dagNodes,
      timelineSteps: s.timelineSteps,
      speculativeBranches: s.speculativeBranches,
      swarmAgents: s.swarmAgents,
      diffFiles: s.diffFiles,
    };
    return JSON.stringify(snapshot, null, 2);
  },

  importSessionJson: (jsonStr) => {
    try {
      const parsed = JSON.parse(jsonStr);
      if (parsed.canvasNodes && parsed.dagNodes) {
        set({
          canvasNodes: parsed.canvasNodes || [],
          dagNodes: parsed.dagNodes || [],
          timelineSteps: parsed.timelineSteps || [],
          speculativeBranches: parsed.speculativeBranches || [],
          swarmAgents: parsed.swarmAgents || [],
          diffFiles: parsed.diffFiles || [],
          selectedModel: parsed.selectedModel || "claude-3-7-sonnet",
        });
        return true;
      }
      return false;
    } catch {
      return false;
    }
  },
}));
