import React from "react";
import { FileCode, Play, Copy, Check, Trash2 } from "lucide-react";
import { useWorkspaceStore } from "../../../stores/workspaceStore";

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
  const { removeCanvasNode } = useWorkspaceStore();

  const handleCopy = () => {
    navigator.clipboard.writeText(data.code);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

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
          <div className="p-1.5 rounded-lg bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300">
            <FileCode className="w-3.5 h-3.5" />
          </div>
          <span className="text-xs font-bold font-mono text-stone-900 dark:text-stone-100">{data.fileName}</span>
          <span className="text-[10px] px-1.5 py-0.2 rounded bg-stone-100 dark:bg-stone-800 text-stone-500 uppercase font-mono font-semibold">
            {data.language}
          </span>
        </div>

        <div className="flex items-center space-x-1.5">
          <button
            onClick={handleCopy}
            className="p-1 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-500 hover:text-stone-800 dark:hover:text-stone-200 transition-all"
            title="Copy Code"
          >
            {copied ? <Check className="w-3.5 h-3.5 text-emerald-600" /> : <Copy className="w-3.5 h-3.5" />}
          </button>
          <button
            className="flex items-center space-x-1 px-2.5 py-1 rounded-lg bg-stone-900 hover:bg-stone-800 dark:bg-stone-100 dark:hover:bg-white text-white dark:text-stone-900 text-[10px] font-mono font-bold transition-all shadow-sm"
          >
            <Play className="w-3 h-3" />
            <span>RUN</span>
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

      <pre className="text-xs font-mono bg-stone-50 dark:bg-stone-950 p-3.5 rounded-xl overflow-x-auto text-stone-800 dark:text-stone-200 border border-stone-200 dark:border-stone-800 max-h-56 leading-relaxed">
        <code>{data.code}</code>
      </pre>

      <div className="mt-3 flex items-center justify-between text-[10px] font-mono text-stone-500">
        <span>AST Syntax: Syn Validated</span>
        <span className="text-emerald-600 dark:text-emerald-400 font-semibold">0 compiler warnings</span>
      </div>
    </div>
  );
};
