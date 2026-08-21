import React from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { Plus, MessageSquare, Bot, FileCode, GitCompare } from "lucide-react";

export const NodePaletteBar: React.FC = () => {
  const { addCanvasNode } = useWorkspaceStore();

  return (
    <div className="absolute top-6 left-1/2 -translate-x-1/2 z-20 flex items-center space-x-1.5 p-1.5 rounded-2xl border border-stone-200 dark:border-stone-800 bg-white/95 dark:bg-stone-900/95 backdrop-blur-xl shadow-lg">
      <div className="px-2.5 py-1 flex items-center space-x-1 text-[10px] text-stone-500 font-mono border-r border-stone-200 dark:border-stone-800 font-bold">
        <Plus className="w-3 h-3 text-stone-700 dark:text-stone-300" />
        <span>ADD NODE</span>
      </div>

      <button
        onClick={() => addCanvasNode("prompt")}
        className="flex items-center space-x-1.5 px-3 py-1 rounded-xl hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-700 dark:text-stone-300 text-xs font-mono transition-all font-semibold"
      >
        <MessageSquare className="w-3.5 h-3.5 text-stone-500" />
        <span>Prompt</span>
      </button>

      <button
        onClick={() => addCanvasNode("subagent")}
        className="flex items-center space-x-1.5 px-3 py-1 rounded-xl hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-700 dark:text-stone-300 text-xs font-mono transition-all font-semibold"
      >
        <Bot className="w-3.5 h-3.5 text-purple-600 dark:text-purple-400" />
        <span>Subagent</span>
      </button>

      <button
        onClick={() => addCanvasNode("code")}
        className="flex items-center space-x-1.5 px-3 py-1 rounded-xl hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-700 dark:text-stone-300 text-xs font-mono transition-all font-semibold"
      >
        <FileCode className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" />
        <span>Code</span>
      </button>

      <button
        onClick={() => addCanvasNode("diff")}
        className="flex items-center space-x-1.5 px-3 py-1 rounded-xl hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-700 dark:text-stone-300 text-xs font-mono transition-all font-semibold"
      >
        <GitCompare className="w-3.5 h-3.5 text-amber-600 dark:text-amber-400" />
        <span>AST Diff</span>
      </button>
    </div>
  );
};
