import React from "react";
import { FileCode, Play, Copy, Check } from "lucide-react";

export interface CodeEditorCanvasNodeProps {
  id: string;
  data: {
    fileName: string;
    language: string;
    code: string;
  };
  isSelected?: boolean;
  onSelect?: () => void;
}

export const CodeEditorCanvasNode: React.FC<CodeEditorCanvasNodeProps> = ({ id, data, isSelected, onSelect }) => {
  const [copied, setCopied] = React.useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(data.code);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div
      onClick={onSelect}
      className={`p-4 rounded-2xl border bg-surface-900/95 backdrop-blur-xl shadow-2xl w-[480px] text-left transition-all cursor-move select-none ${
        isSelected 
          ? "border-cyan-400 glow-cyan ring-1 ring-cyan-400/50" 
          : "border-border-700 hover:border-cyan-500/40"
      }`}
    >
      <div className="flex items-center justify-between mb-2.5 pb-2 border-b border-border-700">
        <div className="flex items-center space-x-2">
          <div className="p-1 rounded-md bg-cyan-500/10 text-cyan-400">
            <FileCode className="w-3.5 h-3.5" />
          </div>
          <span className="text-xs font-bold font-mono text-cyan-300">{data.fileName}</span>
          <span className="text-[10px] px-1.5 py-0.2 rounded bg-surface-800 text-neutral-400 uppercase font-mono">
            {data.language}
          </span>
        </div>

        <div className="flex items-center space-x-1.5">
          <button
            onClick={handleCopy}
            className="p-1 rounded hover:bg-surface-800 text-neutral-400 hover:text-neutral-200 transition-all"
            title="Copy Code"
          >
            {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
          </button>
          <button
            className="flex items-center space-x-1 px-2 py-0.5 rounded bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-300 border border-cyan-500/30 text-[10px] font-mono transition-all"
          >
            <Play className="w-3 h-3" />
            <span>RUN</span>
          </button>
        </div>
      </div>

      <pre className="text-xs font-mono bg-surface-950 p-3 rounded-xl overflow-x-auto text-neutral-300 border border-border-700/80 max-h-56 leading-relaxed">
        <code>{data.code}</code>
      </pre>

      <div className="mt-2.5 flex items-center justify-between text-[10px] font-mono text-neutral-500">
        <span>AST Syntax: Syn Passed</span>
        <span className="text-emerald-400">0 compiler warnings</span>
      </div>
    </div>
  );
};
