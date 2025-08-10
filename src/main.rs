use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
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
use crate::services::{CableColorService, ItemService, LoanService, StorageService, ContainerService};

pub type AppState = (
    Arc<StorageService>,
    Arc<CableColorService>,
    Arc<ItemService>,
    Arc<LoanService>,
    Arc<ContainerService>,
);




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
    let container_service = Arc::new(ContainerService::new(db_pool.clone()));

    // Create app states
    let app_state = (
        storage.clone(),
        cable_color_service,
        item_service.clone(),
        loan_service,
        container_service,
    );
    let api_routes = Router::new()
        // Item routes
        .route(
            "/items",
            get(handlers::list_items).post(handlers::create_item),
        )
        .route(
            "/items/:id",
            get(handlers::get_item)
                .put(handlers::update_item)
                .delete(handlers::delete_item),
        )
        .route("/items/:id/dispose", post(handlers::dispose_item))
        .route("/items/:id/undispose", post(handlers::undispose_item))
        .route("/items/:id/image", post(handlers::add_item_image))
        .route(
            "/items/by-label/:label_id",
            get(handlers::get_item_by_label),
        )
        .route(
            "/items/suggestions/connection_names",
            get(handlers::get_connection_names_suggestions),
        )
        .route(
            "/items/suggestions/storage_locations",
            get(handlers::get_storage_locations_suggestions),
        )
       .route(
           "/items/:itemId/active-loan",
           get(handlers::get_active_loan_for_item),
       )
       .route(
           "/items/bulk",
           axum::routing::delete(handlers::bulk_delete_items),
       )
       .route(
           "/items/bulk/disposed",
           axum::routing::put(handlers::bulk_update_items_disposed_status),
       )
       // Cable color routes
       .route(
           "/cable_colors",
           get(handlers::list_cable_colors).post(handlers::create_cable_color),
       )
        .route(
            "/cable_colors/:id",
            get(handlers::get_cable_color)
                .put(handlers::update_cable_color)
                .delete(handlers::delete_cable_color),
        )
        // Loan routes
        .route("/loans", get(handlers::list_loans).post(handlers::create_loan))
        .route("/loans/:id", get(handlers::get_loan))
        .route("/loans/:id/return", post(handlers::return_loan))
        .route("/loans/history", get(handlers::list_loans))
        // Label routes
        .route("/labels/generate", post(handlers::generate_labels))
        .route("/labels", get(handlers::get_label_info))
        // ID Check routes
        .route("/ids/check/:id", get(handlers::check_global_id))
        // Container routes
        .route(
            "/containers",
            get(handlers::list_containers).post(handlers::create_container),
        )
        .route(
            "/containers/:id",
            get(handlers::get_container)
                .put(handlers::update_container)
                .delete(handlers::delete_container),
        )
       .route(
           "/containers/bulk",
           axum::routing::delete(handlers::bulk_delete_containers),
       )
       .route(
           "/containers/bulk/disposed",
           axum::routing::put(handlers::bulk_update_containers_disposed_status),
       )
        .route(
            "/containers/check/:id",
            get(handlers::check_container_id),
        )
        .route(
            "/containers/by-location/:location",
            get(handlers::get_containers_by_location),
        )
        // Image routes - larger body limit for file uploads
        .route(
            "/images/upload",
            post(handlers::upload_image).layer(DefaultBodyLimit::max(
                config.storage.max_file_size_mb as usize * 1024 * 1024 * 2,
            )), // 2倍のマージンを設定
        )
       .route("/images/:filename", axum::routing::delete(handlers::delete_image))
        .with_state(app_state);

    let mut app = Router::new()
        .route("/", get(root))
        .route("/api/v1/health", get(health_check))
        .nest("/api/v1", api_routes)
        // ファイルアップロード用のボディサイズ制限を設定
        .layer(DefaultBodyLimit::max(
            config.storage.max_file_size_mb as usize * 1024 * 1024,
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_credentials(false),
        )
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
