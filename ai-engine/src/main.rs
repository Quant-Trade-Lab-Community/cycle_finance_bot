//! AI Engine — Cycle Finance yapay zeka agent katmanı (daemon).
//!
//! Periyodik döngü: sembol bağlamını toplar → agent'lar paralel çalışır →
//! koordinatör karar verir → risk gate → icra (paper ring / executiond).
//!
//! HTTP:
//!   GET /api/health   → durum
//!   GET /api/status   → son döngü özeti

use ai_engine::agents::Agent;
use ai_engine::agents::{AgentOutput, AgentRole};
use ai_engine::agents::coordinator::Coordinator;
use ai_engine::agents::risk::RiskAgent;
use ai_engine::agents::sentiment::SentimentAgent;
use ai_engine::agents::signal::SignalAgent;
use ai_engine::config::AiConfig;
use ai_engine::context::ContextBuilder;
use ai_engine::executor::Executor;
use ai_engine::gates::{GateOutcome, RiskGate};
use ai_engine::llm::{LlmProvider, make_provider};
use ai_engine::FinalDecision;
use axum::{extract::State, routing::get, Json, Router};
use parking_lot::RwLock;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;

const HTTP_ADDR: &str = "127.0.0.1:3110";

#[derive(Clone, Serialize)]
struct DecisionView {
    symbol: String,
    action: String,
    confidence: f64,
    quantity: String,
    risk_score: f64,
    sentiment: f64,
    veto: bool,
    rationale: String,
    outcome: String,
}

#[derive(Clone, Serialize)]
struct RunSummary {
    run_id: u64,
    ts_ms: u64,
    provider: String,
    decisions: Vec<DecisionView>,
}

struct AppState {
    started_at: u64,
    provider_name: String,
    last_run: RwLock<Option<RunSummary>>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let cfg = AiConfig::load();

    println!("═══════════════════════════════════════════════════");
    println!("  🤖 AI ENGINE — LLM Agent Katmanı");
    println!("  Semboller  : {}", cfg.schedule.symbols.join(", "));
    println!("  Periyot    : {} sn", cfg.schedule.interval_secs);
    println!("  İcra modu  : {} (onay: {})", cfg.execution.mode, cfg.execution.approval);
    println!("═══════════════════════════════════════════════════");

    let provider = make_provider(&cfg);
    match &provider {
        Some(p) => println!("🤖 LLM provider: {} ({})", p.name(), model_name(&cfg)),
        None => println!(
            "🤖 LLM provider yok → fail-safe HOLD modu.\n   ai.toml [providers] provider + OPENAI_API_KEY/ANTHROPIC_API_KEY ayarlayın."
        ),
    }

    let context_builder = ContextBuilder::new(&cfg);
    let risk_gate = RiskGate::new(&cfg);
    let executor = Executor::new(&cfg);
    let coordinator = Coordinator::new(provider.clone());
    let risk_agent = Arc::new(RiskAgent::new(provider.clone()));
    let sentiment_agent = Arc::new(SentimentAgent::new(provider.clone()));

    let app_state = Arc::new(AppState {
        started_at: ai_engine::now_ms(),
        provider_name: provider.as_ref().map(|p| p.name().to_string()).unwrap_or_else(|| "none".into()),
        last_run: RwLock::new(None),
    });

    // ── HTTP status API ──────────────────────────────────────────
    let router_state = app_state.clone();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/api/health", get(health))
            .route("/api/status", get(status))
            .with_state(router_state);
        let listener = tokio::net::TcpListener::bind(HTTP_ADDR)
            .await
            .expect("ai-engine port bind");
        axum::serve(listener, app).await.expect("ai-engine serve");
    });

    // ── Ana döngü ────────────────────────────────────────────────
    let mut run_id: u64 = 0;
    loop {
        run_id += 1;
        let summary = run_cycle(
            run_id,
            &cfg,
            &provider,
            &context_builder,
            &risk_gate,
            &executor,
            &coordinator,
            &risk_agent,
            &sentiment_agent,
        )
        .await;
        *app_state.last_run.write() = Some(summary);
        tokio::time::sleep(Duration::from_secs(cfg.schedule.interval_secs)).await;
    }
}

fn model_name(cfg: &AiConfig) -> String {
    match cfg.providers.provider.to_ascii_lowercase().as_str() {
        "openai" => cfg.providers.openai_model.clone(),
        "anthropic" => cfg.providers.anthropic_model.clone(),
        _ => "—".into(),
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_cycle(
    run_id: u64,
    cfg: &AiConfig,
    provider: &Option<Arc<dyn LlmProvider>>,
    context_builder: &ContextBuilder,
    risk_gate: &RiskGate,
    executor: &Executor,
    coordinator: &Coordinator,
    risk_agent: &Arc<RiskAgent>,
    sentiment_agent: &Arc<SentimentAgent>,
) -> RunSummary {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔁 DÖNGÜ #{run_id} @ {}", ai_engine::now_ms());
    let mut decisions = Vec::new();

    for symbol in &cfg.schedule.symbols {
        let ctx = context_builder.build(symbol, &cfg.schedule.symbols).await;

        if !ctx.is_healthy() {
            println!("⚠️  {symbol}: fiyat kaynağı sağlıksız — atlandı (price-feed çalışıyor mu?)");
            continue;
        }

        let mark = ctx.price.mark.max(ctx.price.last);
        risk_gate.on_mark(symbol, mark);

        let signal_agent = SignalAgent::new(provider.clone(), symbol);
        let signal_out = signal_agent.run(&ctx).await;
        let risk_out = risk_agent.run(&ctx).await;
        let sentiment_out = sentiment_agent.run(&ctx).await;

        let (signal, risk, sentiment) = match (signal_out, risk_out, sentiment_out) {
            (AgentOutput::Signal(s), AgentOutput::Risk(r), AgentOutput::Sentiment(se)) => (s, r, se),
            _ => unreachable!("agent çıktı türleri sabittir"),
        };

        println!(
            "  🧠 {symbol} sinyal: {} (güven {:.2}, qty {}) | ⚠️ risk: {:.2} veto:{} | 📰 duygu: {:.2}",
            signal.action.as_str(),
            signal.confidence,
            signal.quantity,
            risk.risk_score,
            risk.veto,
            sentiment.sentiment
        );

        let decision = coordinator.decide(&ctx, &signal, &risk, &sentiment).await;
        let mark_dec = Decimal::from_f64(mark).unwrap_or_default();
        let outcome = risk_gate.process(&decision, mark_dec, executor).await;

        println!("  🤖 [{symbol}] {}", outcome);
        decisions.push(decision_view(&decision, &outcome));
    }

    RunSummary {
        run_id,
        ts_ms: ai_engine::now_ms(),
        provider: provider.as_ref().map(|p| p.name().to_string()).unwrap_or_else(|| "none".into()),
        decisions,
    }
}

fn decision_view(d: &FinalDecision, outcome: &GateOutcome) -> DecisionView {
    let outcome_str = match outcome {
        GateOutcome::Executed(m) => format!("executed: {m}"),
        GateOutcome::Held(m) => format!("held: {m}"),
        GateOutcome::Rejected(m) => format!("rejected: {m}"),
    };
    DecisionView {
        symbol: d.symbol.clone(),
        action: d.action.as_str().to_string(),
        confidence: d.confidence,
        quantity: d.quantity.to_string(),
        risk_score: d.risk_score,
        sentiment: d.sentiment,
        veto: d.veto,
        rationale: d.rationale.clone(),
        outcome: outcome_str,
    }
}

async fn health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let has_run = state.last_run.read().is_some();
    Json(serde_json::json!({
        "status": "ok",
        "provider": state.provider_name,
        "started_at": state.started_at,
        "last_run": has_run,
        "agents": [
            AgentRole::Signal.as_str(),
            AgentRole::Risk.as_str(),
            AgentRole::Sentiment.as_str(),
            AgentRole::Coordinator.as_str(),
        ],
    }))
}

async fn status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    match state.last_run.read().as_ref() {
        Some(s) => Json(serde_json::to_value(s).unwrap_or_default()),
        None => Json(serde_json::json!({ "run_id": 0, "note": "henüz döngü çalışmadı" })),
    }
}
