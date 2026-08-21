import React, { useRef, useState } from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { PromptCanvasNode } from "./nodes/PromptCanvasNode";
import { AgentCanvasNode } from "./nodes/AgentCanvasNode";
import { CodeEditorCanvasNode } from "./nodes/CodeEditorCanvasNode";
import { DiffCanvasNode } from "./nodes/DiffCanvasNode";
import { NodePaletteBar } from "./NodePaletteBar";
import { CanvasMiniMap } from "./CanvasMiniMap";
import { ZoomIn, ZoomOut, Maximize2, Move } from "lucide-react";

export const SpatialCanvas: React.FC = () => {
  const { 
    canvasNodes, 
    selectedNodeId, 
    setSelectedNodeId, 
    canvasScale, 
    setCanvasScale, 
    canvasPan, 
    setCanvasPan,
    updateCanvasNodePosition
  } = useWorkspaceStore();

  const containerRef = useRef<HTMLDivElement>(null);
  const [isPanning, setIsPanning] = useState(false);
  const [panStart, setPanStart] = useState({ x: 0, y: 0 });
  const [draggingNodeId, setDraggingNodeId] = useState<string | null>(null);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });

  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.target === containerRef.current || (e.target as HTMLElement).tagName === "svg") {
      setIsPanning(true);
      setPanStart({ x: e.clientX - canvasPan.x, y: e.clientY - canvasPan.y });
    }
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (isPanning) {
      setCanvasPan({
        x: e.clientX - panStart.x,
        y: e.clientY - panStart.y,
      });
    } else if (draggingNodeId) {
      const newX = (e.clientX - canvasPan.x - dragOffset.x) / canvasScale;
      const newY = (e.clientY - canvasPan.y - dragOffset.y) / canvasScale;
      updateCanvasNodePosition(draggingNodeId, Math.round(newX), Math.round(newY));
    }
  };

  const handleMouseUp = () => {
    setIsPanning(false);
    setDraggingNodeId(null);
  };

  const startNodeDrag = (nodeId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setSelectedNodeId(nodeId);
    setDraggingNodeId(nodeId);
    const node = canvasNodes.find((n) => n.id === nodeId);
    if (node) {
      setDragOffset({
        x: e.clientX - (node.x * canvasScale + canvasPan.x),
        y: e.clientY - (node.y * canvasScale + canvasPan.y),
      });
    }
  };

  // Render dynamic cubic bezier paths connecting nodes sequentially
  const renderConnectors = () => {
    if (canvasNodes.length < 2) return null;
    const paths: React.ReactNode[] = [];

    for (let i = 0; i < canvasNodes.length - 1; i++) {
      const source = canvasNodes[i];
      const target = canvasNodes[i + 1];

      // Approximate center connection points
      const sx = source.x + 360;
      const sy = source.y + 70;
      const tx = target.x;
      const ty = target.y + 70;

      const dx = Math.abs(tx - sx) * 0.5;
      const pathData = `M ${sx} ${sy} C ${sx + dx} ${sy}, ${tx - dx} ${ty}, ${tx} ${ty}`;

      paths.push(
        <g key={`edge-${source.id}-${target.id}`}>
          <path
            d={pathData}
            fill="none"
            className="stroke-stone-300 dark:stroke-stone-700"
            strokeWidth="2"
            strokeDasharray="4 4"
          />
          <circle
            cx={tx}
            cy={ty}
            r="4"
            className="fill-stone-600 dark:fill-stone-400"
          />
        </g>
      );
    }
    return paths;
  };

  return (
    <div
      ref={containerRef}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      className="flex-1 relative w-full h-[calc(100vh-3.5rem)] overflow-hidden canvas-grid-pattern cursor-grab active:cursor-grabbing bg-stone-100/70 dark:bg-stone-950 transition-colors duration-200"
    >
      {/* Floating Node Palette */}
      <NodePaletteBar />

      {/* Radar Mini-Map */}
      <CanvasMiniMap />

      {/* Zoom Controls Overlay */}
      <div className="absolute bottom-28 right-6 z-20 flex items-center space-x-1 p-1.5 rounded-2xl border border-stone-200 dark:border-stone-800 bg-white/95 dark:bg-stone-900/95 backdrop-blur-xl shadow-lg">
        <button
          onClick={() => setCanvasScale(Math.min(canvasScale + 0.15, 2.5))}
          className="p-1.5 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-600 dark:text-stone-300 transition-all"
          title="Zoom In"
        >
          <ZoomIn className="w-4 h-4" />
        </button>
        <span className="text-[11px] font-mono px-2 text-stone-700 dark:text-stone-300 font-bold">
          {Math.round(canvasScale * 100)}%
        </span>
        <button
          onClick={() => setCanvasScale(Math.max(canvasScale - 0.15, 0.4))}
          className="p-1.5 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-600 dark:text-stone-300 transition-all"
          title="Zoom Out"
        >
          <ZoomOut className="w-4 h-4" />
        </button>
        <button
          onClick={() => { setCanvasScale(1.0); setCanvasPan({ x: 0, y: 0 }); }}
          className="p-1.5 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-600 dark:text-stone-300 transition-all"
          title="Reset Viewport"
        >
          <Maximize2 className="w-4 h-4" />
        </button>
      </div>

      {/* Viewport Info Pill */}
      <div className="absolute bottom-28 left-48 z-20 flex items-center space-x-2 px-3 py-1 rounded-xl border border-stone-200 dark:border-stone-800 bg-white/80 dark:bg-stone-900/80 backdrop-blur-md text-[10px] font-mono text-stone-500 shadow-sm">
        <Move className="w-3 h-3 text-stone-400" />
        <span>Pan: {Math.round(canvasPan.x)}, {Math.round(canvasPan.y)}</span>
        <span>•</span>
        <span>Nodes: {canvasNodes.length}</span>
      </div>

      {/* Infinite Canvas Surface with Transform Matrix */}
      <div
        style={{
          transform: `translate(${canvasPan.x}px, ${canvasPan.y}px) scale(${canvasScale})`,
          transformOrigin: "0 0",
        }}
        className="absolute inset-0 w-full h-full pointer-events-none"
      >
        {/* SVG Bezier Connectors Layer */}
        <svg className="absolute inset-0 w-[5000px] h-[5000px] pointer-events-none overflow-visible">
          {renderConnectors()}
        </svg>

        {/* Polymorphic Node Cards Container */}
        <div className="absolute inset-0 pointer-events-auto">
          {canvasNodes.map((node) => {
            const isSelected = selectedNodeId === node.id;
            return (
              <div
                key={node.id}
                style={{
                  transform: `translate(${node.x}px, ${node.y}px)`,
                  position: "absolute",
                }}
                onMouseDown={(e) => startNodeDrag(node.id, e)}
              >
                {node.type === "prompt" && (
                  <PromptCanvasNode
                    id={node.id}
                    data={node.data as any}
                    isSelected={isSelected}
                    onSelect={() => setSelectedNodeId(node.id)}
                  />
                )}
                {node.type === "subagent" && (
                  <AgentCanvasNode
                    id={node.id}
                    data={node.data as any}
                    isSelected={isSelected}
                    onSelect={() => setSelectedNodeId(node.id)}
                  />
                )}
                {node.type === "code" && (
                  <CodeEditorCanvasNode
                    id={node.id}
                    data={node.data as any}
                    isSelected={isSelected}
                    onSelect={() => setSelectedNodeId(node.id)}
                  />
                )}
                {node.type === "diff" && (
                  <DiffCanvasNode
                    id={node.id}
                    data={node.data as any}
                    isSelected={isSelected}
                    onSelect={() => setSelectedNodeId(node.id)}
                  />
                )}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
