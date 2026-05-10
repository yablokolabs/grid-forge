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
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("run API server");
    tracing::info!("grid-forge API shut down gracefully");
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler");
        tokio::select! {
            _ = ctrl_c => tracing::info!("received SIGINT"),
            _ = sigterm.recv() => tracing::info!("received SIGTERM"),
        }
    }
    #[cfg(not(unix))]
    {
        ctrl_c.await.expect("install Ctrl+C handler");
        tracing::info!("received Ctrl+C");
    }
}
