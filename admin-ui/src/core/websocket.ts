import * as postcard from './postcard';

export interface MetricsSnapshot {
    p99_latency_ns: number;
    pnl: number;
    free_balance: number;
    ring_buffer_usage: number;
}

export class HFTWebSocket {
    private ws: WebSocket | null = null;
    private listeners = new Map<string, Function>();

    connect() {
        this.ws = new WebSocket('ws://localhost:8080/ws');
        this.ws.binaryType = 'arraybuffer';
        
        this.ws.onmessage = (ev) => {
            try {
                // Postcard'dan veri deserialize ediliyor (Sıfır JSON overhead)
                // Dikkat: u64 ve i64 js'de tam doğru deserialize edilebilmeli, postcard-js bigint desteğine sahiptir.
                const delta = postcard.decode(new Uint8Array(ev.data)) as MetricsSnapshot;
                
                Object.entries(delta).forEach(([key, val]) => {
                    if (this.listeners.has(key)) {
                        this.listeners.get(key)?.call(null, val);
                    }
                });
                
                // Ayrıca tüm objeyi dinleyenler için '*' eventi fırlatıyoruz
                if (this.listeners.has('*')) {
                    this.listeners.get('*')?.call(null, delta);
                }
            } catch (e) {
                console.error("Postcard decode error", e);
            }
        };

        this.ws.onclose = () => {
            console.log("WebSocket connection closed, retrying in 1s...");
            setTimeout(() => this.connect(), 1000);
        };
        
        this.ws.onerror = (err) => {
            console.error("WebSocket error", err);
        };
    }

    on(event: string, callback: Function) {
        this.listeners.set(event, callback);
    }
    
    sendCommand(cmd: string) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            // Şimdilik komutları JSON text olarak gönderiyoruz, rust tarafı ikisini de dinliyor
            this.ws.send(JSON.stringify({ cmd }));
        }
    }
}
