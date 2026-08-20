import React from "react";

export interface SubagentNodeProps {
  id: string;
  role: "commander" | "architect" | "coder" | "qa_tester" | "red_team";
  model: string;
  phase: string;
  progress: number;
}

export const SubagentNode: React.FC<SubagentNodeProps> = ({ id, role, model, phase, progress }) => {
  return (
    <div
      data-node-id={id}
      className="p-4 rounded-xl border border-emerald-500/30 bg-neutral-900/90 shadow-2xl backdrop-blur-md w-80 text-left"
    >
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs font-mono uppercase tracking-wider text-emerald-400 font-semibold">{role}</span>
        <span className="text-[10px] px-2 py-0.5 rounded-full bg-emerald-950 text-emerald-300 border border-emerald-800">
          {phase}
        </span>
      </div>
      <div className="text-xs text-neutral-400 font-mono mb-2">MODEL: {model}</div>
      <div className="w-full bg-neutral-800 rounded-full h-1.5 overflow-hidden">
        <div className="bg-emerald-400 h-1.5 rounded-full transition-all duration-300" style={{ width: `${progress}%` }} />
      </div>
    </div>
  );
};
