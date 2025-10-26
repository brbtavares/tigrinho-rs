use axum::http::StatusCode;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use axum_extra::TypedHeader;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

use tigrinho_core::{spin_once, EngineParams, Paytable, ProvablyFairRng, ReelsConfig};
use tigrinho_shared::{AdminSetParamsRequest, SpinRequest, SpinResponse, VerifyResponse};

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    api_key: String,
}

// DB schema is defined in migrations (see migrations/ folder)

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct StoredParams {
    server_seed: String,
    server_seed_hash: String,
    rtp_target: f64,
    paytable_json: String,
    nonce: i64,
}

async fn get_params(pool: &SqlitePool) -> anyhow::Result<StoredParams> {
    let row = sqlx::query_as::<_, StoredParams>(
        "SELECT server_seed, server_seed_hash, rtp_target, paytable_json, nonce FROM params WHERE id = 1"
    ).fetch_one(pool).await?;
    Ok(row)
}

async fn set_params(pool: &SqlitePool, p: &StoredParams) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE params SET server_seed = ?, server_seed_hash = ?, rtp_target = ?, paytable_json = ?, nonce = ? WHERE id = 1"
    )
    .bind(&p.server_seed)
    .bind(&p.server_seed_hash)
    .bind(p.rtp_target)
    .bind(&p.paytable_json)
    .bind(p.nonce)
    .execute(pool).await?;
    Ok(())
}

async fn init_db(db: &SqlitePool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(db).await?;
    // ensure server_seed_hash matches server_seed
    let mut p = get_params(db).await?;
    let hash = tigrinho_core::derive_hash_hex(p.server_seed.as_bytes());
    if p.server_seed_hash != hash {
        p.server_seed_hash = hash;
        set_params(db, &p).await?;
    }
    Ok(())
}

async fn route_verify(State(state): State<Arc<AppState>>) -> Json<VerifyResponse> {
    let p = get_params(&state.db).await.expect("params");
    Json(VerifyResponse {
        server_seed_hash: p.server_seed_hash,
    })
}

async fn route_spin(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SpinRequest>,
) -> Result<Json<SpinResponse>, StatusCode> {
    if req.bet <= 0.0 || req.lines == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut p = get_params(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    p.nonce += 1;
    let rng = ProvablyFairRng::new(&p.server_seed, &req.client_seed, p.nonce as u64);
    let paytable: Vec<tigrinho_core::PaytableEntry> = serde_json::from_str(&p.paytable_json)
        .unwrap_or_else(|_| tigrinho_core::Paytable::simple_default().0);
    let params = EngineParams {
        reels: ReelsConfig::default_3x3(),
        paytable: tigrinho_core::Paytable(paytable),
        rtp_target: p.rtp_target,
    };
    let outcome = spin_once(&rng, &params, req.bet, req.lines);

    // log spin
    let reels_indices: Vec<Vec<u8>> = outcome
        .reel_window
        .iter()
        .map(|row| row.iter().map(|s| s.to_index()).collect())
        .collect();
    let reels_json = serde_json::to_string(&reels_indices).unwrap();
    let ts = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO spins (ts, client_seed, nonce, server_seed_hash, result_reels_json, payout) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(ts)
    .bind(&req.client_seed)
    .bind(p.nonce)
    .bind(&p.server_seed_hash)
    .bind(reels_json)
    .bind(outcome.payout)
    .execute(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // persist incremented nonce
    set_params(&state.db, &p)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SpinResponse {
        server_seed_hash: p.server_seed_hash,
        nonce: p.nonce as u64,
        reels: reels_indices,
        payout: outcome.payout,
    }))
}

async fn route_admin_set_params(
    State(state): State<Arc<AppState>>,
    TypedHeader(axum_extra::headers::Authorization(bearer)): TypedHeader<
        axum_extra::headers::Authorization<axum_extra::headers::authorization::Bearer>,
    >,
    Json(req): Json<AdminSetParamsRequest>,
) -> Result<StatusCode, StatusCode> {
    if bearer.token() != state.api_key {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let mut p = get_params(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    p.rtp_target = req.rtp_target;
    p.paytable_json = serde_json::to_string(&req.paytable).unwrap_or("[]".to_string());
    set_params(&state.db, &p)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(
            &std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://tigrinho.db".to_string()),
        )
        .await?;
    init_db(&db).await?;

    let state = Arc::new(AppState {
        db,
        api_key: std::env::var("API_KEY").unwrap_or_else(|_| "dev-key".into()),
    });

    let app = Router::new()
        .route("/verify", get(route_verify))
        .route("/spin", post(route_spin))
        .route("/admin/set-params", post(route_admin_set_params))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let addr = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
