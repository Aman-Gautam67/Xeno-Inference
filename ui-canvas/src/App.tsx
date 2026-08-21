import React, { useEffect } from "react";
import { useWorkspaceStore, ViewMode } from "./stores/workspaceStore";
import { HeaderNav } from "./components/layout/HeaderNav";
import { SidebarExplorer } from "./components/layout/SidebarExplorer";
import { TelemetryHUD } from "./components/layout/TelemetryHUD";
import { OmniBar } from "./components/layout/OmniBar";
import { HomepageView } from "./components/home/HomepageView";
import { SpatialCanvas } from "./components/canvas/SpatialCanvas";
import { TorSandboxedBrowserView } from "./components/browser/TorSandboxedBrowserView";
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
    isExportOpen,
    updateMetricsTick,
    themeMode
  } = useWorkspaceStore();

  // Sync Theme with Document Root and Body
  useEffect(() => {
    if (typeof document !== "undefined") {
      if (themeMode === "dark") {
        document.documentElement.classList.add("dark");
        document.documentElement.classList.remove("light");
        document.body.classList.add("dark");
        document.body.classList.remove("light");
      } else {
        document.documentElement.classList.remove("dark");
        document.documentElement.classList.add("light");
        document.body.classList.remove("dark");
        document.body.classList.add("light");
      }
    }
  }, [themeMode]);

  // Dynamic Telemetry Parameter Engine (Updates real parameters every 1 second)
  useEffect(() => {
    const interval = setInterval(() => {
      updateMetricsTick();
    }, 1000);
    return () => clearInterval(interval);
  }, [updateMetricsTick]);

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

      // 1-8 View Switcher
      const views: ViewMode[] = ["home", "canvas", "browser", "dag", "timeline", "terminal", "diff", "swarm"];
      const keyNum = parseInt(e.key, 10);
      if (keyNum >= 1 && keyNum <= 8) {
        e.preventDefault();
        setActiveView(views[keyNum - 1]);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [setActiveView, toggleSidebar, toggleShortcuts, toggleExport, isShortcutsOpen, isExportOpen]);

  return (
    <div className="w-screen h-screen bg-stone-50 dark:bg-stone-950 text-stone-900 dark:text-stone-100 flex flex-col justify-between relative overflow-hidden font-sans select-none transition-colors duration-200">
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
        {/* Collapsible Left Explorer Sidebar (Hidden on Home page for minimalism) */}
        {activeView !== "home" && <SidebarExplorer />}

        {/* Dynamic Viewport Surface */}
        <main className="flex-1 flex overflow-hidden relative">
          {activeView === "home" && <HomepageView />}
          {activeView === "canvas" && <SpatialCanvas />}
          {activeView === "browser" && <TorSandboxedBrowserView />}
          {activeView === "dag" && <LiveExecutionDAG />}
          {activeView === "timeline" && <DeepThinkingTimeline />}
          {activeView === "terminal" && <SandboxedTerminalView />}
          {activeView === "diff" && <ASTDiffStudioView />}
          {activeView === "swarm" && <MultiAgentSwarmView />}
        </main>
      </div>

      {/* Bottom Command & Routing Omni-Bar (Shown on non-home pages) */}
      {activeView !== "home" && <OmniBar />}
    </div>
  );
};

export default App;
