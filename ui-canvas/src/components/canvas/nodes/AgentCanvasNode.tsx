import React from "react";
import { Bot, Cpu, Activity, Zap, CheckCircle2 } from "lucide-react";

export interface AgentCanvasNodeProps {
  id: string;
  data: {
    role: string;
    model: string;
    phase: string;
    progress: number;
    activeFile?: string;
    tokens: number;
  };
  isSelected?: boolean;
  onSelect?: () => void;
}

export const AgentCanvasNode: React.FC<AgentCanvasNodeProps> = ({ id, data, isSelected, onSelect }) => {
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
          <span className="text-xs font-bold font-mono text-purple-300 uppercase tracking-wide">
            Subagent: {data.role}
          </span>
        </div>
        <div className="flex items-center space-x-1.5 text-[10px] font-mono text-purple-400 bg-purple-950/60 px-2 py-0.5 rounded-full border border-purple-500/30">
          <Cpu className="w-3 h-3" />
          <span>{data.model}</span>
        </div>
      </div>

      <div className="space-y-2 bg-surface-950/70 p-3 rounded-xl border border-border-700/60 font-mono text-xs">
        <div className="flex items-center justify-between text-neutral-400 text-[11px]">
          <span className="flex items-center gap-1.5">
            <Activity className="w-3 h-3 text-cyan-400 animate-spin" />
            <span>Phase:</span>
          </span>
          <span className="text-cyan-300 font-semibold">{data.phase}</span>
        </div>

        {data.activeFile && (
          <div className="text-[10px] text-neutral-500 truncate">
            Target: <span className="text-neutral-300">{data.activeFile}</span>
          </div>
        )}

        {/* Progress Bar */}
        <div className="space-y-1">
          <div className="flex justify-between text-[10px] text-neutral-500">
            <span>Execution Loop</span>
            <span>{data.progress}%</span>
          </div>
          <div className="w-full h-1.5 bg-surface-800 rounded-full overflow-hidden">
            <div
              className="h-full bg-gradient-to-r from-cyan-500 via-purple-500 to-emerald-400 transition-all duration-300"
              style={{ width: `${data.progress}%` }}
            />
          </div>
        </div>
      </div>

      <div className="mt-3 flex items-center justify-between text-[10px] font-mono text-neutral-500">
        <span className="flex items-center gap-1">
          <Zap className="w-3 h-3 text-cyan-400" />
          {data.tokens} tokens
        </span>
        <span className="text-emerald-400 flex items-center gap-1">
          <CheckCircle2 className="w-3 h-3" /> PAORV Active
        </span>
      </div>
    </div>
  );
};
