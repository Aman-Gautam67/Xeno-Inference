import { useEffect, useState, useRef } from "react";

export interface WebSocketStatus {
  connected: boolean;
  url: string;
  latencyMs: number;
}

export const useXenoWebSocket = (url?: string) => {
  const [status, setStatus] = useState<WebSocketStatus>({
    connected: false,
    url: url || "ws://127.0.0.1:8080/stream",
    latencyMs: 0,
  });
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    // Only attempt live WebSocket connection if explicitly enabled via window flag or localStorage
    const isWsEnabled = typeof window !== "undefined" && (
      (window as unknown as { __XENO_ENABLE_WS__?: boolean }).__XENO_ENABLE_WS__ === true ||
      localStorage.getItem("xeno_enable_ws") === "true"
    );

    if (!isWsEnabled) {
      return;
    }

    const targetUrl = url || "ws://127.0.0.1:8080/stream";
    let pingInterval: number | null = null;
    let pingStartTime = 0;

    try {
      const ws = new WebSocket(targetUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        setStatus({ connected: true, url: targetUrl, latencyMs: 0 });
        pingInterval = window.setInterval(() => {
          if (ws.readyState === WebSocket.OPEN) {
            pingStartTime = Date.now();
            ws.send(JSON.stringify({ type: "ping" }));
          }
        }, 5000);
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (data.type === "pong") {
            const latency = Date.now() - pingStartTime;
            setStatus((prev) => ({ ...prev, latencyMs: latency }));
          }
        } catch {
          // Ignore non-json frames
        }
      };

      ws.onclose = () => {
        setStatus({ connected: false, url: targetUrl, latencyMs: 0 });
        if (pingInterval) clearInterval(pingInterval);
      };

      ws.onerror = () => {
        if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
          ws.close();
        }
      };
    } catch {
      setStatus({ connected: false, url: targetUrl, latencyMs: 0 });
    }

    return () => {
      if (wsRef.current) wsRef.current.close();
      if (pingInterval) clearInterval(pingInterval);
    };
  }, [url]);

  return status;
};
