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
      className={`p-5 rounded-2xl border bg-white dark:bg-stone-900 shadow-md card-elevation w-96 text-left transition-all cursor-move select-none ${
        isSelected 
          ? "border-stone-900 dark:border-stone-100 ring-2 ring-stone-900/10 dark:ring-stone-100/20" 
          : "border-stone-200 dark:border-stone-800 hover:border-stone-400 dark:hover:border-stone-600"
      }`}
    >
      <div className="flex items-center justify-between mb-3 pb-2.5 border-b border-stone-100 dark:border-stone-800">
        <div className="flex items-center space-x-2">
          <div className="p-1.5 rounded-lg bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300">
            <Bot className="w-3.5 h-3.5" />
          </div>
          <span className="text-xs font-bold font-display tracking-wider text-stone-900 dark:text-stone-100 uppercase">
            {data.role}
          </span>
        </div>
        <div className="flex items-center space-x-1.5">
          <span className="flex items-center gap-1 text-[10px] font-mono text-stone-700 dark:text-stone-300 bg-stone-100 dark:bg-stone-800 px-2 py-0.5 rounded-full border border-stone-200 dark:border-stone-700 font-semibold">
            <Activity className="w-3 h-3 text-emerald-600 dark:text-emerald-400 animate-pulse" /> {data.progress}%
          </span>
          <button
            onClick={(e) => {
              e.stopPropagation();
              removeCanvasNode(id);
            }}
            className="p-1 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-400 hover:text-rose-600 transition-all"
            title="Delete Node"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <div className="space-y-2.5">
        <div className="flex items-center space-x-1.5 text-[11px] font-mono text-stone-500">
          <Cpu className="w-3.5 h-3.5 text-stone-400" />
          <span className="font-semibold text-stone-700 dark:text-stone-300">{data.model}</span>
        </div>

        <p className="text-xs text-stone-700 dark:text-stone-300 font-sans leading-relaxed bg-stone-50 dark:bg-stone-950 p-3 rounded-xl border border-stone-200 dark:border-stone-800">
          {data.task}
        </p>

        {/* Progress Bar */}
        <div className="w-full bg-stone-100 dark:bg-stone-800 rounded-full h-1.5 overflow-hidden">
          <div
            className="bg-stone-900 dark:bg-stone-100 h-full transition-all duration-300 rounded-full"
            style={{ width: `${data.progress}%` }}
          />
        </div>
      </div>

      <div className="mt-3 flex items-center justify-between text-[10px] font-mono text-stone-500">
        <span className="flex items-center gap-1">
          <Zap className="w-3 h-3 text-amber-600 dark:text-amber-400" />
          {data.tokensGenerated.toLocaleString()} Tokens
        </span>
        <span className="text-emerald-600 dark:text-emerald-400 font-semibold">Active Pipeline</span>
      </div>
    </div>
  );
};
