import React from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { Zap, Coins, HardDrive, Gauge, Clock, Layers, Activity } from "lucide-react";

export const TelemetryHUD: React.FC = () => {
  const { telemetry, soundEnabled } = useWorkspaceStore();

  const totalTokens = 48230;
  const maxContext = 128000;
  const contextPct = ((totalTokens / maxContext) * 100).toFixed(1);

  return (
    <div className="fixed top-18 right-6 z-30 p-3 rounded-2xl border border-border-700 bg-surface-900/85 backdrop-blur-2xl shadow-2xl flex items-center space-x-5 text-xs font-mono">
      {/* Speed Meter */}
      <div className="flex items-center space-x-2.5">
        <div className="p-1.5 rounded-lg bg-cyan-500/10 border border-cyan-500/30 text-cyan-400">
          <Zap className="w-4 h-4" />
        </div>
        <div>
          <div className="text-neutral-500 uppercase text-[9px] tracking-wider">Speed</div>
          <div className="text-cyan-400 font-bold text-sm tracking-tight">
            {telemetry.velocity.toFixed(1)} <span className="text-[10px] font-normal text-neutral-400">tok/s</span>
          </div>
        </div>
      </div>

      <div className="w-[1px] h-7 bg-border-700" />

      {/* Context Window Capacity Tracker */}
      <div className="flex items-center space-x-2.5">
        <div className="p-1.5 rounded-lg bg-indigo-500/10 border border-indigo-500/30 text-indigo-400">
          <Layers className="w-4 h-4" />
        </div>
        <div>
          <div className="text-neutral-500 uppercase text-[9px] tracking-wider">Context Window</div>
          <div className="text-indigo-300 font-bold text-sm tracking-tight">
            {contextPct}% <span className="text-[10px] font-normal text-neutral-400">({(totalTokens / 1000).toFixed(1)}k / {maxContext / 1000}k)</span>
          </div>
        </div>
      </div>

      <div className="w-[1px] h-7 bg-border-700" />

      {/* VRAM Gauge */}
      <div className="flex items-center space-x-2.5">
        <div className="p-1.5 rounded-lg bg-purple-500/10 border border-purple-500/30 text-purple-400">
          <HardDrive className="w-4 h-4" />
        </div>
        <div>
          <div className="text-neutral-500 uppercase text-[9px] tracking-wider">VRAM / CUDA</div>
          <div className="text-purple-400 font-bold text-sm tracking-tight">
            {telemetry.vramUsedGb.toFixed(1)} <span className="text-[10px] font-normal text-neutral-400">/ {telemetry.vramTotalGb.toFixed(1)} GB</span>
          </div>
        </div>
      </div>

      <div className="w-[1px] h-7 bg-border-700" />

      {/* GPU Load */}
      <div className="flex items-center space-x-2.5">
        <div className="p-1.5 rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-400">
          <Gauge className="w-4 h-4" />
        </div>
        <div>
          <div className="text-neutral-500 uppercase text-[9px] tracking-wider">GPU Core</div>
          <div className="text-amber-400 font-bold text-sm tracking-tight">
            {telemetry.gpuLoadPct.toFixed(1)}%
          </div>
        </div>
      </div>

      <div className="w-[1px] h-7 bg-border-700" />

      {/* Session Cost */}
      <div className="flex items-center space-x-2.5">
        <div className="p-1.5 rounded-lg bg-emerald-500/10 border border-emerald-500/30 text-emerald-400">
          <Coins className="w-4 h-4" />
        </div>
        <div>
          <div className="text-neutral-500 uppercase text-[9px] tracking-wider">Est. Cost</div>
          <div className="text-emerald-400 font-bold text-sm tracking-tight">
            ${telemetry.costUsd.toFixed(4)}
          </div>
        </div>
      </div>

      {/* Audio Activity Wave Indicator */}
      {soundEnabled && (
        <>
          <div className="w-[1px] h-7 bg-border-700" />
          <div className="flex items-center space-x-1 text-cyan-400 animate-pulse" title="Audio telemetry synthesizer active">
            <Activity className="w-3.5 h-3.5" />
          </div>
        </>
      )}
    </div>
  );
};
