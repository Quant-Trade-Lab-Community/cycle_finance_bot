import './style.css';
import { HFTWebSocket, type MetricsSnapshot } from './core/websocket';
import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';

const app = document.querySelector<HTMLDivElement>('#app')!;

app.innerHTML = `
  <header>
    <h1>Demir Yumruk v3.0 - Titanium Core</h1>
    <div id="status">Connecting...</div>
  </header>
  <div class="dashboard">
    <div class="panel">
      <h2>P99 Latency (ns)</h2>
      <div id="latency-val" class="metric-value">0</div>
      <div id="chart-container"></div>
    </div>
    <div class="panel">
      <h2>Ring Buffer Usage (%)</h2>
      <div id="ring-val" class="metric-value">0%</div>
    </div>
    <div class="panel">
      <h2>Daily PnL (USDT)</h2>
      <div id="pnl-val" class="metric-value">0</div>
    </div>
    <div class="panel">
      <h2>Emergency Control</h2>
      <div class="kill-switch-container">
        <button id="kill-btn" class="kill-switch">
          <div class="kill-switch-progress" id="kill-progress"></div>
          <span class="kill-switch-text" id="kill-text">KILL</span>
        </button>
      </div>
    </div>
  </div>
`;

// Setup uPlot for latency
const chartOpts: uPlot.Options = {
    width: 300,
    height: 150,
    series: [
        {},
        {
            stroke: "#66FCF1",
            fill: "rgba(102, 252, 241, 0.1)",
        }
    ],
    axes: [
        { show: false },
        { stroke: "#C5C6C7", grid: { stroke: "#1F2833" } }
    ]
};

const chartData: [number[], number[]] = [[], []];
let chart: uPlot | null = null;
const chartContainer = document.getElementById('chart-container');
if (chartContainer) {
    chart = new uPlot(chartOpts, chartData, chartContainer);
}

// DOM Elements
const latencyEl = document.getElementById('latency-val')!;
const ringEl = document.getElementById('ring-val')!;
const pnlEl = document.getElementById('pnl-val')!;
const statusEl = document.getElementById('status')!;
const killBtn = document.getElementById('kill-btn')!;
const killProgress = document.getElementById('kill-progress')!;
const killText = document.getElementById('kill-text')!;

let latestData: MetricsSnapshot | null = null;

const ws = new HFTWebSocket();
ws.on('*', (data: MetricsSnapshot) => {
    latestData = data;
    statusEl.textContent = "CONNECTED (Zero-Copy)";
    statusEl.style.color = "var(--neon-green)";
});

// Render Loop using requestAnimationFrame
let lastTime = 0;
function renderLoop(time: number) {
    if (latestData && time - lastTime > 100) { // Update UI every 100ms
        latencyEl.textContent = latestData.p99_latency_ns.toString();
        ringEl.textContent = `${latestData.ring_buffer_usage}%`;
        pnlEl.textContent = latestData.pnl.toString();
        
        // Update Chart
        const now = Date.now() / 1000;
        chartData[0].push(now);
        chartData[1].push(latestData.p99_latency_ns);
        
        if (chartData[0].length > 50) {
            chartData[0].shift();
            chartData[1].shift();
        }
        
        if (chart) {
            chart.setData(chartData);
        }
        
        lastTime = time;
    }
    requestAnimationFrame(renderLoop);
}
requestAnimationFrame(renderLoop);

// Kill Switch Logic (Press and Hold for 3s)
let killTimer: number | null = null;
let killStartTime: number = 0;
let isDraining = false;

function updateKillProgress() {
    if (!killTimer) return;
    const elapsed = Date.now() - killStartTime;
    const percent = Math.min((elapsed / 3000) * 100, 100);
    killProgress.style.height = `${percent}%`;
    
    if (percent >= 100) {
        clearInterval(killTimer);
        killTimer = null;
        triggerKill();
    }
}

function startKillSequence() {
    if (isDraining) return;
    killStartTime = Date.now();
    killTimer = setInterval(updateKillProgress, 50);
}

function cancelKillSequence() {
    if (isDraining) return;
    if (killTimer) {
        clearInterval(killTimer);
        killTimer = null;
    }
    killProgress.style.height = '0%';
}

function triggerKill() {
    isDraining = true;
    killBtn.classList.add('draining');
    killText.textContent = "DRAINED";
    killProgress.style.height = '100%';
    killProgress.style.background = "rgba(102, 252, 241, 0.3)";
    ws.sendCommand("drain");
}

killBtn.addEventListener('pointerdown', startKillSequence);
killBtn.addEventListener('pointerup', cancelKillSequence);
killBtn.addEventListener('pointerleave', cancelKillSequence);

// Start connection
ws.connect();
