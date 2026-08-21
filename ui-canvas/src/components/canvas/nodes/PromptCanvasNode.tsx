import React from "react";
import { MessageSquare, Sparkles, CheckCircle2, Clock, Trash2 } from "lucide-react";
import { useWorkspaceStore } from "../../../stores/workspaceStore";

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
            <MessageSquare className="w-3.5 h-3.5" />
          </div>
          <span className="text-xs font-bold font-display tracking-wider text-stone-900 dark:text-stone-100 uppercase">
            {data.title}
          </span>
        </div>
        <div className="flex items-center space-x-1.5">
          {data.status === "completed" ? (
            <span className="flex items-center gap-1 text-[10px] font-mono text-emerald-700 dark:text-emerald-300 bg-emerald-50 dark:bg-emerald-950/60 px-2 py-0.5 rounded-full border border-emerald-200 dark:border-emerald-800">
              <CheckCircle2 className="w-3 h-3 text-emerald-600" /> VERIFIED
            </span>
          ) : (
            <span className="flex items-center gap-1 text-[10px] font-mono text-amber-700 dark:text-amber-300 bg-amber-50 dark:bg-amber-950/60 px-2 py-0.5 rounded-full border border-amber-200 dark:border-amber-800">
              <Clock className="w-3 h-3 animate-spin text-amber-600" /> RUNNING
            </span>
          )}
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

      <p className="text-xs text-stone-700 dark:text-stone-300 font-editorial leading-relaxed bg-stone-50 dark:bg-stone-950 p-3.5 rounded-xl border border-stone-200 dark:border-stone-800">
        "{data.instruction}"
      </p>

      <div className="mt-3 flex items-center justify-between text-[10px] font-mono text-stone-500">
        <span>Prompt Tokens: {data.tokens}</span>
        <span className="flex items-center gap-1 text-stone-600 dark:text-stone-400">
          <Sparkles className="w-3 h-3 text-amber-600 dark:text-amber-400" /> PAORV Seed
        </span>
      </div>
    </div>
  );
};
