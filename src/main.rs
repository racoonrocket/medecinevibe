mod models;
mod data_loader;
mod stats;
mod handlers;

use axum::{
    http::Method,
    routing::get,
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Any},
    services::ServeDir,
};
use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer,
    key_extractor::SmartIpKeyExtractor,
};
use std::net::SocketAddr;
use crate::handlers::*;

#[tokio::main]
async fn main() {
    println!("🚀 Démarrage du serveur Internat Stats...");

    let _ = data_loader::DATA.len();
    let _ = data_loader::SPECIALITE_INDEX.len();
    let _ = data_loader::VILLE_INDEX.len();
    println!("✅ Données chargées.");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(vec![
            "https://medecinevibe.fr".parse().unwrap(),
            "https://www.medecinevibe.fr".parse().unwrap(),
            "https://medecinevibe.onrender.com".parse().unwrap(),
            "http://localhost:3000".parse().unwrap(),
        ])
        .allow_headers(Any);

    let governor_config = GovernorConfigBuilder::default()
        .per_second(700)
        .burst_size(170)
        .finish()
        .unwrap();

    let governor_layer = GovernorLayer {
        config: std::sync::Arc::new(governor_config),
    };

    let api_routes = Router::new()
        .route("/specialites", get(get_specialites))
        .route("/stats/:specialite", get(get_stats))
        .route("/calculate/:specialite", get(calculate_ranking))
        .route("/health", get(health_check))
        .route("/villes", get(get_villes))
        .route("/minmax/:specialite/:ville/:annee", get(get_min_max));

    let app = Router::new()
        .nest("/api", api_routes)
        .nest_service("/", ServeDir::new("static").precompressed_gzip())
        .layer(
            ServiceBuilder::new()
                .layer(governor_layer)
                .layer(cors),
        );

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT doit être un nombre");

   /* let addr = SocketAddr::from(([0, 0, 0, 0], port)); */
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("🌐 Serveur en écoute sur http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Impossible de bind l'adresse");
    
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("Erreur du serveur");
}