import React from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { Compass } from "lucide-react";

export const CanvasMiniMap: React.FC = () => {
  const { canvasNodes, selectedNodeId, setSelectedNodeId, canvasPan, canvasScale } = useWorkspaceStore();

  const mapWidth = 140;
  const mapHeight = 90;
  const scaleFactor = 0.08;

  return (
    <div className="absolute bottom-28 left-6 z-20 p-2 rounded-2xl border border-border-700 bg-surface-900/90 backdrop-blur-xl shadow-2xl space-y-1 select-none">
      <div className="flex items-center space-x-1 text-[9px] text-neutral-400 font-mono">
        <Compass className="w-3 h-3 text-cyan-400" />
        <span className="font-bold">RADAR MAP</span>
      </div>

      <div
        className="relative bg-surface-950 rounded-xl border border-border-700/80 overflow-hidden"
        style={{ width: `${mapWidth}px`, height: `${mapHeight}px` }}
      >
        {/* Node Indicators on Radar */}
        {canvasNodes.map((node) => {
          const isSelected = selectedNodeId === node.id;
          const x = Math.min(Math.max((node.x + 200) * scaleFactor, 4), mapWidth - 10);
          const y = Math.min(Math.max((node.y + 100) * scaleFactor, 4), mapHeight - 10);

          let dotColor = "bg-cyan-400";
          if (node.type === "subagent") dotColor = "bg-purple-400";
          if (node.type === "code") dotColor = "bg-emerald-400";
          if (node.type === "diff") dotColor = "bg-amber-400";

          return (
            <div
              key={node.id}
              onClick={(e) => {
                e.stopPropagation();
                setSelectedNodeId(node.id);
              }}
              style={{ left: `${x}px`, top: `${y}px` }}
              className={`absolute w-2 h-2 rounded-full cursor-pointer transition-all ${dotColor} ${
                isSelected ? "ring-2 ring-white scale-125 glow-cyan" : "opacity-80 hover:scale-110"
              }`}
              title={`${node.type}: ${node.id}`}
            />
          );
        })}

        {/* Viewport Box */}
        <div
          className="absolute border border-cyan-400/40 bg-cyan-500/5 rounded pointer-events-none"
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
