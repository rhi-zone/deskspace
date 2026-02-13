use std::net::SocketAddr;
use std::sync::Arc;

use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

use deskspace::api::{self, AppState};
use deskspace::projections::{dir_list, image_preview, text_markdown, text_raw};
use deskspace::registry::ProjectionRegistry;
use deskspace::workspace::Workspace;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let root = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());

    let workspace = Workspace::new(&root)?;
    tracing::info!("serving workspace: {}", workspace.root().display());

    let mut registry = ProjectionRegistry::new();
    registry.register(Arc::new(dir_list::DirList));
    registry.register(Arc::new(text_raw::TextRaw));
    registry.register(Arc::new(text_markdown::TextMarkdown));
    registry.register(Arc::new(image_preview::ImagePreview));

    let state = Arc::new(AppState {
        workspace,
        registry,
    });

    // UI is served from ui/ directory relative to the binary's working directory
    let ui_dir = std::env::current_exe()?
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.join("ui"))
        .unwrap_or_else(|| "ui".into());

    // Fallback: try ./ui relative to cwd
    let ui_dir = if ui_dir.exists() { ui_dir } else { "ui".into() };

    let app = api::router(state)
        .nest_service(
            "/ui/node_modules",
            ServeDir::new(ui_dir.join("node_modules")),
        )
        .fallback_service(ServeDir::new(&ui_dir));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
