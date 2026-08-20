import React from "react";

export interface PromptNodeProps {
  id: string;
  label: string;
  content: string;
  status: "pending" | "running" | "completed";
}

export const PromptNode: React.FC<PromptNodeProps> = ({ id, label, content, status }) => {
  return (
    <div
      data-node-id={id}
      className="p-4 rounded-xl border border-cyan-500/30 bg-neutral-900/90 shadow-2xl backdrop-blur-md w-80 text-left"
    >
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs font-mono uppercase tracking-wider text-cyan-400 font-semibold">{label}</span>
        <span className="text-[10px] px-2 py-0.5 rounded-full bg-cyan-950 text-cyan-300 border border-cyan-800">
          {status}
        </span>
      </div>
      <p className="text-sm text-neutral-200 leading-relaxed font-sans">{content}</p>
    </div>
  );
};
