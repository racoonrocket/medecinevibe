mod models;
mod data_loader;
mod stats;
mod handlers;

use axum::{
    routing::get,
    Router,
    http::Method,
};
use crate::handlers::*;
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Any},
    services::ServeDir,
};
use std::net::SocketAddr;

use handlers::{get_specialites, get_stats, calculate_ranking, health_check};

#[tokio::main]
async fn main() {
    println!("🚀 Démarrage du serveur Internat Stats...");
    
    // Force le chargement des données au démarrage
    let _data_len = data_loader::DATA.len();
    let _specialites_len = data_loader::SPECIALITE_INDEX.len();
    let _villes_len = data_loader::VILLE_INDEX.len();
    
    println!("✅ Serveur prêt !");
    
    // Configuration CORS pour le frontend
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);
    
    // Routes API
    let api_routes = Router::new()
        .route("/specialites", get(get_specialites))
        .route("/stats/:specialite", get(get_stats))
        .route("/calculate/:specialite", get(calculate_ranking))
        .route("/health", get(health_check))
        .route("/villes", get(get_villes))
        .route("/minmax/:specialite/:ville/:annee", get(get_min_max));
    
    let app = Router::<()>::new()
        .nest("/api", api_routes)
        .nest_service("/", ServeDir::new("static"))
        .layer(ServiceBuilder::new().layer(cors));
        
   
    let port = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string()) 
    .parse::<u16>()
    .expect("PORT doit être un nombre");

    let addr = SocketAddr::from(([0, 0, 0, 0], port)); 
    println!("🌐 Serveur en écoute sur http://0.0.0.0:{}", port);
    println!("🌐 Serveur en écoute sur http://{}", addr);

    /*
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🌐 Serveur en écoute sur http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
    */
}