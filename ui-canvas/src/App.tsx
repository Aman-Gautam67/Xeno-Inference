import React, { useEffect } from "react";
import { useWorkspaceStore, ViewMode } from "./stores/workspaceStore";
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
import { ShortcutsModal } from "./components/modals/ShortcutsModal";
import { SessionExportModal } from "./components/modals/SessionExportModal";

export const App: React.FC = () => {
  const { 
    activeView, 
    setActiveView, 
    toggleSidebar, 
    toggleShortcuts, 
    toggleExport, 
    isShortcutsOpen, 
    isExportOpen 
  } = useWorkspaceStore();

  // Global Keyboard Shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore when typing in input or textarea
      if (
        e.target instanceof HTMLInputElement || 
        e.target instanceof HTMLTextAreaElement || 
        e.target instanceof HTMLSelectElement
      ) {
        if (e.key === "Escape") {
          (e.target as HTMLElement).blur();
        }
        return;
      }

      if (e.key === "Escape") {
        if (isShortcutsOpen) toggleShortcuts();
        if (isExportOpen) toggleExport();
        return;
      }

      if (e.key === "?" || (e.shiftKey && e.key === "/")) {
        e.preventDefault();
        toggleShortcuts();
        return;
      }

      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "e") {
        e.preventDefault();
        toggleExport();
        return;
      }

      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "b") {
        e.preventDefault();
        toggleSidebar();
        return;
      }

      // 1-6 View Switcher
      const views: ViewMode[] = ["canvas", "dag", "timeline", "terminal", "diff", "swarm"];
      const keyNum = parseInt(e.key, 10);
      if (keyNum >= 1 && keyNum <= 6) {
        e.preventDefault();
        setActiveView(views[keyNum - 1]);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [setActiveView, toggleSidebar, toggleShortcuts, toggleExport, isShortcutsOpen, isExportOpen]);

  return (
    <div className="w-screen h-screen bg-void text-neutral-100 flex flex-col justify-between relative overflow-hidden font-sans select-none">
      {/* Modals */}
      <ShortcutsModal />
      <SessionExportModal />

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
