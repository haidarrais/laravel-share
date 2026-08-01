//! Localhost-only web inspector (`http://127.0.0.1:<port>`) mirroring the
//! terminal log, with request replay ("resend this webhook").

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};

use crate::log::CapturedExchange;

#[derive(Clone)]
pub struct InspectorState {
    pub store: Arc<crate::log::ExchangeStore>,
    pub local_port: u16,
    pub basic_auth: Option<String>,
}

/// Spawn the inspector HTTP server on `127.0.0.1:<port>`.
pub async fn spawn(
    store: Arc<crate::log::ExchangeStore>,
    local_port: u16,
    basic_auth: Option<String>,
    port: u16,
) -> anyhow::Result<()> {
    let state = InspectorState {
        store,
        local_port,
        basic_auth,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/exchanges", get(list_exchanges))
        .route("/api/exchanges/{id}", get(get_exchange))
        .route("/api/exchanges/{id}/replay", get(replay))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!("\x1b[2mInspector\x1b[0m    http://127.0.0.1:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> impl IntoResponse {
    axum::response::Html(INDEX_HTML)
}

async fn list_exchanges(State(state): State<InspectorState>) -> impl IntoResponse {
    let items: Vec<_> = state
        .store
        .all()
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.request.id,
                "method": e.request.method,
                "path": e.request.path,
                "status": e.status,
                "latency_ms": e.latency_ms,
                "received_at": e.request.received_at.to_rfc3339(),
            })
        })
        .collect();
    axum::response::Json(serde_json::json!({ "exchanges": items }))
}

async fn get_exchange(
    State(state): State<InspectorState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.store.by_id(&id) {
        Some(e) => axum::response::Json(exchange_json(&e)).into_response(),
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

async fn replay(State(state): State<InspectorState>, Path(id): Path<String>) -> impl IntoResponse {
    let Some(e) = state.store.by_id(&id) else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    match crate::forward::forward(
        state.local_port,
        &e.request.method,
        &e.request.path,
        &e.request.query,
        &e.request.headers,
        e.request.body.clone(),
        state.basic_auth.as_deref(),
    )
    .await
    {
        Ok(resp) => {
            let mut value = exchange_json(&e);
            let mut map = value.as_object_mut().unwrap().clone();
            map.insert("replay_status".into(), serde_json::json!(resp.status));
            map.insert(
                "replay_body".into(),
                serde_json::json!(String::from_utf8_lossy(&resp.body)),
            );
            map.insert(
                "replay_latency_ms".into(),
                serde_json::json!(resp.latency_ms),
            );
            axum::response::Json(serde_json::Value::Object(map)).into_response()
        }
        Err(err) => (StatusCode::BAD_GATEWAY, err.to_string()).into_response(),
    }
}

fn exchange_json(e: &CapturedExchange) -> serde_json::Value {
    serde_json::json!({
        "id": e.request.id,
        "method": e.request.method,
        "path": e.request.path,
        "query": e.request.query,
        "headers": e.request.headers,
        "body": String::from_utf8_lossy(&e.request.body),
        "status": e.status,
        "latency_ms": e.latency_ms,
    })
}

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>Artisan Share Inspector</title>
<style>
  :root { color-scheme: dark; }
  body { font-family: "Geist Mono", ui-monospace, monospace; background:#0d1117; color:#c9d1d9; margin:0; }
  header { padding: 16px 24px; border-bottom:1px solid #21262d; }
  header h1 { font-size:18px; margin:0; color:#58a6ff; }
  table { width:100%; border-collapse:collapse; }
  th, td { text-align:left; padding:8px 24px; border-bottom:1px solid #161b22; font-size:13px; }
  th { color:#8b949e; font-weight:600; text-transform:uppercase; font-size:11px; }
  tr:hover { background:#161b22; cursor:pointer; }
  .method { font-weight:700; }
  #detail { padding:24px; white-space:pre-wrap; font-size:13px; }
  #detail h2 { color:#58a6ff; font-size:15px; }
  button { background:#238636; color:#fff; border:0; border-radius:6px; padding:8px 16px; cursor:pointer; }
</style>
</head>
<body>
<header><h1>Artisan Share Inspector</h1></header>
<div id="list"><table><thead><tr><th>Method</th><th>Path</th><th>Status</th><th>Latency</th><th>Time</th></tr></thead><tbody id="rows"></tbody></table></div>
<div id="detail" style="display:none"></div>
<script>
const $ = (s)=>document.querySelector(s);
async function refresh(){
  const r = await fetch('/api/exchanges');
  const d = await r.json();
  const rows = $('#rows');
  rows.innerHTML = d.exchanges.map(e=>
    `<tr data-id="${e.id}"><td class="method">${e.method}</td><td>${e.path}</td><td>${e.status}</td><td>${e.latency_ms}ms</td><td>${e.received_at}</td></tr>`
  ).join('');
  document.querySelectorAll('#rows tr').forEach(tr=>{
    tr.onclick = async ()=>{
      const rr = await fetch('/api/exchanges/'+tr.dataset.id);
      const x = await rr.json();
      $('#list').style.display='none'; $('#detail').style.display='block';
      $('#detail').innerHTML =
        `<h2>${x.method} ${x.path}</h2>` +
        `<button onclick="replay('${x.id}')">Replay</button><div id="replayOut"></div>` +
        `<h3>Headers</h3><pre>${escapeHtml(JSON.stringify(x.headers,null,2))}</pre>` +
        `<h3>Body</h3><pre>${escapeHtml(x.body)}</pre>`;
    };
  });
}
async function replay(id){
  const r = await fetch('/api/exchanges/'+id+'/replay');
  const x = await r.json();
  $('#replayOut').textContent = 'Replayed → status ' + (x.replay_status||'?') + ' in ' + (x.replay_latency_ms||'?') + 'ms';
}
function escapeHtml(s){ return s.replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c])); }
refresh(); setInterval(refresh, 2000);
</script>
</body>
</html>"#;
