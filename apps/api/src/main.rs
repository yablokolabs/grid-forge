use grid_forge_api::{build_router, ApiState};
use grid_forge_common::{telemetry, AppConfig};

#[tokio::main]
async fn main() {
    telemetry::init("grid-forge-api");
    let config = AppConfig::from_env().expect("load configuration");
    let addr = config.bind_addr;
    let state = ApiState::demo(config).await.expect("create demo state");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind API listener");
    tracing::info!(%addr, "grid-forge API listening");
    axum::serve(listener, build_router(state))
        .await
        .expect("run API server");
}
