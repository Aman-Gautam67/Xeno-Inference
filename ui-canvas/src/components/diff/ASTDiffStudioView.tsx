import React, { useState } from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { FileCode, Check, RotateCcw, ShieldCheck, GitCompare, Sparkles } from "lucide-react";

export const ASTDiffStudioView: React.FC = () => {
  const { diffFiles, toggleStageDiff } = useWorkspaceStore();
  const [selectedDiffId, setSelectedDiffId] = useState(diffFiles[0]?.id || "");

  const activeDiff = diffFiles.find((d) => d.id === selectedDiffId) || diffFiles[0];

  return (
    <div className="flex-1 h-[calc(100vh-3.5rem)] flex bg-void text-xs font-mono overflow-hidden">
      {/* Left: Changed Files List */}
      <div className="w-80 border-r border-border-700 bg-surface-900/95 p-4 space-y-4 flex flex-col">
        <div className="flex items-center justify-between pb-3 border-b border-border-700">
          <div className="flex items-center space-x-2">
            <GitCompare className="w-4 h-4 text-amber-400" />
            <span className="font-bold text-neutral-200">AST MODIFIED FILES</span>
          </div>
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-surface-800 text-neutral-400">
            {diffFiles.length} files
          </span>
        </div>

        <div className="flex-1 space-y-2 overflow-y-auto">
          {diffFiles.map((diff) => {
            const isSelected = diff.id === selectedDiffId;
            return (
              <div
                key={diff.id}
                onClick={() => setSelectedDiffId(diff.id)}
                className={`p-3 rounded-xl border cursor-pointer transition-all space-y-1.5 ${
                  isSelected 
                    ? "bg-surface-850 border-amber-500/50 glow-amber" 
                    : "bg-surface-950/60 border-border-700 hover:border-border-600"
                }`}
              >
                <div className="flex items-center justify-between">
                  <span className="font-bold text-neutral-200 truncate max-w-[180px]">
                    {diff.filePath.split("/").pop()}
                  </span>
                  {diff.staged ? (
                    <span className="text-[9px] px-1.5 py-0.2 rounded bg-emerald-950 text-emerald-400 border border-emerald-500/30">
                      STAGED
                    </span>
                  ) : (
                    <span className="text-[9px] px-1.5 py-0.2 rounded bg-surface-800 text-neutral-500">
                      UNSTAGED
                    </span>
                  )}
                </div>
                <div className="text-[10px] text-neutral-500 truncate">{diff.filePath}</div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Right: Side-by-Side AST Diff Editor */}
      <div className="flex-1 flex flex-col bg-surface-950">
        {/* Editor Top Bar */}
        <div className="h-12 border-b border-border-700 bg-surface-900/90 px-6 flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <FileCode className="w-4 h-4 text-cyan-400" />
            <span className="font-bold text-neutral-200 text-xs">{activeDiff.filePath}</span>
            <span className="flex items-center gap-1 text-[10px] px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
              <ShieldCheck className="w-3 h-3" /> AST Syntax Syn Validated
            </span>
          </div>

          <div className="flex items-center space-x-2">
            <button
              onClick={() => toggleStageDiff(activeDiff.id)}
              className={`flex items-center space-x-1.5 px-3 py-1.5 rounded-lg border text-xs font-semibold transition-all ${
                activeDiff.staged
                  ? "bg-emerald-500/20 text-emerald-300 border-emerald-500/40 glow-emerald"
                  : "bg-surface-800 text-neutral-300 border-border-700 hover:border-emerald-500/40"
              }`}
            >
              <Check className="w-3.5 h-3.5" />
              <span>{activeDiff.staged ? "Staged for Commit" : "Stage Changes"}</span>
            </button>
          </div>
        </div>

        {/* Diff Content View */}
        <div className="flex-1 grid grid-cols-2 divide-x divide-border-700 overflow-y-auto p-4 bg-void">
          {/* Original Version */}
          <div className="p-3 space-y-2">
            <div className="text-[10px] uppercase text-rose-400 font-bold tracking-wider">
              Original AST (Target File)
            </div>
            <pre className="p-3 bg-surface-950 rounded-xl border border-border-700 text-rose-300/80 leading-relaxed overflow-x-auto">
              <code>{activeDiff.originalCode}</code>
            </pre>
          </div>

          {/* Replacement Version */}
          <div className="p-3 space-y-2">
            <div className="text-[10px] uppercase text-emerald-400 font-bold tracking-wider">
              Synthesized AST Replacement
            </div>
            <pre className="p-3 bg-surface-950 rounded-xl border border-border-700 text-emerald-300/90 leading-relaxed overflow-x-auto">
              <code>{activeDiff.modifiedCode}</code>
            </pre>
          </div>
        </div>
      </div>
    </div>
  );
};
