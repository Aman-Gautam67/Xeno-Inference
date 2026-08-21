import React from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { Plus, MessageSquare, Bot, FileCode, GitCompare } from "lucide-react";

export const NodePaletteBar: React.FC = () => {
  const { addCanvasNode } = useWorkspaceStore();

  return (
    <div className="absolute top-6 left-1/2 -translate-x-1/2 z-20 flex items-center space-x-1.5 p-1.5 rounded-2xl border border-border-700 bg-surface-900/90 backdrop-blur-xl shadow-2xl">
      <div className="px-2 py-1 flex items-center space-x-1 text-[10px] text-neutral-400 font-mono border-r border-border-700">
        <Plus className="w-3 h-3 text-cyan-400" />
        <span className="font-bold">ADD NODE</span>
      </div>

      <button
        onClick={() => addCanvasNode("prompt")}
        className="flex items-center space-x-1.5 px-3 py-1 rounded-xl bg-surface-800 hover:bg-cyan-500/20 hover:text-cyan-300 text-neutral-300 text-xs font-mono transition-all border border-transparent hover:border-cyan-500/30"
      >
        <MessageSquare className="w-3.5 h-3.5 text-cyan-400" />
        <span>Prompt</span>
      </button>

      <button
        onClick={() => addCanvasNode("subagent")}
        className="flex items-center space-x-1.5 px-3 py-1 rounded-xl bg-surface-800 hover:bg-purple-500/20 hover:text-purple-300 text-neutral-300 text-xs font-mono transition-all border border-transparent hover:border-purple-500/30"
      >
        <Bot className="w-3.5 h-3.5 text-purple-400" />
        <span>Subagent</span>
      </button>

      <button
        onClick={() => addCanvasNode("code")}
        className="flex items-center space-x-1.5 px-3 py-1 rounded-xl bg-surface-800 hover:bg-emerald-500/20 hover:text-emerald-300 text-neutral-300 text-xs font-mono transition-all border border-transparent hover:border-emerald-500/30"
      >
        <FileCode className="w-3.5 h-3.5 text-emerald-400" />
        <span>Code Block</span>
      </button>

      <button
        onClick={() => addCanvasNode("diff")}
        className="flex items-center space-x-1.5 px-3 py-1 rounded-xl bg-surface-800 hover:bg-amber-500/20 hover:text-amber-300 text-neutral-300 text-xs font-mono transition-all border border-transparent hover:border-amber-500/30"
      >
        <GitCompare className="w-3.5 h-3.5 text-amber-400" />
        <span>AST Diff</span>
      </button>
    </div>
  );
};
