import React from "react";
import { useWorkspaceStore } from "./stores/workspaceStore";
import { HeaderNav } from "./components/layout/HeaderNav";
import { SidebarExplorer } from "./components/layout/SidebarExplorer";
import { TelemetryHUD } from "./components/layout/TelemetryHUD";
import { OmniBar } from "./components/layout/OmniBar";
import { SpatialCanvas } from "./components/canvas/SpatialCanvas";
import { LiveExecutionDAG } from "./components/dag/LiveExecutionDAG";
import { DeepThinkingTimeline } from "./components/timeline/DeepThinkingTimeline";
import { SandboxedTerminalView } from "./components/terminal/SandboxedTerminalView";
import { ASTDiffStudioView } from "./components/diff/ASTDiffStudioView";
import { MultiAgentSwarmView } from "./components/swarm/MultiAgentSwarmView";

export const App: React.FC = () => {
  const { activeView } = useWorkspaceStore();

  return (
    <div className="w-screen h-screen bg-void text-neutral-100 flex flex-col justify-between relative overflow-hidden font-sans select-none">
      {/* Top Navigation */}
      <HeaderNav />

      {/* Real-Time Floating Telemetry HUD (Shown on Canvas, Swarm, and DAG modes) */}
      {(activeView === "canvas" || activeView === "swarm" || activeView === "dag") && (
        <TelemetryHUD />
      )}

      {/* Main App Workspace */}
      <div className="flex-1 flex overflow-hidden relative">
        {/* Collapsible Left Explorer Sidebar */}
        <SidebarExplorer />

        {/* Dynamic Viewport Surface */}
        <main className="flex-1 flex overflow-hidden relative">
          {activeView === "canvas" && <SpatialCanvas />}
          {activeView === "dag" && <LiveExecutionDAG />}
          {activeView === "timeline" && <DeepThinkingTimeline />}
          {activeView === "terminal" && <SandboxedTerminalView />}
          {activeView === "diff" && <ASTDiffStudioView />}
          {activeView === "swarm" && <MultiAgentSwarmView />}
        </main>
      </div>

      {/* Bottom Command & Routing Omni-Bar */}
      <OmniBar />
    </div>
  );
};

export default App;
