import React from "react";
import { 
  useWorkspaceStore, 
  ViewMode, 
  ProviderModel 
} from "../../stores/workspaceStore";
import { useXenoWebSocket } from "../../hooks/useXenoWebSocket";
import { 
  Home,
  LayoutGrid, 
  Globe,
  GitFork, 
  BrainCircuit, 
  Terminal, 
  FileCode, 
  Users, 
  ShieldAlert, 
  ShieldCheck, 
  Cpu, 
  Menu,
  Sparkles,
  Keyboard,
  FileJson,
  Volume2,
  VolumeX,
  Radio,
  Sun,
  Moon
} from "lucide-react";

export const HeaderNav: React.FC = () => {
  const wsStatus = useXenoWebSocket();
  const { 
    activeView, 
    setActiveView, 
    themeMode,
    toggleTheme,
    selectedModel, 
    setSelectedModel,
    isAirGapped,
    toggleAirGap,
    toggleSidebar,
    toggleShortcuts,
    toggleExport,
    soundEnabled,
    toggleSound
  } = useWorkspaceStore();

  const navTabs: { id: ViewMode; label: string; icon: React.ReactNode }[] = [
    { id: "home", label: "Home", icon: <Home className="w-3.5 h-3.5" /> },
    { id: "canvas", label: "Canvas", icon: <LayoutGrid className="w-3.5 h-3.5" /> },
    { id: "browser", label: "Tor Browser", icon: <Globe className="w-3.5 h-3.5" /> },
    { id: "dag", label: "DAG", icon: <GitFork className="w-3.5 h-3.5" /> },
    { id: "timeline", label: "Thinking", icon: <BrainCircuit className="w-3.5 h-3.5" /> },
    { id: "terminal", label: "Terminal", icon: <Terminal className="w-3.5 h-3.5" /> },
    { id: "diff", label: "Diff Studio", icon: <FileCode className="w-3.5 h-3.5" /> },
    { id: "swarm", label: "Swarm", icon: <Users className="w-3.5 h-3.5" /> },
  ];

  return (
    <header className="h-14 border-b border-stone-200 dark:border-stone-800 bg-white/95 dark:bg-stone-900/95 backdrop-blur-xl px-4 flex items-center justify-between z-40 relative transition-colors duration-200 shadow-sm">
      {/* Brand & Sidebar Toggle */}
      <div className="flex items-center space-x-3">
        <button 
          onClick={toggleSidebar}
          className="p-1.5 rounded-lg border border-stone-200 dark:border-stone-800 hover:border-stone-400 dark:hover:border-stone-600 bg-stone-50 dark:bg-stone-800/80 text-stone-600 dark:text-stone-300 transition-all"
          title="Toggle Explorer Sidebar"
        >
          <Menu className="w-4 h-4" />
        </button>

        <div 
          onClick={() => setActiveView("home")}
          className="flex items-center space-x-2.5 cursor-pointer"
        >
          <span className="font-display font-bold text-sm tracking-wider uppercase text-stone-900 dark:text-stone-50 flex items-center gap-1.5">
            <Sparkles className="w-4 h-4 text-amber-600 dark:text-amber-400" />
            XENO INFERENCE
          </span>
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-stone-100 dark:bg-stone-800 border border-stone-200 dark:border-stone-700 text-stone-600 dark:text-stone-400 font-mono">
            SOVEREIGN
          </span>
        </div>
      </div>

      {/* View Mode Navigation Tabs */}
      <nav className="flex items-center space-x-1 bg-stone-100/80 dark:bg-stone-950/80 p-1 rounded-xl border border-stone-200 dark:border-stone-800 overflow-x-auto max-w-2xl">
        {navTabs.map((tab) => {
          const isActive = activeView === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveView(tab.id)}
              className={`flex items-center space-x-1.5 px-2.5 py-1 rounded-lg text-xs font-mono transition-all ${
                isActive
                  ? "bg-white dark:bg-stone-800 text-stone-900 dark:text-stone-50 border border-stone-200 dark:border-stone-700 shadow-sm font-semibold"
                  : "text-stone-500 hover:text-stone-900 dark:text-stone-400 dark:hover:text-stone-200 hover:bg-white/50 dark:hover:bg-stone-850"
              }`}
            >
              {tab.icon}
              <span className="hidden sm:inline">{tab.label}</span>
            </button>
          );
        })}
      </nav>

      {/* Model Switcher, Air-Gap, Theme Toggle, and Actions */}
      <div className="flex items-center space-x-2">
        {/* Air-Gap Toggle */}
        <button
          onClick={toggleAirGap}
          className={`flex items-center space-x-1.5 px-2.5 py-1 rounded-lg border text-xs font-mono transition-all ${
            isAirGapped 
              ? "bg-emerald-50 dark:bg-emerald-950/40 border-emerald-300 dark:border-emerald-800 text-emerald-700 dark:text-emerald-300"
              : "bg-stone-50 dark:bg-stone-800 border-stone-200 dark:border-stone-700 text-stone-600 dark:text-stone-400 hover:border-stone-400"
          }`}
          title={isAirGapped ? "Air-Gap Active: Zero network outbound" : "Cloud Gateway Active"}
        >
          {isAirGapped ? <ShieldCheck className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" /> : <ShieldAlert className="w-3.5 h-3.5 text-amber-600 dark:text-amber-400" />}
          <span className="hidden xl:inline">{isAirGapped ? "AIR-GAP" : "HYBRID"}</span>
        </button>

        {/* Model Selector */}
        <div className="hidden lg:flex items-center space-x-1.5 bg-stone-50 dark:bg-stone-800 px-2.5 py-1 rounded-lg border border-stone-200 dark:border-stone-700">
          <Cpu className="w-3.5 h-3.5 text-stone-500 dark:text-stone-400" />
          <select
            value={selectedModel}
            onChange={(e) => setSelectedModel(e.target.value as ProviderModel)}
            className="bg-transparent text-xs font-mono text-stone-800 dark:text-stone-200 outline-none cursor-pointer"
          >
            <option value="claude-3-7-sonnet" className="bg-white dark:bg-stone-900">Claude 3.7 Sonnet (Thinking)</option>
            <option value="deepseek-r1" className="bg-white dark:bg-stone-900">DeepSeek R1 (Inline CoT)</option>
            <option value="gpt-4o" className="bg-white dark:bg-stone-900">OpenAI GPT-4o</option>
            <option value="gemini-2-pro" className="bg-white dark:bg-stone-900">Google Gemini 2.0 Pro</option>
            <option value="local-gguf" className="bg-white dark:bg-stone-900">Local Llama 3.3 (GGUF)</option>
            <option value="groq-llama3" className="bg-white dark:bg-stone-900">Groq LPU (500+ tok/s)</option>
          </select>
        </div>

        {/* Theme Toggle (Light / Dark Mode) */}
        <button
          onClick={toggleTheme}
          className="p-1.5 rounded-lg border border-stone-200 dark:border-stone-800 hover:border-stone-400 dark:hover:border-stone-600 bg-stone-50 dark:bg-stone-800 text-stone-700 dark:text-stone-300 transition-all shadow-sm"
          title={`Switch to ${themeMode === "light" ? "Dark" : "Light"} mode`}
        >
          {themeMode === "light" ? <Moon className="w-4 h-4" /> : <Sun className="w-4 h-4 text-amber-400" />}
        </button>

        {/* QOL Utilities: Export, Shortcuts, Sound */}
        <div className="flex items-center space-x-1 pl-1 border-l border-stone-200 dark:border-stone-800">
          <button
            onClick={toggleExport}
            className="p-1.5 rounded-lg border border-stone-200 dark:border-stone-800 hover:border-stone-400 dark:hover:border-stone-600 bg-stone-50 dark:bg-stone-800 text-stone-600 dark:text-stone-300 transition-all"
            title="Export / Import Session State"
          >
            <FileJson className="w-4 h-4" />
          </button>
          <button
            onClick={toggleShortcuts}
            className="p-1.5 rounded-lg border border-stone-200 dark:border-stone-800 hover:border-stone-400 dark:hover:border-stone-600 bg-stone-50 dark:bg-stone-800 text-stone-600 dark:text-stone-300 transition-all"
            title="Keyboard Shortcuts (Shift + ?)"
          >
            <Keyboard className="w-4 h-4" />
          </button>
          <button
            onClick={toggleSound}
            className={`p-1.5 rounded-lg border transition-all ${
              soundEnabled
                ? "border-stone-300 dark:border-stone-700 text-stone-800 dark:text-stone-200 bg-stone-100 dark:bg-stone-800"
                : "border-stone-200 dark:border-stone-800 text-stone-400 hover:bg-stone-50 dark:hover:bg-stone-850"
            }`}
            title={soundEnabled ? "Audio / Haptic Feedback ON" : "Audio Feedback Muted"}
          >
            {soundEnabled ? <Volume2 className="w-4 h-4" /> : <VolumeX className="w-4 h-4" />}
          </button>
        </div>
      </div>
    </header>
  );
};
