use axum::{
    extract::{Path, Query},
    response::Json,
    http::StatusCode,
};
use serde_json;
use crate::data_loader::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::stats::{calculate_statistics, estimate_ranking};
use crate::models::*;

#[derive(Deserialize)]
pub struct CalculateQuery {
    pub rang: u32,
    pub annee: Option<u16>,
    pub ville: Option<String>,
}

#[derive(Serialize)]
pub struct SpecialitesResponse {
    pub specialites: Vec<String>,
    pub villes: Vec<String>,
}

pub async fn get_specialites() -> Json<SpecialitesResponse> {
    Json(SpecialitesResponse {
        specialites: get_all_specialites(),
        villes: get_all_villes(),
    })
}

pub async fn get_stats(
    Path(specialite): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Statistics>, StatusCode> {
    let annee = params.get("annee")
        .and_then(|s| s.parse::<u16>().ok());
    let ville = params.get("ville");
    let mode_str = params.get("mode").map(|s| s.as_str()).unwrap_or("brute");
    let mode = Mode::from_str(mode_str);
    
    let mut records = get_records_by_specialite_mode(&specialite, annee, mode);
    
    if let Some(ville_filter) = ville {
        if !ville_filter.is_empty() {
            records = records.into_iter()
                .filter(|record| record.ville.to_uppercase() == ville_filter.to_uppercase())
                .collect();
        }
    }
    
    if records.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // MODIFICATION ICI : convertir Vec<Record> en Vec<&Record>
    let record_refs: Vec<&Record> = records.iter().collect();
    Ok(Json(calculate_statistics(&record_refs)))
}

pub async fn get_villes() -> Json<Vec<String>> {
    Json(get_all_villes())
}

pub async fn calculate_ranking(
    Path(specialite): Path<String>,
    Query(query): Query<CalculateQuery>,
) -> Result<Json<RankingResponse>, StatusCode> {
    let mut records = get_records_by_specialite(&specialite, query.annee);
    
    if let Some(ville_filter) = &query.ville {
        if !ville_filter.is_empty() {
            records = records.into_iter()
                .filter(|record| record.ville.to_uppercase() == ville_filter.to_uppercase())
                .collect();
        }
    }
    
    if records.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }
    
    Ok(Json(estimate_ranking(query.rang, &specialite, &records)))
}

pub async fn health_check() -> &'static str {
    "OK"
}

pub async fn get_min_max(
    Path((specialite, ville, annee)): Path<(String, String, u16)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some((min, max)) = get_min_max_by_specialite_ville_annee(&specialite, &ville, annee) {
        Ok(Json(serde_json::json!({
            "specialite": specialite,
            "ville": ville,
            "annee": annee,
            "rang_min": min,
            "rang_max": max
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}