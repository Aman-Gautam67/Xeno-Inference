import React from "react";

export interface DiffNodeProps {
  id: string;
  filePath: string;
  diffContent: string;
}

export const DiffNode: React.FC<DiffNodeProps> = ({ id, filePath, diffContent }) => {
  return (
    <div
      data-node-id={id}
      className="p-4 rounded-xl border border-amber-500/30 bg-neutral-900/90 shadow-2xl backdrop-blur-md w-96 text-left"
    >
      <div className="text-xs font-mono text-amber-400 mb-2 font-semibold truncate">{filePath}</div>
      <pre className="text-xs font-mono bg-neutral-950 p-2.5 rounded-lg overflow-x-auto text-neutral-300 max-h-48 border border-neutral-800">
        {diffContent.split("\n").map((line, idx) => {
          const colorClass = line.startsWith("+")
            ? "text-emerald-400 bg-emerald-950/40"
            : line.startsWith("-")
            ? "text-rose-400 bg-rose-950/40"
            : "text-neutral-400";
          return (
            <div key={idx} className={`${colorClass} px-1 rounded`}>
              {line}
            </div>
          );
        })}
      </pre>
    </div>
  );
};
