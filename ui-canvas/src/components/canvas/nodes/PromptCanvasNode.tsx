import React from "react";
import { MessageSquare, Sparkles, CheckCircle2, Clock } from "lucide-react";

export interface PromptCanvasNodeProps {
  id: string;
  data: {
    title: string;
    instruction: string;
    status: "completed" | "running" | "pending";
    tokens: number;
  };
  isSelected?: boolean;
  onSelect?: () => void;
}

export const PromptCanvasNode: React.FC<PromptCanvasNodeProps> = ({ id, data, isSelected, onSelect }) => {
  return (
    <div
      onClick={onSelect}
      className={`p-4 rounded-2xl border bg-surface-900/90 backdrop-blur-xl shadow-2xl w-96 text-left transition-all cursor-move select-none ${
        isSelected 
          ? "border-cyan-400 glow-cyan ring-1 ring-cyan-400/50" 
          : "border-border-700 hover:border-cyan-500/40"
      }`}
    >
      <div className="flex items-center justify-between mb-2.5 pb-2 border-b border-border-700">
        <div className="flex items-center space-x-2">
          <div className="p-1 rounded-md bg-cyan-500/10 text-cyan-400">
            <MessageSquare className="w-3.5 h-3.5" />
          </div>
          <span className="text-xs font-bold font-mono text-cyan-300">{data.title}</span>
        </div>
        <div className="flex items-center space-x-1.5">
          {data.status === "completed" ? (
            <span className="flex items-center gap-1 text-[10px] font-mono text-emerald-400 bg-emerald-950/60 px-2 py-0.5 rounded-full border border-emerald-500/30">
              <CheckCircle2 className="w-3 h-3" /> VERIFIED
            </span>
          ) : (
            <span className="flex items-center gap-1 text-[10px] font-mono text-amber-400 bg-amber-950/60 px-2 py-0.5 rounded-full border border-amber-500/30">
              <Clock className="w-3 h-3 animate-spin" /> RUNNING
            </span>
          )}
        </div>
      </div>

      <p className="text-xs text-neutral-200 font-mono leading-relaxed bg-surface-950/70 p-3 rounded-xl border border-border-700/60">
        {data.instruction}
      </p>

      <div className="mt-3 flex items-center justify-between text-[10px] font-mono text-neutral-500">
        <span>Prompt Tokens: {data.tokens}</span>
        <span className="flex items-center gap-1 text-cyan-400/70">
          <Sparkles className="w-3 h-3" /> PAORV Seed
        </span>
      </div>
    </div>
  );
};
