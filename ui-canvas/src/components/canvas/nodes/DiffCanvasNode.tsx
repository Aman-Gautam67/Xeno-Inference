import React from "react";
import { GitCompare, RotateCcw, Check } from "lucide-react";

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
  return (
    <div
      onClick={onSelect}
      className={`p-4 rounded-2xl border bg-surface-900/95 backdrop-blur-xl shadow-2xl w-[480px] text-left transition-all cursor-move select-none ${
        isSelected 
          ? "border-amber-400 glow-amber ring-1 ring-amber-400/50" 
          : "border-border-700 hover:border-amber-500/40"
      }`}
    >
      <div className="flex items-center justify-between mb-2.5 pb-2 border-b border-border-700">
        <div className="flex items-center space-x-2">
          <div className="p-1 rounded-md bg-amber-500/10 text-amber-400">
            <GitCompare className="w-3.5 h-3.5" />
          </div>
          <span className="text-xs font-bold font-mono text-amber-300 truncate max-w-[280px]">
            {data.filePath}
          </span>
        </div>

        <div className="flex items-center space-x-1.5">
          <button
            className="flex items-center space-x-1 px-2 py-0.5 rounded bg-surface-800 hover:bg-surface-700 text-neutral-300 border border-border-700 text-[10px] font-mono transition-all"
            title="Rollback AST Patch"
          >
            <RotateCcw className="w-3 h-3" />
            <span>Rollback</span>
          </button>
          <button
            className="flex items-center space-x-1 px-2 py-0.5 rounded bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 text-[10px] font-mono transition-all"
            title="Stage AST Modification"
          >
            <Check className="w-3 h-3" />
            <span>Stage</span>
          </button>
        </div>
      </div>

      <pre className="text-xs font-mono bg-surface-950 p-3 rounded-xl overflow-x-auto text-neutral-300 border border-border-700/80 max-h-48 leading-relaxed">
        {data.diff.split("\n").map((line, idx) => {
          const isAdd = line.startsWith("+");
          const isDel = line.startsWith("-");
          return (
            <div
              key={idx}
              className={`px-1 rounded ${
                isAdd 
                  ? "text-emerald-400 bg-emerald-950/40" 
                  : isDel 
                  ? "text-rose-400 bg-rose-950/40" 
                  : "text-neutral-400"
              }`}
            >
              {line}
            </div>
          );
        })}
      </pre>

      <div className="mt-2.5 flex items-center justify-between text-[10px] font-mono text-neutral-500">
        <span>Atomic Character Replace: Exact Match</span>
        <span className="text-amber-400">Preview Staged</span>
      </div>
    </div>
  );
};
