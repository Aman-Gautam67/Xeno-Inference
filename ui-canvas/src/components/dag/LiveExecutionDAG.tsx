import React from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { 
  GitFork, 
  Crown, 
  Box, 
  Code2, 
  FlaskConical, 
  ShieldAlert, 
  CheckCircle2, 
  Clock, 
  Activity, 
  Zap,
  Terminal
} from "lucide-react";

export const LiveExecutionDAG: React.FC = () => {
  const { dagNodes, selectedDagNodeId, setSelectedDagNodeId } = useWorkspaceStore();
  const activeNode = dagNodes.find((n) => n.id === selectedDagNodeId) || dagNodes[0];

  const getRoleIcon = (role: string) => {
    switch (role) {
      case "commander": return <Crown className="w-4 h-4 text-rose-400" />;
      case "architect": return <Box className="w-4 h-4 text-purple-400" />;
      case "coder": return <Code2 className="w-4 h-4 text-cyan-400" />;
      case "qa": return <FlaskConical className="w-4 h-4 text-amber-400" />;
      case "red_team": return <ShieldAlert className="w-4 h-4 text-crimson-400" />;
      default: return <GitFork className="w-4 h-4 text-neutral-400" />;
    }
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "completed":
        return (
          <span className="flex items-center gap-1 text-[10px] text-emerald-400 bg-emerald-950/60 px-2 py-0.5 rounded-full border border-emerald-500/30">
            <CheckCircle2 className="w-3 h-3" /> PASS
          </span>
        );
      case "running":
        return (
          <span className="flex items-center gap-1 text-[10px] text-cyan-400 bg-cyan-950/60 px-2 py-0.5 rounded-full border border-cyan-500/30 glow-cyan">
            <Activity className="w-3 h-3 animate-spin" /> RUNNING
          </span>
        );
      default:
        return (
          <span className="flex items-center gap-1 text-[10px] text-neutral-400 bg-surface-800 px-2 py-0.5 rounded-full border border-border-700">
            <Clock className="w-3 h-3" /> PENDING
          </span>
        );
    }
  };

  return (
    <div className="flex-1 h-[calc(100vh-3.5rem)] flex bg-void overflow-hidden text-xs font-mono">
      {/* Left / Center: Interactive DAG Canvas */}
      <div className="flex-1 p-8 overflow-y-auto canvas-grid-pattern relative flex flex-col items-center space-y-8">
        <div className="w-full max-w-2xl flex items-center justify-between pb-4 border-b border-border-700">
          <div className="flex items-center space-x-2">
            <GitFork className="w-4 h-4 text-cyan-400" />
            <h2 className="text-sm font-bold text-neutral-100 tracking-wider">REAL-TIME EXECUTION DAG</h2>
          </div>
          <span className="text-[10px] px-2 py-0.5 rounded bg-surface-800 border border-border-700 text-neutral-400">
            {dagNodes.length} Graph Nodes • Dynamic Petgraph Engine
          </span>
        </div>

        {/* Nodes Flow */}
        <div className="w-full max-w-2xl space-y-6">
          {dagNodes.map((node, index) => {
            const isSelected = selectedDagNodeId === node.id;
            return (
              <div key={node.id} className="space-y-4">
                <div
                  onClick={() => setSelectedDagNodeId(node.id)}
                  className={`p-4 rounded-2xl border bg-surface-900/90 backdrop-blur-xl shadow-2xl flex items-center justify-between cursor-pointer transition-all ${
                    isSelected
                      ? "border-cyan-400 glow-cyan ring-1 ring-cyan-400/50 bg-surface-850"
                      : "border-border-700 hover:border-cyan-500/40"
                  }`}
                >
                  <div className="flex items-center space-x-3.5">
                    <div className="p-2 rounded-xl bg-surface-800 border border-border-700">
                      {getRoleIcon(node.role)}
                    </div>
                    <div>
                      <div className="font-bold text-neutral-200 text-xs">{node.label}</div>
                      <div className="text-[10px] text-neutral-500 flex items-center gap-2 mt-0.5">
                        <span>Model: {node.model}</span>
                        {node.latencyMs > 0 && <span>• {node.latencyMs}ms</span>}
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center space-x-3">
                    {getStatusBadge(node.status)}
                  </div>
                </div>

                {/* Arrow Connector between nodes */}
                {index < dagNodes.length - 1 && (
                  <div className="flex justify-center">
                    <div className="w-0.5 h-6 bg-gradient-to-b from-cyan-500 to-purple-500 opacity-60" />
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Right Drawer: Node Telemetry & Output Inspector */}
      <div className="w-96 border-l border-border-700 bg-surface-900/95 flex flex-col p-6 space-y-5 overflow-y-auto">
        <div className="pb-3 border-b border-border-700 flex items-center justify-between">
          <span className="font-bold text-neutral-200 uppercase tracking-wider text-xs">Node Inspector</span>
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-surface-800 text-cyan-400 border border-border-700">
            {activeNode.id}
          </span>
        </div>

        <div className="space-y-3 bg-surface-950/70 p-4 rounded-xl border border-border-700/60">
          <div>
            <div className="text-[10px] text-neutral-500 uppercase">Label</div>
            <div className="text-neutral-200 font-semibold">{activeNode.label}</div>
          </div>
          <div className="grid grid-cols-2 gap-2 pt-2 border-t border-border-700">
            <div>
              <div className="text-[10px] text-neutral-500 uppercase">Role</div>
              <div className="text-cyan-400 capitalize">{activeNode.role}</div>
            </div>
            <div>
              <div className="text-[10px] text-neutral-500 uppercase">Status</div>
              <div className="text-emerald-400 capitalize">{activeNode.status}</div>
            </div>
          </div>
          <div className="grid grid-cols-2 gap-2 pt-2 border-t border-border-700">
            <div>
              <div className="text-[10px] text-neutral-500 uppercase">Assigned Model</div>
              <div className="text-purple-400">{activeNode.model}</div>
            </div>
            <div>
              <div className="text-[10px] text-neutral-500 uppercase">Execution Time</div>
              <div className="text-amber-400">{activeNode.latencyMs} ms</div>
            </div>
          </div>
        </div>

        {/* Stdout / Telemetry Output */}
        <div className="space-y-2">
          <div className="flex items-center space-x-1.5 text-neutral-400 text-xs">
            <Terminal className="w-3.5 h-3.5 text-cyan-400" />
            <span>Process Stdout / Artifact</span>
          </div>
          <pre className="p-3.5 rounded-xl bg-surface-950 border border-border-700 text-[11px] text-neutral-300 overflow-x-auto leading-relaxed max-h-60">
            {activeNode.stdout || "Awaiting upstream dependency resolution before execution..."}
          </pre>
        </div>

        {/* Dependencies */}
        <div className="space-y-2 pt-2 border-t border-border-700">
          <div className="text-[10px] text-neutral-500 uppercase">Dependencies</div>
          <div className="flex flex-wrap gap-1.5">
            {activeNode.dependencies.length > 0 ? (
              activeNode.dependencies.map((dep) => (
                <span key={dep} className="px-2 py-0.5 rounded bg-surface-800 border border-border-700 text-neutral-300 text-[10px]">
                  {dep}
                </span>
              ))
            ) : (
              <span className="text-neutral-500 text-[10px]">Root Node (No dependencies)</span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
