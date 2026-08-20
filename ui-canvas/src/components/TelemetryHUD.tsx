import React from "react";

export interface TelemetryHUDProps {
  velocity: number;
  costUsd: number;
  vramUsedGb: number;
  vramTotalGb: number;
  ttftMs: number;
}

export const TelemetryHUD: React.FC<TelemetryHUDProps> = ({
  velocity,
  costUsd,
  vramUsedGb,
  vramTotalGb,
  ttftMs,
}) => {
  return (
    <div className="fixed top-4 right-4 p-4 rounded-xl border border-neutral-800 bg-neutral-900/80 backdrop-blur-xl shadow-2xl flex items-center space-x-6 text-xs font-mono z-50">
      <div>
        <div className="text-neutral-500 uppercase text-[10px]">Speed</div>
        <div className="text-cyan-400 font-bold">{velocity.toFixed(1)} tok/s</div>
      </div>
      <div>
        <div className="text-neutral-500 uppercase text-[10px]">Cost</div>
        <div className="text-emerald-400 font-bold">${costUsd.toFixed(4)}</div>
      </div>
      <div>
        <div className="text-neutral-500 uppercase text-[10px]">VRAM</div>
        <div className="text-purple-400 font-bold">
          {vramUsedGb.toFixed(1)} / {vramTotalGb.toFixed(1)} GB
        </div>
      </div>
      <div>
        <div className="text-neutral-500 uppercase text-[10px]">TTFT</div>
        <div className="text-amber-400 font-bold">{ttftMs} ms</div>
      </div>
    </div>
  );
};
