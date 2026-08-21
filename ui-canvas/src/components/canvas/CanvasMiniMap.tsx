import React from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { Compass } from "lucide-react";

export const CanvasMiniMap: React.FC = () => {
  const { canvasNodes, selectedNodeId, setSelectedNodeId, canvasPan, canvasScale } = useWorkspaceStore();

  const mapWidth = 140;
  const mapHeight = 90;
  const scaleFactor = 0.08;

  return (
    <div className="absolute bottom-28 left-6 z-20 p-2 rounded-2xl border border-stone-200 dark:border-stone-800 bg-white/95 dark:bg-stone-900/95 backdrop-blur-xl shadow-lg space-y-1 select-none">
      <div className="flex items-center space-x-1 text-[9px] text-stone-500 font-mono">
        <Compass className="w-3 h-3 text-stone-700 dark:text-stone-300" />
        <span className="font-bold">RADAR MAP</span>
      </div>

      <div
        className="relative bg-stone-100 dark:bg-stone-950 rounded-xl border border-stone-200 dark:border-stone-800 overflow-hidden"
        style={{ width: `${mapWidth}px`, height: `${mapHeight}px` }}
      >
        {/* Node Indicators on Radar */}
        {canvasNodes.map((node) => {
          const isSelected = selectedNodeId === node.id;
          const x = Math.min(Math.max((node.x + 200) * scaleFactor, 4), mapWidth - 10);
          const y = Math.min(Math.max((node.y + 100) * scaleFactor, 4), mapHeight - 10);

          let dotColor = "bg-stone-700 dark:bg-stone-300";
          if (node.type === "subagent") dotColor = "bg-purple-600 dark:bg-purple-400";
          if (node.type === "code") dotColor = "bg-emerald-600 dark:bg-emerald-400";
          if (node.type === "diff") dotColor = "bg-amber-600 dark:bg-amber-400";

          return (
            <div
              key={node.id}
              onClick={(e) => {
                e.stopPropagation();
                setSelectedNodeId(node.id);
              }}
              style={{ left: `${x}px`, top: `${y}px` }}
              className={`absolute w-2 h-2 rounded-full cursor-pointer transition-all ${dotColor} ${
                isSelected ? "ring-2 ring-stone-900 dark:ring-stone-100 scale-125 shadow-sm" : "opacity-80 hover:scale-110"
              }`}
              title={`${node.type}: ${node.id}`}
            />
          );
        })}

        {/* Viewport Box */}
        <div
          className="absolute border border-stone-500/50 bg-stone-500/10 rounded pointer-events-none"
          style={{
            left: `${Math.max((-canvasPan.x * scaleFactor) / canvasScale, 0)}px`,
            top: `${Math.max((-canvasPan.y * scaleFactor) / canvasScale, 0)}px`,
            width: `${Math.min(mapWidth / canvasScale, mapWidth)}px`,
            height: `${Math.min(mapHeight / canvasScale, mapHeight)}px`,
          }}
        />
      </div>
    </div>
  );
};
