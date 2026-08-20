import React, { useRef, useState } from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { PromptCanvasNode } from "./nodes/PromptCanvasNode";
import { AgentCanvasNode } from "./nodes/AgentCanvasNode";
import { CodeEditorCanvasNode } from "./nodes/CodeEditorCanvasNode";
import { DiffCanvasNode } from "./nodes/DiffCanvasNode";
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
      setCanvasPan({ x: e.clientX - panStart.x, y: e.clientY - panStart.y });
    } else if (draggingNodeId) {
      const node = canvasNodes.find((n) => n.id === draggingNodeId);
      if (node) {
        const newX = (e.clientX - canvasPan.x) / canvasScale - dragOffset.x;
        const newY = (e.clientY - canvasPan.y) / canvasScale - dragOffset.y;
        updateCanvasNodePosition(draggingNodeId, newX, newY);
      }
    }
  };

  const handleMouseUp = () => {
    setIsPanning(false);
    setDraggingNodeId(null);
  };

  const handleNodeDragStart = (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    setSelectedNodeId(id);
    setDraggingNodeId(id);
    const node = canvasNodes.find((n) => n.id === id);
    if (node) {
      const mouseX = (e.clientX - canvasPan.x) / canvasScale;
      const mouseY = (e.clientY - canvasPan.y) / canvasScale;
      setDragOffset({ x: mouseX - node.x, y: mouseY - node.y });
    }
  };

  // Helper to find node center for bezier edges
  const getNodeCenter = (id: string, width = 384, height = 180) => {
    const node = canvasNodes.find((n) => n.id === id);
    if (!node) return { x: 0, y: 0, right: 0, left: 0 };
    return {
      x: node.x + width / 2,
      y: node.y + height / 2,
      right: node.x + width,
      left: node.x,
    };
  };

  const promptCenter = getNodeCenter("node-prompt");
  const coderCenter = getNodeCenter("node-coder");
  const codeCenter = getNodeCenter("node-code-block", 480);
  const diffCenter = getNodeCenter("node-diff", 480);

  // SVG Cubic Bezier Curve
  const renderBezier = (x1: number, y1: number, x2: number, y2: number, color = "#00f0ff") => {
    const dx = Math.abs(x2 - x1) * 0.5;
    const path = `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;
    return (
      <g>
        <path
          d={path}
          fill="none"
          stroke={color}
          strokeWidth="2.5"
          strokeDasharray="6 4"
          className="opacity-70 animate-pulse"
        />
        <circle cx={x1} cy={y1} r="4" fill={color} />
        <circle cx={x2} cy={y2} r="4" fill={color} />
      </g>
    );
  };

  return (
    <div
      ref={containerRef}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      className="flex-1 relative w-full h-[calc(100vh-3.5rem)] overflow-hidden canvas-grid-pattern cursor-grab active:cursor-grabbing bg-void"
    >
      {/* Zoom Controls Overlay */}
      <div className="absolute bottom-28 right-6 z-20 flex items-center space-x-1 p-1.5 rounded-xl border border-border-700 bg-surface-900/90 backdrop-blur-xl shadow-2xl">
        <button
          onClick={() => setCanvasScale(Math.min(canvasScale + 0.15, 2.5))}
          className="p-1.5 rounded-lg hover:bg-surface-800 text-neutral-400 hover:text-cyan-400 transition-all"
          title="Zoom In"
        >
          <ZoomIn className="w-4 h-4" />
        </button>
        <span className="text-[11px] font-mono px-2 text-neutral-300">
          {Math.round(canvasScale * 100)}%
        </span>
        <button
          onClick={() => setCanvasScale(Math.max(canvasScale - 0.15, 0.4))}
          className="p-1.5 rounded-lg hover:bg-surface-800 text-neutral-400 hover:text-cyan-400 transition-all"
          title="Zoom Out"
        >
          <ZoomOut className="w-4 h-4" />
        </button>
        <button
          onClick={() => { setCanvasScale(1.0); setCanvasPan({ x: 0, y: 0 }); }}
          className="p-1.5 rounded-lg hover:bg-surface-800 text-neutral-400 hover:text-cyan-400 transition-all"
          title="Reset View"
        >
          <Maximize2 className="w-4 h-4" />
        </button>
      </div>

      {/* Transform Container */}
      <div
        className="w-full h-full absolute origin-top-left transition-transform duration-75"
        style={{
          transform: `translate(${canvasPan.x}px, ${canvasPan.y}px) scale(${canvasScale})`,
        }}
      >
        {/* SVG Connection Layer */}
        <svg className="w-[4000px] h-[4000px] absolute top-0 left-0 pointer-events-none z-0">
          {renderBezier(promptCenter.right, promptCenter.y, coderCenter.left, coderCenter.y, "#00f0ff")}
          {renderBezier(coderCenter.right, coderCenter.y - 20, codeCenter.left, codeCenter.y, "#bd00ff")}
          {renderBezier(coderCenter.right, coderCenter.y + 20, diffCenter.left, diffCenter.y, "#ffb800")}
        </svg>

        {/* Polymorphic Nodes */}
        {canvasNodes.map((node) => {
          const isSelected = selectedNodeId === node.id;
          return (
            <div
              key={node.id}
              style={{
                transform: `translate(${node.x}px, ${node.y}px)`,
                position: "absolute",
                zIndex: isSelected ? 15 : 10,
              }}
              onMouseDown={(e) => handleNodeDragStart(e, node.id)}
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
  );
};
