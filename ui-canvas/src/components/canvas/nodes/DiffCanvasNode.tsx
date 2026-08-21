import React from "react";
import { GitCompare, RotateCcw, Check, Trash2 } from "lucide-react";
import { useWorkspaceStore } from "../../../stores/workspaceStore";

export interface DiffCanvasNodeProps {
  id: string;
  data: {
    filePath: string;
    diff: string;
  };
  isSelected?: boolean;
  onSelect?: () => void;
}

export const DiffCanvasNode: React.FC<DiffCanvasNodeProps> = ({ id, data, isSelected, onSelect }) => {
  const { removeCanvasNode } = useWorkspaceStore();

  return (
    <div
      onClick={onSelect}
      className={`p-5 rounded-2xl border bg-white dark:bg-stone-900 shadow-md card-elevation w-[480px] text-left transition-all cursor-move select-none ${
        isSelected 
          ? "border-stone-900 dark:border-stone-100 ring-2 ring-stone-900/10 dark:ring-stone-100/20" 
          : "border-stone-200 dark:border-stone-800 hover:border-stone-400 dark:hover:border-stone-600"
      }`}
    >
      <div className="flex items-center justify-between mb-3 pb-2.5 border-b border-stone-100 dark:border-stone-800">
        <div className="flex items-center space-x-2">
          <div className="p-1.5 rounded-lg bg-stone-100 dark:bg-stone-800 text-amber-700 dark:text-amber-400">
            <GitCompare className="w-3.5 h-3.5" />
          </div>
          <span className="text-xs font-bold font-mono text-stone-900 dark:text-stone-100 truncate max-w-[240px]">
            {data.filePath}
          </span>
        </div>

        <div className="flex items-center space-x-1.5">
          <button
            className="flex items-center space-x-1 px-2 py-0.5 rounded-lg bg-stone-100 dark:bg-stone-800 hover:bg-stone-200 dark:hover:bg-stone-700 text-stone-700 dark:text-stone-300 border border-stone-200 dark:border-stone-700 text-[10px] font-mono transition-all"
            title="Rollback AST Patch"
          >
            <RotateCcw className="w-3 h-3" />
            <span>Rollback</span>
          </button>
          <button
            className="flex items-center space-x-1 px-2.5 py-0.5 rounded-lg bg-emerald-50 dark:bg-emerald-950/60 hover:bg-emerald-100 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800 text-[10px] font-mono font-bold transition-all"
            title="Stage AST Modification"
          >
            <Check className="w-3 h-3" />
            <span>Stage</span>
          </button>
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

      <pre className="text-xs font-mono bg-stone-50 dark:bg-stone-950 p-3.5 rounded-xl overflow-x-auto text-stone-800 dark:text-stone-200 border border-stone-200 dark:border-stone-800 max-h-48 leading-relaxed">
        {data.diff.split("\n").map((line, idx) => {
          const isAdd = line.startsWith("+");
          const isDel = line.startsWith("-");
          return (
            <div
              key={idx}
              className={`px-1 rounded ${
                isAdd 
                  ? "text-emerald-700 dark:text-emerald-300 bg-emerald-100/60 dark:bg-emerald-950/50" 
                  : isDel 
                  ? "text-rose-700 dark:text-rose-300 bg-rose-100/60 dark:bg-rose-950/50" 
                  : "text-stone-600 dark:text-stone-400"
              }`}
            >
              {line}
            </div>
          );
        })}
      </pre>

      <div className="mt-3 flex items-center justify-between text-[10px] font-mono text-stone-500">
        <span>Atomic Character Replace: Exact Match</span>
        <span className="text-amber-700 dark:text-amber-400 font-semibold">Preview Staged</span>
      </div>
    </div>
  );
};
