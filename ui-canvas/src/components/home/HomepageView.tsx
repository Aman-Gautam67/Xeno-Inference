import React, { useState } from "react";
import { useWorkspaceStore, ViewMode } from "../../stores/workspaceStore";
import { 
  Sparkles, 
  ArrowRight, 
  LayoutGrid, 
  Globe, 
  GitFork, 
  BrainCircuit, 
  Terminal, 
  FileCode, 
  Users, 
  Cpu, 
  HardDrive, 
  Clock, 
  Activity, 
  ShieldCheck,
  Zap,
  Layers,
  Compass
} from "lucide-react";

export const HomepageView: React.FC = () => {
  const { 
    setActiveView, 
    systemMetrics, 
    handleSmartPrompt,
    selectedModel,
    isAirGapped
  } = useWorkspaceStore();

  const [promptInput, setPromptInput] = useState("");
  const [routedMessage, setRoutedMessage] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!promptInput.trim()) return;
    const res = handleSmartPrompt(promptInput);
    setRoutedMessage(res.message);
  };

  const actionCards: { id: ViewMode; title: string; subtitle: string; icon: React.ReactNode; badge: string; accent: string }[] = [
    {
      id: "canvas",
      title: "Spatial Canvas",
      subtitle: "2D infinite whiteboard with polymorphic nodes and live bezier data flows.",
      icon: <LayoutGrid className="w-5 h-5 text-stone-700 dark:text-stone-300" />,
      badge: "Interactive 2D",
      accent: "hover:border-stone-400 dark:hover:border-stone-600",
    },
    {
      id: "browser",
      title: "Tor Browser Sandbox",
      subtitle: "Anonymized 3-hop onion router with script isolation and security shields.",
      icon: <Globe className="w-5 h-5 text-emerald-700 dark:text-emerald-400" />,
      badge: "Tor SOCKS5",
      accent: "hover:border-emerald-500/50",
    },
    {
      id: "dag",
      title: "Live Execution DAG",
      subtitle: "Petgraph-driven task dependency visualizer with real-time status transitions.",
      icon: <GitFork className="w-5 h-5 text-stone-700 dark:text-stone-300" />,
      badge: "Petgraph Core",
      accent: "hover:border-stone-400 dark:hover:border-stone-600",
    },
    {
      id: "timeline",
      title: "Deep Thinking Timeline",
      subtitle: "PAORV cognitive stream with speculative reasoning branch evaluations.",
      icon: <BrainCircuit className="w-5 h-5 text-amber-700 dark:text-amber-400" />,
      badge: "Speculative CoT",
      accent: "hover:border-amber-500/50",
    },
    {
      id: "terminal",
      title: "Virtual ConPTY",
      subtitle: "Isolated terminal emulator with Job Object security tier boundaries.",
      icon: <Terminal className="w-5 h-5 text-stone-700 dark:text-stone-300" />,
      badge: "Tier 1-3 Sandbox",
      accent: "hover:border-stone-400 dark:hover:border-stone-600",
    },
    {
      id: "diff",
      title: "AST Diff Studio",
      subtitle: "Character-exact syntax-validated code diff viewer with staging controls.",
      icon: <FileCode className="w-5 h-5 text-stone-700 dark:text-stone-300" />,
      badge: "Syn AST Validated",
      accent: "hover:border-stone-400 dark:hover:border-stone-600",
    },
    {
      id: "swarm",
      title: "Swarm Arena",
      subtitle: "Autonomous 5-role agent council with 3-way consensus verification.",
      icon: <Users className="w-5 h-5 text-stone-700 dark:text-stone-300" />,
      badge: "5-Role Council",
      accent: "hover:border-stone-400 dark:hover:border-stone-600",
    },
  ];

  const quickPrompts = [
    "Review AST diff in ast_validator.rs",
    "Run workspace cargo test in sandboxed terminal",
    "Browse duckduckgo onion research page",
    "Inspect live DAG task dependencies",
    "Launch autonomous 5-agent swarm council",
  ];

  const formatUptime = (secs: number) => {
    const mins = Math.floor(secs / 60);
    const hours = Math.floor(mins / 60);
    return `${hours}h ${mins % 60}m ${secs % 60}s`;
  };

  return (
    <div className="flex-1 h-[calc(100vh-3.5rem)] overflow-y-auto bg-stone-50 dark:bg-stone-950 text-stone-900 dark:text-stone-100 p-8 md:p-12 space-y-12 transition-colors duration-200">
      {/* Hero Header — Classical Romanian / Roman Editorial Aesthetic */}
      <div className="max-w-4xl mx-auto text-center space-y-4 pt-4">
        <div className="inline-flex items-center space-x-2 px-3 py-1 rounded-full border border-stone-300 dark:border-stone-800 bg-white dark:bg-stone-900 shadow-sm text-xs font-mono text-stone-600 dark:text-stone-400">
          <Sparkles className="w-3.5 h-3.5 text-amber-600 dark:text-amber-400" />
          <span>SOVEREIGN AGENTIC WORKSTATION</span>
          <span>•</span>
          <span className="text-emerald-600 dark:text-emerald-400 font-semibold">AIR-GAP VERIFIED</span>
        </div>

        <h1 className="text-4xl md:text-5xl font-display font-bold text-stone-900 dark:text-stone-50 tracking-tight leading-tight">
          XENO INFERENCE
        </h1>

        <p className="text-base md:text-lg font-editorial italic text-stone-600 dark:text-stone-400 max-w-2xl mx-auto leading-relaxed">
          High-performance, GPU-accelerated spatial AI canvas with autonomous multi-agent councils, AST verification, and Tor-routed sovereign execution.
        </p>
      </div>

      {/* Smart Omniprompt Bar */}
      <div className="max-w-3xl mx-auto space-y-3">
        <form
          onSubmit={handleSubmit}
          className="relative flex items-center bg-white dark:bg-stone-900 rounded-2xl border border-stone-300 dark:border-stone-800 shadow-md card-elevation p-2 transition-all focus-within:border-stone-500 dark:focus-within:border-stone-400 focus-within:ring-2 focus-within:ring-stone-200 dark:focus-within:ring-stone-800"
        >
          <div className="pl-3 pr-2 text-stone-400">
            <Compass className="w-5 h-5" />
          </div>
          <input
            type="text"
            value={promptInput}
            onChange={(e) => setPromptInput(e.target.value)}
            placeholder="Type any goal, tool call, or instruction (e.g. 'browse onion site', 'review diff', 'run tests')..."
            className="flex-1 bg-transparent py-2.5 px-2 text-sm text-stone-900 dark:text-stone-100 placeholder:text-stone-400 outline-none font-sans"
          />
          <button
            type="submit"
            className="flex items-center space-x-1.5 px-4 py-2 bg-stone-900 hover:bg-stone-800 dark:bg-stone-100 dark:hover:bg-white text-white dark:text-stone-950 font-medium rounded-xl text-xs transition-all"
          >
            <span>Direct</span>
            <ArrowRight className="w-3.5 h-3.5" />
          </button>
        </form>

        {routedMessage && (
          <div className="text-xs font-mono text-center text-emerald-600 dark:text-emerald-400 animate-fade-in">
            › {routedMessage}
          </div>
        )}

        {/* Quick Suggestion Pills */}
        <div className="flex flex-wrap items-center justify-center gap-2 pt-1">
          {quickPrompts.map((qp, i) => (
            <button
              key={i}
              onClick={() => {
                setPromptInput(qp);
                handleSmartPrompt(qp);
              }}
              className="text-[11px] font-mono px-3 py-1 rounded-full bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 text-stone-600 dark:text-stone-400 hover:border-stone-400 dark:hover:border-stone-600 hover:text-stone-900 dark:hover:text-stone-200 transition-all shadow-sm"
            >
              {qp}
            </button>
          ))}
        </div>
      </div>

      {/* Action Mode Grid */}
      <div className="max-w-5xl mx-auto space-y-4">
        <div className="flex items-center justify-between pb-2 border-b border-stone-200 dark:border-stone-800">
          <span className="font-display font-bold text-sm tracking-wider uppercase text-stone-800 dark:text-stone-200">
            Workstation Surfaces
          </span>
          <span className="text-xs font-mono text-stone-500">7 Active Viewports</span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {actionCards.map((card) => (
            <div
              key={card.id}
              onClick={() => setActiveView(card.id)}
              className={`p-5 rounded-2xl bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 shadow-sm card-elevation cursor-pointer flex flex-col justify-between space-y-4 transition-all ${card.accent}`}
            >
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <div className="p-2 rounded-xl bg-stone-100 dark:bg-stone-800 border border-stone-200 dark:border-stone-700">
                    {card.icon}
                  </div>
                  <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-stone-100 dark:bg-stone-800 text-stone-600 dark:text-stone-400 border border-stone-200 dark:border-stone-700">
                    {card.badge}
                  </span>
                </div>
                <h3 className="font-display font-bold text-base text-stone-900 dark:text-stone-100">
                  {card.title}
                </h3>
                <p className="text-xs text-stone-500 dark:text-stone-400 leading-relaxed font-sans">
                  {card.subtitle}
                </p>
              </div>

              <div className="flex items-center text-xs font-mono font-semibold text-stone-700 dark:text-stone-300 pt-2 border-t border-stone-100 dark:border-stone-800">
                <span>Enter Workspace</span>
                <ArrowRight className="w-3.5 h-3.5 ml-1 transition-transform group-hover:translate-x-1" />
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Live Server-Rendered Dynamic Parameter Telemetry Matrix */}
      <div className="max-w-5xl mx-auto space-y-3 pt-4">
        <div className="flex items-center justify-between pb-2 border-b border-stone-200 dark:border-stone-800">
          <span className="font-display font-bold text-sm tracking-wider uppercase text-stone-800 dark:text-stone-200">
            Real Hardware & Server Telemetry
          </span>
          <span className="text-xs font-mono text-emerald-600 dark:text-emerald-400 flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
            Live Reactive State
          </span>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-3 font-mono text-xs">
          {/* CPU Cores */}
          <div className="p-3.5 rounded-xl bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 shadow-sm space-y-1">
            <div className="flex items-center space-x-1.5 text-stone-500 text-[10px]">
              <Cpu className="w-3.5 h-3.5" />
              <span>CPU Cores</span>
            </div>
            <div className="font-bold text-stone-900 dark:text-stone-100 text-sm">
              {systemMetrics.cpuCores} Cores
            </div>
          </div>

          {/* Heap Memory */}
          <div className="p-3.5 rounded-xl bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 shadow-sm space-y-1">
            <div className="flex items-center space-x-1.5 text-stone-500 text-[10px]">
              <HardDrive className="w-3.5 h-3.5" />
              <span>JS Heap RAM</span>
            </div>
            <div className="font-bold text-stone-900 dark:text-stone-100 text-sm">
              {systemMetrics.ramHeapMb} MB
            </div>
          </div>

          {/* Velocity */}
          <div className="p-3.5 rounded-xl bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 shadow-sm space-y-1">
            <div className="flex items-center space-x-1.5 text-stone-500 text-[10px]">
              <Zap className="w-3.5 h-3.5" />
              <span>Token Speed</span>
            </div>
            <div className="font-bold text-stone-900 dark:text-stone-100 text-sm">
              {systemMetrics.liveTokPerSec} tok/s
            </div>
          </div>

          {/* Total Tokens */}
          <div className="p-3.5 rounded-xl bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 shadow-sm space-y-1">
            <div className="flex items-center space-x-1.5 text-stone-500 text-[10px]">
              <Layers className="w-3.5 h-3.5" />
              <span>Session Tokens</span>
            </div>
            <div className="font-bold text-stone-900 dark:text-stone-100 text-sm">
              {systemMetrics.liveTokenCount.toLocaleString()}
            </div>
          </div>

          {/* Screen / DPI */}
          <div className="p-3.5 rounded-xl bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 shadow-sm space-y-1">
            <div className="flex items-center space-x-1.5 text-stone-500 text-[10px]">
              <Activity className="w-3.5 h-3.5" />
              <span>DPI / Viewport</span>
            </div>
            <div className="font-bold text-stone-900 dark:text-stone-100 text-sm truncate">
              {systemMetrics.devicePixelRatio}x ({systemMetrics.screenResolution})
            </div>
          </div>

          {/* Session Uptime */}
          <div className="p-3.5 rounded-xl bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 shadow-sm space-y-1">
            <div className="flex items-center space-x-1.5 text-stone-500 text-[10px]">
              <Clock className="w-3.5 h-3.5" />
              <span>Uptime</span>
            </div>
            <div className="font-bold text-stone-900 dark:text-stone-100 text-sm truncate">
              {formatUptime(systemMetrics.activeSessionUptimeSecs)}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
