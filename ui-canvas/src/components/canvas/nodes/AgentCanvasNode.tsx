import React from "react";
import { Bot, Cpu, Zap, Activity, Trash2 } from "lucide-react";
import { useWorkspaceStore } from "../../../stores/workspaceStore";

export interface AgentCanvasNodeProps {
  id: string;
  data: {
    role: string;
    model: string;
    task: string;
    status: "running" | "idle" | "completed";
    progress: number;
    tokensGenerated: number;
  };
  isSelected?: boolean;
  onSelect?: () => void;
}

export const AgentCanvasNode: React.FC<AgentCanvasNodeProps> = ({ id, data, isSelected, onSelect }) => {
  const { removeCanvasNode } = useWorkspaceStore();

  return (
    <div
      onClick={onSelect}
      className={`p-4 rounded-2xl border bg-surface-900/90 backdrop-blur-xl shadow-2xl w-96 text-left transition-all cursor-move select-none ${
        isSelected 
          ? "border-purple-400 glow-purple ring-1 ring-purple-400/50" 
          : "border-border-700 hover:border-purple-500/40"
      }`}
    >
      <div className="flex items-center justify-between mb-2.5 pb-2 border-b border-border-700">
        <div className="flex items-center space-x-2">
          <div className="p-1 rounded-md bg-purple-500/10 text-purple-400">
            <Bot className="w-3.5 h-3.5" />
          </div>
          <span className="text-xs font-bold font-mono text-purple-300">{data.role}</span>
        </div>
        <div className="flex items-center space-x-1.5">
          <span className="flex items-center gap-1 text-[10px] font-mono text-purple-400 bg-purple-950/60 px-2 py-0.5 rounded-full border border-purple-500/30">
            <Activity className="w-3 h-3 animate-pulse" /> {data.progress}%
          </span>
          <button
            onClick={(e) => {
              e.stopPropagation();
              removeCanvasNode(id);
            }}
            className="p-1 rounded hover:bg-surface-800 text-neutral-500 hover:text-rose-400 transition-all"
            title="Delete Node"
          >
            <Trash2 className="w-3 h-3" />
          </button>
        </div>
      </div>

      <div className="space-y-2">
        <div className="flex items-center space-x-1 text-[10px] font-mono text-neutral-400">
          <Cpu className="w-3 h-3 text-neutral-500" />
          <span>{data.model}</span>
        </div>

        <p className="text-xs text-neutral-200 font-mono leading-relaxed bg-surface-950/70 p-3 rounded-xl border border-border-700/60">
          {data.task}
        </p>

        {/* Progress Bar */}
        <div className="w-full bg-surface-950 rounded-full h-1.5 overflow-hidden border border-border-700/50">
          <div
            className="bg-purple-500 h-full transition-all duration-300 glow-purple"
            style={{ width: `${data.progress}%` }}
          />
        </div>
      </div>

      <div className="mt-3 flex items-center justify-between text-[10px] font-mono text-neutral-500">
        <span className="flex items-center gap-1">
          <Zap className="w-3 h-3 text-cyan-400" />
          {data.tokensGenerated} Tokens
        </span>
        <span className="text-purple-400">Active Thread</span>
      </div>
    </div>
  );
};
