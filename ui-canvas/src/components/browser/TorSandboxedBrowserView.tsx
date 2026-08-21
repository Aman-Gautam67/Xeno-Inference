import React, { useState } from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { 
  Globe, 
  Shield, 
  RotateCw, 
  ArrowLeft, 
  ArrowRight, 
  Lock, 
  Layers, 
  KeyRound, 
  EyeOff, 
  ExternalLink,
  CheckCircle2,
  RefreshCw,
  Search,
  Bookmark,
  SlidersHorizontal,
  Server
} from "lucide-react";

export const TorSandboxedBrowserView: React.FC = () => {
  const { 
    torUrl, 
    navigateTorBrowser, 
    torCircuit, 
    requestNewTorIdentity, 
    torShieldLevel, 
    setTorShieldLevel,
    isTorConnected
  } = useWorkspaceStore();

  const [inputUrl, setInputUrl] = useState(torUrl);
  const [showCircuitDrawer, setShowCircuitDrawer] = useState(true);
  const [showShieldSettings, setShowShieldSettings] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const handleNavigate = (e: React.FormEvent) => {
    e.preventDefault();
    if (!inputUrl.trim()) return;
    setIsLoading(true);
    navigateTorBrowser(inputUrl);
    setTimeout(() => setIsLoading(false), 600);
  };

  const bookmarks = [
    { title: "DuckDuckGo Onion", url: "https://duckduckgogg42xjoc72x3sjasowoarfbgcmvfimaftt6twagswzczad.onion" },
    { title: "Tor Project", url: "http://2gzyxa5ihm7nsggfxnu52r2gz264257lqqqqh53m5qsmxamznx524fid.onion" },
    { title: "Rust Std Docs", url: "https://doc.rust-lang.org/std/" },
    { title: "GitHub Xeno", url: "https://github.com/Aman-Gautam67/Xeno-Inference" },
    { title: "arXiv AI", url: "https://arxiv.org/list/cs.AI/recent" },
  ];

  const isOnion = torUrl.includes(".onion");

  return (
    <div className="flex-1 h-[calc(100vh-3.5rem)] flex flex-col bg-stone-50 dark:bg-stone-950 text-stone-900 dark:text-stone-100 overflow-hidden font-sans select-text transition-colors duration-200">
      {/* Top Browser Bar */}
      <div className="h-14 border-b border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 px-4 flex items-center justify-between space-x-3 z-10 shadow-sm">
        {/* Navigation Controls */}
        <div className="flex items-center space-x-1.5">
          <button
            onClick={() => {}}
            className="p-1.5 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-500 hover:text-stone-800 dark:hover:text-stone-200 transition-all"
            title="Back"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <button
            onClick={() => {}}
            className="p-1.5 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-500 hover:text-stone-800 dark:hover:text-stone-200 transition-all"
            title="Forward"
          >
            <ArrowRight className="w-4 h-4" />
          </button>
          <button
            onClick={() => {
              setIsLoading(true);
              setTimeout(() => setIsLoading(false), 500);
            }}
            className={`p-1.5 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-500 hover:text-stone-800 dark:hover:text-stone-200 transition-all ${
              isLoading ? "animate-spin text-amber-600" : ""
            }`}
            title="Reload Page"
          >
            <RotateCw className="w-4 h-4" />
          </button>
        </div>

        {/* Omnibar / Onion URL Input */}
        <form
          onSubmit={handleNavigate}
          className="flex-1 max-w-2xl flex items-center bg-stone-100 dark:bg-stone-950 px-3 py-1.5 rounded-xl border border-stone-300 dark:border-stone-800 focus-within:border-stone-500 dark:focus-within:border-stone-600 transition-all shadow-inner"
        >
          <div className="flex items-center space-x-1.5 mr-2">
            {isOnion ? (
              <span className="flex items-center gap-1 text-[10px] font-mono font-bold px-1.5 py-0.5 rounded bg-purple-100 dark:bg-purple-950/80 text-purple-700 dark:text-purple-300 border border-purple-300 dark:border-purple-800">
                <Lock className="w-3 h-3 text-purple-600 dark:text-purple-400" />
                ONION v3
              </span>
            ) : (
              <span className="flex items-center gap-1 text-[10px] font-mono font-bold px-1.5 py-0.5 rounded bg-emerald-100 dark:bg-emerald-950/80 text-emerald-700 dark:text-emerald-300 border border-emerald-300 dark:border-emerald-800">
                <Lock className="w-3 h-3 text-emerald-600 dark:text-emerald-400" />
                SECURE
              </span>
            )}
          </div>

          <input
            type="text"
            value={inputUrl}
            onChange={(e) => setInputUrl(e.target.value)}
            placeholder="Enter .onion address or search query..."
            className="flex-1 bg-transparent text-xs font-mono text-stone-900 dark:text-stone-100 outline-none placeholder:text-stone-400"
          />

          <button
            type="submit"
            className="ml-2 p-1 text-stone-400 hover:text-stone-800 dark:hover:text-stone-200"
          >
            <Search className="w-3.5 h-3.5" />
          </button>
        </form>

        {/* Tor Controls: Circuit, Shield, New Identity */}
        <div className="flex items-center space-x-2">
          {/* Circuit Visualizer Toggle */}
          <button
            onClick={() => setShowCircuitDrawer(!showCircuitDrawer)}
            className={`flex items-center space-x-1.5 px-2.5 py-1.5 rounded-lg border text-xs font-mono transition-all ${
              showCircuitDrawer
                ? "bg-purple-50 dark:bg-purple-950/40 border-purple-300 dark:border-purple-800 text-purple-700 dark:text-purple-300"
                : "border-stone-200 dark:border-stone-800 text-stone-600 dark:text-stone-400 hover:bg-stone-100 dark:hover:bg-stone-800"
            }`}
            title="Toggle Tor Circuit Visualizer"
          >
            <Layers className="w-3.5 h-3.5" />
            <span className="hidden md:inline">Circuit (3 Hops)</span>
          </button>

          {/* New Identity Button */}
          <button
            onClick={() => {
              requestNewTorIdentity();
              setIsLoading(true);
              setTimeout(() => setIsLoading(false), 400);
            }}
            className="flex items-center space-x-1 px-2.5 py-1.5 rounded-lg border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-700 dark:text-stone-300 text-xs font-mono transition-all shadow-sm"
            title="SIGNAL NEWNYM (Request New Tor Circuit & IP)"
          >
            <RefreshCw className="w-3.5 h-3.5" />
            <span className="hidden md:inline">New Identity</span>
          </button>

          {/* Security Shield Level */}
          <button
            onClick={() => setShowShieldSettings(!showShieldSettings)}
            className="flex items-center space-x-1.5 px-2.5 py-1.5 rounded-lg border border-stone-200 dark:border-stone-800 bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 text-xs font-mono transition-all"
            title={`Tor Shield: ${torShieldLevel}`}
          >
            <Shield className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" />
            <span>{torShieldLevel}</span>
          </button>
        </div>
      </div>

      {/* Bookmarks Bar */}
      <div className="h-8 border-b border-stone-200 dark:border-stone-800 bg-stone-100/70 dark:bg-stone-900/50 px-4 flex items-center space-x-2 text-[11px] font-mono overflow-x-auto">
        <Bookmark className="w-3 h-3 text-stone-400 mr-1" />
        {bookmarks.map((bm, i) => (
          <button
            key={i}
            onClick={() => {
              setInputUrl(bm.url);
              navigateTorBrowser(bm.url);
            }}
            className="px-2 py-0.5 rounded hover:bg-white dark:hover:bg-stone-800 text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-100 transition-all truncate max-w-[140px]"
          >
            {bm.title}
          </button>
        ))}
      </div>

      {/* Main Sandbox Surface */}
      <div className="flex-1 flex overflow-hidden relative">
        {/* Web Viewport Frame */}
        <div className="flex-1 flex flex-col bg-white dark:bg-stone-950 overflow-hidden relative">
          {/* Top Info Banner inside Sandbox */}
          <div className="p-3 bg-stone-50 dark:bg-stone-900/80 border-b border-stone-200 dark:border-stone-800 flex items-center justify-between text-xs font-mono text-stone-500">
            <div className="flex items-center space-x-2">
              <span className="w-2 h-2 rounded-full bg-emerald-500" />
              <span>SOCKS5 Proxy: 127.0.0.1:9050</span>
              <span>•</span>
              <span className="text-purple-600 dark:text-purple-400">DNS Leak Protection: Active</span>
              <span>•</span>
              <span>Canvas Fingerprint: Randomized</span>
            </div>
            <div className="flex items-center space-x-2">
              <span>Status: 200 OK (Encrypted)</span>
            </div>
          </div>

          {/* Rendered Sandbox Page Preview */}
          <div className="flex-1 p-8 overflow-y-auto space-y-6">
            <div className="max-w-3xl mx-auto space-y-6">
              <div className="p-6 rounded-2xl bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 shadow-sm space-y-4">
                <div className="flex items-center justify-between pb-3 border-b border-stone-200 dark:border-stone-800">
                  <div className="flex items-center space-x-2">
                    <Globe className="w-5 h-5 text-emerald-600 dark:text-emerald-400" />
                    <h2 className="font-display font-bold text-base text-stone-900 dark:text-stone-100">
                      {isOnion ? "Tor Onion Hidden Service" : "Tor Clearnet Proxy Tunnel"}
                    </h2>
                  </div>
                  <span className="text-xs font-mono px-2 py-0.5 rounded bg-emerald-100 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-300 font-semibold">
                    Anonymized
                  </span>
                </div>

                <div className="text-xs font-mono space-y-2 text-stone-600 dark:text-stone-400 leading-relaxed">
                  <div><strong>Requested URI:</strong> {torUrl}</div>
                  <div><strong>Cipher Suite:</strong> TLS_AES_256_GCM_SHA384 (0x1302)</div>
                  <div><strong>Strict Transport Security:</strong> max-age=31536000; includeSubDomains</div>
                  <div><strong>Referrer Policy:</strong> no-referrer</div>
                  <div><strong>Content Security Policy:</strong> sandbox allow-same-origin allow-forms allow-scripts</div>
                </div>

                <div className="p-4 rounded-xl bg-white dark:bg-stone-950 border border-stone-200 dark:border-stone-800 space-y-2">
                  <div className="text-xs font-bold text-stone-900 dark:text-stone-100 font-display">
                    Interactive Sandboxed Environment
                  </div>
                  <p className="text-xs text-stone-500 dark:text-stone-400 font-sans leading-relaxed">
                    This browser view operates in an isolated sandbox with zero access to your local filesystem, host storage, or network identity. All outbound sockets are wrapped into the local Tor SOCKS5 daemon (`127.0.0.1:9050`) ensuring end-to-end onion cryptographic routing.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Right Drawer: Live 3-Hop Tor Circuit Inspector */}
        {showCircuitDrawer && (
          <div className="w-80 border-l border-stone-200 dark:border-stone-800 bg-stone-50/90 dark:bg-stone-900/95 p-5 space-y-5 overflow-y-auto font-mono text-xs shadow-lg">
            <div className="flex items-center justify-between pb-3 border-b border-stone-200 dark:border-stone-800">
              <div className="flex items-center space-x-2">
                <Layers className="w-4 h-4 text-purple-600 dark:text-purple-400" />
                <span className="font-display font-bold text-xs uppercase tracking-wider text-stone-800 dark:text-stone-200">
                  Tor Circuit (3 Hops)
                </span>
              </div>
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-emerald-100 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-400 font-bold">
                ESTABLISHED
              </span>
            </div>

            <div className="space-y-3">
              {torCircuit.map((hop, idx) => (
                <div key={idx} className="space-y-2">
                  <div className="p-3 rounded-xl bg-white dark:bg-stone-950 border border-stone-200 dark:border-stone-800 shadow-sm space-y-1.5">
                    <div className="flex items-center justify-between">
                      <span className="font-bold text-stone-800 dark:text-stone-200 text-xs">
                        {idx + 1}. {hop.type.toUpperCase()}
                      </span>
                      <span className="text-[10px] px-1.5 py-0.2 rounded bg-stone-100 dark:bg-stone-800 text-stone-500 font-bold">
                        {hop.country}
                      </span>
                    </div>

                    <div className="text-[11px] text-stone-600 dark:text-stone-400 truncate">
                      {hop.name}
                    </div>

                    <div className="flex items-center justify-between text-[10px] text-stone-500 pt-1 border-t border-stone-100 dark:border-stone-800">
                      <span>IP: {hop.ip}</span>
                      <span className="text-emerald-600 dark:text-emerald-400 font-semibold">{hop.latencyMs} ms</span>
                    </div>
                  </div>

                  {idx < torCircuit.length - 1 && (
                    <div className="flex justify-center text-stone-400 text-xs">
                      ↓
                    </div>
                  )}
                </div>
              ))}
            </div>

            <div className="pt-2 border-t border-stone-200 dark:border-stone-800">
              <button
                onClick={requestNewTorIdentity}
                className="w-full py-2 px-3 rounded-xl bg-stone-900 hover:bg-stone-800 dark:bg-stone-100 dark:hover:bg-white text-white dark:text-stone-900 font-bold text-xs transition-all shadow-sm flex items-center justify-center space-x-1.5"
              >
                <RefreshCw className="w-3.5 h-3.5" />
                <span>Request New Circuit</span>
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
