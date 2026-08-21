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
      case "commander": return <Crown className="w-4 h-4 text-stone-700 dark:text-stone-300" />;
      case "architect": return <Box className="w-4 h-4 text-purple-600 dark:text-purple-400" />;
      case "coder": return <Code2 className="w-4 h-4 text-stone-900 dark:text-stone-100" />;
      case "qa": return <FlaskConical className="w-4 h-4 text-amber-600 dark:text-amber-400" />;
      case "red_team": return <ShieldAlert className="w-4 h-4 text-rose-600 dark:text-rose-400" />;
      default: return <GitFork className="w-4 h-4 text-stone-500" />;
    }
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "completed":
        return (
          <span className="flex items-center gap-1 text-[10px] text-emerald-700 dark:text-emerald-300 bg-emerald-50 dark:bg-emerald-950/60 px-2 py-0.5 rounded-full border border-emerald-200 dark:border-emerald-800 font-bold">
            <CheckCircle2 className="w-3 h-3 text-emerald-600" /> PASS
          </span>
        );
      case "running":
        return (
          <span className="flex items-center gap-1 text-[10px] text-amber-700 dark:text-amber-300 bg-amber-50 dark:bg-amber-950/60 px-2 py-0.5 rounded-full border border-amber-200 dark:border-amber-800 font-bold">
            <Activity className="w-3 h-3 animate-spin text-amber-600" /> RUNNING
          </span>
        );
      default:
        return (
          <span className="flex items-center gap-1 text-[10px] text-stone-500 bg-stone-100 dark:bg-stone-800 px-2 py-0.5 rounded-full border border-stone-200 dark:border-stone-700">
            <Clock className="w-3 h-3" /> PENDING
          </span>
        );
    }
  };

  return (
    <div className="flex-1 h-[calc(100vh-3.5rem)] flex bg-stone-50 dark:bg-stone-950 overflow-hidden text-xs font-mono transition-colors duration-200">
      {/* Left / Center: Interactive DAG Canvas */}
      <div className="flex-1 p-8 overflow-y-auto canvas-grid-pattern relative flex flex-col items-center space-y-8">
        <div className="w-full max-w-2xl flex items-center justify-between pb-4 border-b border-stone-200 dark:border-stone-800">
          <div className="flex items-center space-x-2">
            <GitFork className="w-4 h-4 text-stone-700 dark:text-stone-300" />
            <span className="font-display font-bold text-sm tracking-wider uppercase text-stone-900 dark:text-stone-100">
              Live Petgraph Execution DAG
            </span>
          </div>
          <div className="flex items-center space-x-2 text-[10px] text-stone-500">
            <span>5 Nodes Total</span>
            <span>•</span>
            <span className="text-emerald-600 dark:text-emerald-400 font-bold">Acyclic Verified</span>
          </div>
        </div>

        {/* Node Hierarchy Chain */}
        <div className="w-full max-w-2xl space-y-6">
          {dagNodes.map((node, index) => {
            const isSelected = node.id === activeNode?.id;
            return (
              <div key={node.id} className="relative flex flex-col items-center">
                {/* Node Card */}
                <div
                  onClick={() => setSelectedDagNodeId(node.id)}
                  className={`w-full p-4 rounded-2xl border bg-white dark:bg-stone-900 shadow-sm card-elevation cursor-pointer transition-all ${
                    isSelected 
                      ? "border-stone-900 dark:border-stone-100 ring-2 ring-stone-900/10 dark:ring-stone-100/20" 
                      : "border-stone-200 dark:border-stone-800 hover:border-stone-400 dark:hover:border-stone-600"
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                      <div className="p-2 rounded-xl bg-stone-100 dark:bg-stone-800">
                        {getRoleIcon(node.role)}
                      </div>
                      <div>
                        <div className="font-bold text-stone-900 dark:text-stone-100 text-xs font-display">{node.label}</div>
                        <div className="text-[10px] text-stone-500">{node.model}</div>
                      </div>
                    </div>
                    <div className="flex items-center space-x-3">
                      {node.latencyMs > 0 && (
                        <span className="text-[10px] text-stone-500 font-mono">
                          {node.latencyMs}ms
                        </span>
                      )}
                      {getStatusBadge(node.status)}
                    </div>
                  </div>
                </div>

                {/* Downward Connector Arrow */}
                {index < dagNodes.length - 1 && (
                  <div className="h-6 w-[2px] bg-stone-300 dark:bg-stone-700 my-1 relative">
                    <div className="absolute -bottom-1 -left-[3px] w-2 h-2 border-r-2 border-b-2 border-stone-400 dark:border-stone-600 rotate-45" />
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Right: Node Telemetry & stdout Inspector Drawer */}
      <div className="w-96 border-l border-stone-200 dark:border-stone-800 bg-white/95 dark:bg-stone-900/95 p-5 space-y-5 flex flex-col justify-between overflow-y-auto">
        <div className="space-y-4">
          <div className="flex items-center justify-between pb-3 border-b border-stone-200 dark:border-stone-800">
            <span className="font-display font-bold uppercase tracking-wider text-stone-800 dark:text-stone-200 text-xs">
              Node Inspector
            </span>
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-stone-100 dark:bg-stone-800 text-stone-600 dark:text-stone-400 font-mono">
              {activeNode?.id}
            </span>
          </div>

          <div className="space-y-2">
            <div className="text-stone-500 uppercase text-[10px] font-bold">Node Label</div>
            <div className="p-3 rounded-xl bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 text-stone-900 dark:text-stone-100 font-semibold">
              {activeNode?.label}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-2">
            <div className="p-3 rounded-xl bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 space-y-1">
              <div className="text-stone-500 uppercase text-[9px] font-bold">Role</div>
              <div className="font-bold text-stone-900 dark:text-stone-100 capitalize">{activeNode?.role}</div>
            </div>
            <div className="p-3 rounded-xl bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 space-y-1">
              <div className="text-stone-500 uppercase text-[9px] font-bold">Latency</div>
              <div className="font-bold text-stone-900 dark:text-stone-100">{activeNode?.latencyMs} ms</div>
            </div>
          </div>

          <div className="space-y-2">
            <div className="flex items-center space-x-1.5 text-stone-500 uppercase text-[10px] font-bold">
              <Terminal className="w-3.5 h-3.5" />
              <span>Node Stream Stdout</span>
            </div>
            <pre className="p-3.5 rounded-xl bg-stone-50 dark:bg-stone-950 border border-stone-200 dark:border-stone-800 text-[11px] text-stone-700 dark:text-stone-300 max-h-48 overflow-y-auto leading-relaxed font-mono">
              {activeNode?.stdout || "// Awaiting node execution..."}
            </pre>
          </div>
        </div>

        <div className="pt-3 border-t border-stone-200 dark:border-stone-800 text-[10px] text-stone-500 flex items-center justify-between">
          <span>Topological Stage: Active</span>
          <span className="text-emerald-600 dark:text-emerald-400 font-bold">Syn Validated</span>
        </div>
      </div>
    </div>
  );
};
