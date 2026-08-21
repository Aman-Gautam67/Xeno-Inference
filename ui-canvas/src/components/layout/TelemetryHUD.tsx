import React from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { Zap, Coins, HardDrive, Gauge, Layers, Activity } from "lucide-react";

export const TelemetryHUD: React.FC = () => {
  const { systemMetrics, soundEnabled } = useWorkspaceStore();

  const maxContext = 128000;
  const contextPct = ((systemMetrics.liveTokenCount / maxContext) * 100).toFixed(1);

  return (
    <div className="fixed top-18 right-6 z-30 p-3 rounded-2xl border border-stone-200 dark:border-stone-800 bg-white/90 dark:bg-stone-900/90 backdrop-blur-xl shadow-lg flex items-center space-x-5 text-xs font-mono transition-colors duration-200">
      {/* Speed Meter */}
      <div className="flex items-center space-x-2.5">
        <div className="p-1.5 rounded-lg bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300">
          <Zap className="w-4 h-4 text-amber-600 dark:text-amber-400" />
        </div>
        <div>
          <div className="text-stone-400 dark:text-stone-500 uppercase text-[9px] tracking-wider font-bold">Speed</div>
          <div className="text-stone-900 dark:text-stone-100 font-bold text-sm tracking-tight">
            {systemMetrics.liveTokPerSec.toFixed(1)} <span className="text-[10px] font-normal text-stone-500">tok/s</span>
          </div>
        </div>
      </div>

      <div className="w-[1px] h-7 bg-stone-200 dark:bg-stone-800" />

      {/* Context Window Capacity Tracker */}
      <div className="flex items-center space-x-2.5">
        <div className="p-1.5 rounded-lg bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300">
          <Layers className="w-4 h-4 text-stone-600 dark:text-stone-400" />
        </div>
        <div>
          <div className="text-stone-400 dark:text-stone-500 uppercase text-[9px] tracking-wider font-bold">Context</div>
          <div className="text-stone-900 dark:text-stone-100 font-bold text-sm tracking-tight">
            {contextPct}% <span className="text-[10px] font-normal text-stone-500">({(systemMetrics.liveTokenCount / 1000).toFixed(1)}k / 128k)</span>
          </div>
        </div>
      </div>

      <div className="w-[1px] h-7 bg-stone-200 dark:bg-stone-800" />

      {/* RAM Heap Gauge */}
      <div className="flex items-center space-x-2.5">
        <div className="p-1.5 rounded-lg bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300">
          <HardDrive className="w-4 h-4 text-purple-600 dark:text-purple-400" />
        </div>
        <div>
          <div className="text-stone-400 dark:text-stone-500 uppercase text-[9px] tracking-wider font-bold">RAM Heap</div>
          <div className="text-stone-900 dark:text-stone-100 font-bold text-sm tracking-tight">
            {systemMetrics.ramHeapMb} <span className="text-[10px] font-normal text-stone-500">MB</span>
          </div>
        </div>
      </div>

      <div className="w-[1px] h-7 bg-stone-200 dark:bg-stone-800" />

      {/* GPU Load */}
      <div className="flex items-center space-x-2.5">
        <div className="p-1.5 rounded-lg bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300">
          <Gauge className="w-4 h-4 text-stone-600 dark:text-stone-400" />
        </div>
        <div>
          <div className="text-stone-400 dark:text-stone-500 uppercase text-[9px] tracking-wider font-bold">GPU Load</div>
          <div className="text-stone-900 dark:text-stone-100 font-bold text-sm tracking-tight">
            {systemMetrics.gpuLoadPct.toFixed(1)}%
          </div>
        </div>
      </div>

      <div className="w-[1px] h-7 bg-stone-200 dark:bg-stone-800" />

      {/* Session Cost */}
      <div className="flex items-center space-x-2.5">
        <div className="p-1.5 rounded-lg bg-emerald-50 dark:bg-emerald-950/60 text-emerald-700 dark:text-emerald-300">
          <Coins className="w-4 h-4" />
        </div>
        <div>
          <div className="text-stone-400 dark:text-stone-500 uppercase text-[9px] tracking-wider font-bold">Cost</div>
          <div className="text-emerald-700 dark:text-emerald-400 font-bold text-sm tracking-tight">
            ${systemMetrics.costUsd.toFixed(4)}
          </div>
        </div>
      </div>

      {/* Audio Activity Wave Indicator */}
      {soundEnabled && (
        <>
          <div className="w-[1px] h-7 bg-stone-200 dark:bg-stone-800" />
          <div className="flex items-center space-x-1 text-stone-600 dark:text-stone-400 animate-pulse" title="Audio telemetry active">
            <Activity className="w-3.5 h-3.5" />
          </div>
        </>
      )}
    </div>
  );
};
