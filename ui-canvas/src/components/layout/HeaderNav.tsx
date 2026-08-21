import { 
  useWorkspaceStore, 
  ViewMode, 
  ProviderModel 
} from "../../stores/workspaceStore";
import { useXenoWebSocket } from "../../hooks/useXenoWebSocket";
import { 
  LayoutGrid, 
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
  Radio
} from "lucide-react";

export const HeaderNav: React.FC = () => {
  const wsStatus = useXenoWebSocket();
  const { 
    activeView, 
    setActiveView, 
    selectedModel, 
    setSelectedModel,
    isAirGapped,
    toggleAirGap,
    isSidebarOpen,
    toggleSidebar,
    toggleShortcuts,
    toggleExport,
    soundEnabled,
    toggleSound
  } = useWorkspaceStore();

  const navTabs: { id: ViewMode; label: string; icon: React.ReactNode }[] = [
    { id: "canvas", label: "Spatial Canvas", icon: <LayoutGrid className="w-4 h-4" /> },
    { id: "dag", label: "Live DAG", icon: <GitFork className="w-4 h-4" /> },
    { id: "timeline", label: "Deep Thinking", icon: <BrainCircuit className="w-4 h-4" /> },
    { id: "terminal", label: "Terminal PTY", icon: <Terminal className="w-4 h-4" /> },
    { id: "diff", label: "AST Diff Studio", icon: <FileCode className="w-4 h-4" /> },
    { id: "swarm", label: "Swarm Arena", icon: <Users className="w-4 h-4" /> },
  ];

  return (
    <header className="h-14 border-b border-border-700 bg-surface-900/90 backdrop-blur-xl px-4 flex items-center justify-between z-40 relative">
      {/* Brand & Sidebar Toggle */}
      <div className="flex items-center space-x-3">
        <button 
          onClick={toggleSidebar}
          className="p-1.5 rounded-lg border border-border-700 hover:border-cyan-500/50 hover:bg-surface-800 text-neutral-400 hover:text-cyan-400 transition-all"
          title="Toggle Explorer Sidebar"
        >
          <Menu className="w-4 h-4" />
        </button>

        <div className="flex items-center space-x-2.5">
          <div className="w-2.5 h-2.5 rounded-full bg-cyan-400 animate-pulse glow-cyan" />
          <span className="font-bold text-xs tracking-widest uppercase font-mono text-cyan-400 flex items-center gap-1.5">
            <Sparkles className="w-3.5 h-3.5 text-cyan-400" />
            XENO INFERENCE
          </span>
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-surface-800 border border-border-700 text-neutral-400 font-mono">
            v3.0.0-SOVEREIGN
          </span>
          <div 
            className={`flex items-center space-x-1 text-[10px] px-2 py-0.5 rounded font-mono border ${
              wsStatus.connected 
                ? "bg-emerald-950/60 text-emerald-400 border-emerald-500/40 glow-emerald" 
                : "bg-surface-800 text-neutral-400 border-border-700"
            }`}
            title={wsStatus.connected ? `Connected to ${wsStatus.url} (${wsStatus.latencyMs}ms)` : `Local state mode (${wsStatus.url})`}
          >
            <Radio className={`w-3 h-3 ${wsStatus.connected ? "text-emerald-400 animate-pulse" : "text-neutral-500"}`} />
            <span>{wsStatus.connected ? `WS: ${wsStatus.latencyMs}ms` : "STANDALONE"}</span>
          </div>
        </div>
      </div>

      {/* View Mode Navigation Tabs */}
      <nav className="flex items-center space-x-1 bg-surface-950/80 p-1 rounded-xl border border-border-700">
        {navTabs.map((tab) => {
          const isActive = activeView === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveView(tab.id)}
              className={`flex items-center space-x-2 px-3 py-1.5 rounded-lg text-xs font-mono transition-all ${
                isActive
                  ? "bg-cyan-500/15 text-cyan-300 border border-cyan-500/40 shadow-sm glow-cyan"
                  : "text-neutral-400 hover:text-neutral-200 hover:bg-surface-800/60"
              }`}
            >
              {tab.icon}
              <span>{tab.label}</span>
            </button>
          );
        })}
      </nav>

      {/* Model Switcher & Air-Gap Control */}
      <div className="flex items-center space-x-3">
        {/* Air-Gap Toggle */}
        <button
          onClick={toggleAirGap}
          className={`flex items-center space-x-1.5 px-2.5 py-1.5 rounded-lg border text-xs font-mono transition-all ${
            isAirGapped 
              ? "bg-emerald-500/15 border-emerald-500/40 text-emerald-400 glow-emerald"
              : "bg-surface-800 border-border-700 text-neutral-400 hover:border-amber-500/40 hover:text-amber-300"
          }`}
          title={isAirGapped ? "Air-Gap Active: Zero network outbound" : "Cloud Gateway Active"}
        >
          {isAirGapped ? <ShieldCheck className="w-3.5 h-3.5" /> : <ShieldAlert className="w-3.5 h-3.5 text-amber-400" />}
          <span>{isAirGapped ? "AIR-GAP LOCKED" : "CLOUD + LOCAL"}</span>
        </button>

        {/* Model Selector */}
        <div className="flex items-center space-x-1.5 bg-surface-800 px-2.5 py-1 rounded-lg border border-border-700">
          <Cpu className="w-3.5 h-3.5 text-purple-400" />
          <select
            value={selectedModel}
            onChange={(e) => setSelectedModel(e.target.value as ProviderModel)}
            className="bg-transparent text-xs font-mono text-neutral-200 outline-none cursor-pointer"
          >
            <option value="claude-3-7-sonnet" className="bg-surface-900 text-neutral-200">Claude 3.7 Sonnet (Thinking)</option>
            <option value="deepseek-r1" className="bg-surface-900 text-neutral-200">DeepSeek R1 (Inline CoT)</option>
            <option value="gpt-4o" className="bg-surface-900 text-neutral-200">OpenAI GPT-4o</option>
            <option value="gemini-2-pro" className="bg-surface-900 text-neutral-200">Google Gemini 2.0 Pro</option>
            <option value="local-gguf" className="bg-surface-900 text-neutral-200">Local Llama 3.3 (GGUF / CUDA)</option>
            <option value="groq-llama3" className="bg-surface-900 text-neutral-200">Groq LPU (500+ tok/s)</option>
          </select>
        </div>

        {/* QOL Utilities: Export, Shortcuts, Audio Feedback */}
        <div className="flex items-center space-x-1 pl-1 border-l border-border-700">
          <button
            onClick={toggleExport}
            className="p-1.5 rounded-lg border border-border-700 hover:border-cyan-500/50 hover:bg-surface-800 text-neutral-400 hover:text-cyan-400 transition-all"
            title="Export / Import Session State"
          >
            <FileJson className="w-4 h-4" />
          </button>
          <button
            onClick={toggleShortcuts}
            className="p-1.5 rounded-lg border border-border-700 hover:border-cyan-500/50 hover:bg-surface-800 text-neutral-400 hover:text-cyan-400 transition-all"
            title="Keyboard Shortcuts (Shift + ?)"
          >
            <Keyboard className="w-4 h-4" />
          </button>
          <button
            onClick={toggleSound}
            className={`p-1.5 rounded-lg border transition-all ${
              soundEnabled
                ? "border-cyan-500/40 text-cyan-400 bg-cyan-500/10"
                : "border-border-700 text-neutral-500 hover:bg-surface-800"
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
