use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod error;
mod handlers;
mod models;
mod services;

use crate::config::{Config, StorageType};
use crate::db::DatabasePool;
use crate::services::{
    CableColorService, ContainerService, ItemService, LoanService, StorageService,
};

pub type AppState = (
    Arc<StorageService>,
    Arc<CableColorService>,
    Arc<ItemService>,
    Arc<LoanService>,
);

#[derive(Clone)]
pub struct ContainerAppState {
    pub db: DatabasePool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hyperdashi_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting HyperDashi server...");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded: {:?}", config);

    // Initialize database connection
    let db_pool = DatabasePool::new(&config).await?;
    info!("Database connection established");

    // Run migrations
    db_pool.migrate().await?;
    info!("Database migrations completed");

    // Initialize storage
    let storage = Arc::new(StorageService::new(&config).await?);
    info!("Storage initialized");

    // Initialize services
    let cable_color_service = Arc::new(CableColorService::new(db_pool.clone()));
    let item_service = Arc::new(ItemService::new(db_pool.clone()));
    let loan_service = Arc::new(LoanService::new(db_pool.clone()));

    // Create app states
    let app_state = (
        storage.clone(),
        cable_color_service,
        item_service.clone(),
        loan_service,
    );
    let container_app_state = ContainerAppState {
        db: db_pool.clone(),
    };

    // Build application routes
    let mut app = Router::new()
        .route("/", get(root))
        .route("/api/v1/health", get(health_check))
        // Item routes
        .route(
            "/api/v1/items",
            get(handlers::list_items).post(handlers::create_item),
        )
        .route(
            "/api/v1/items/:id",
            get(handlers::get_item)
                .put(handlers::update_item)
                .delete(handlers::delete_item),
        )
        .route("/api/v1/items/:id/dispose", post(handlers::dispose_item))
        .route(
            "/api/v1/items/:id/undispose",
            post(handlers::undispose_item),
        )
        .route(
            "/api/v1/items/by-label/:label_id",
            get(handlers::get_item_by_label),
        )
        .route(
            "/api/v1/items/suggestions/connection_names",
            get(handlers::get_connection_names_suggestions),
        )
        .route(
            "/api/v1/items/suggestions/storage_locations",
            get(handlers::get_storage_locations_suggestions),
        )
        // Cable color routes
        .route(
            "/api/v1/cable_colors",
            get(handlers::list_cable_colors).post(handlers::create_cable_color),
        )
        .route(
            "/api/v1/cable_colors/:id",
            get(handlers::get_cable_color)
                .put(handlers::update_cable_color)
                .delete(handlers::delete_cable_color),
        )
        // Loan routes
        .route(
            "/api/v1/loans",
            get(handlers::list_loans).post(handlers::create_loan),
        )
        .route("/api/v1/loans/:id", get(handlers::get_loan))
        .route("/api/v1/loans/:id/return", post(handlers::return_loan))
        // Label routes
        .route("/api/v1/labels/generate", post(handlers::generate_labels))
        .route("/api/v1/labels", get(handlers::get_label_info))
        // Image routes - larger body limit for file uploads
        .route(
            "/api/v1/images/upload",
            post(handlers::upload_image).layer(DefaultBodyLimit::max(
                config.storage.max_file_size_mb as usize * 1024 * 1024 * 2,
            )), // 2倍のマージンを設定
        )
        // Add state
        .with_state(app_state)
        // Container routes with separate state
        .route(
            "/api/v1/containers",
            get(handlers::container_list).post(handlers::container_create),
        )
        .route(
            "/api/v1/containers/:id",
            get(handlers::container_get)
                .put(handlers::container_update)
                .delete(handlers::container_delete),
        )
        .route(
            "/api/v1/containers/by-location/:location",
            get(handlers::containers_by_location),
        )
        .with_state(container_app_state)
        // ファイルアップロード用のボディサイズ制限を設定
        .layer(DefaultBodyLimit::max(
            config.storage.max_file_size_mb as usize * 1024 * 1024,
        ))
        .layer({
            let cors_origins = env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,http://127.0.0.1:3000".to_string());

            info!("CORS allowed origins: {}", cors_origins);

            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_credentials(false)
        })
        .layer(TraceLayer::new_for_http());

    // Add static file serving for local storage
    if matches!(config.storage.storage_type, StorageType::Local) {
        if let Some(local_config) = &config.storage.local {
            info!(
                "Enabling static file serving for uploads at {}",
                local_config.path
            );
            app = app.nest_service("/uploads", ServeDir::new(&local_config.path));
        }
    }

    // Start server
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>()?,
        config.server.port,
    ));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "HyperDashi Server"
}

async fn health_check() -> &'static str {
    "OK"
}
