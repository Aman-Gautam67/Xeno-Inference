import { useEffect, useState, useRef } from "react";

export interface WebSocketStatus {
  connected: boolean;
  url: string;
  latencyMs: number;
}

export const useXenoWebSocket = (url = "ws://127.0.0.1:8080/stream") => {
  const [status, setStatus] = useState<WebSocketStatus>({
    connected: false,
    url,
    latencyMs: 0,
  });
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<number | null>(null);

  useEffect(() => {
    let pingInterval: number | null = null;
    let pingStartTime = 0;

    const connect = () => {
      try {
        const ws = new WebSocket(url);
        wsRef.current = ws;

        ws.onopen = () => {
          setStatus((prev) => ({ ...prev, connected: true }));
          // Ping for latency measurement
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
          setStatus((prev) => ({ ...prev, connected: false, latencyMs: 0 }));
          if (pingInterval) clearInterval(pingInterval);
          // Try reconnect in 4 seconds
          reconnectTimeoutRef.current = window.setTimeout(connect, 4000);
        };

        ws.onerror = () => {
          ws.close();
        };
      } catch {
        setStatus((prev) => ({ ...prev, connected: false }));
        reconnectTimeoutRef.current = window.setTimeout(connect, 4000);
      }
    };

    connect();

    return () => {
      if (wsRef.current) wsRef.current.close();
      if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current);
      if (pingInterval) clearInterval(pingInterval);
    };
  }, [url]);

  return status;
};
