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


#[derive(Serialize)]
pub struct VilleYearStats {
    pub ville: String,
    pub annee: u16,
    pub rang_min: u32,
    pub rang_max: u32,
}

#[derive(Serialize)]
pub struct BulkVilleAllYearsResponse {
    pub specialite: String,
    pub mode: String,
    pub data: Vec<VilleYearStats>,
}

#[derive(Serialize)]
pub struct BulkVilleStats {
    pub ville: String,
    pub rang_min: u32,
    pub rang_max: u32,
}

#[derive(Serialize)]
pub struct BulkResponse {
    pub specialite: String,
    pub annee: u16,
    pub mode: String,
    pub villes: Vec<BulkVilleStats>,
}
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

fn is_safe_param(param: &str) -> bool {
    !param.contains("..") 
    && !param.contains("/") 
    && param.len() <= 50
    && !param.trim().is_empty()
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
    if !is_safe_param(&specialite) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // ✅ AJOUTE - Valider la ville si elle existe
    if let Some(ville) = &params.get("ville") {
        if !is_safe_param(ville) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
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

    if !is_safe_param(&specialite) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // ✅ AJOUTE - Valider la ville si elle existe
    if let Some(ville) = &query.ville {
        if !is_safe_param(ville) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

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
    if !is_safe_param(&specialite) || !is_safe_param(&ville) {
        return Err(StatusCode::BAD_REQUEST);
    }
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

pub async fn get_stats_bulk(
    Path(specialite): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<BulkResponse>, StatusCode> {

    if !is_safe_param(&specialite) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let annee = params.get("annee")
        .and_then(|x| x.parse::<u16>().ok())
        .unwrap_or(2025);

    let mode_str = params.get("mode").map(|s| s.as_str()).unwrap_or("brute");
    let mode = Mode::from_str(mode_str);

    let records = get_records_by_specialite_mode(&specialite, Some(annee), mode);
    if records.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    use std::collections::HashMap;
    let mut map: HashMap<String, Vec<&Record>> = HashMap::new();

    for r in &records {
        map.entry(r.ville.to_uppercase())
            .or_insert_with(Vec::new)
            .push(r);
    }

    let mut villes_stats = Vec::new();
    for (ville, refs) in map {
        let stats = calculate_statistics(&refs);
        villes_stats.push(BulkVilleStats {
            ville,
            rang_min: stats.rang_min,
            rang_max: stats.rang_max,
        });
    }

    Ok(Json(BulkResponse {
        specialite,
        annee,
        mode: mode_str.to_string(),
        villes: villes_stats,
    }))
}




pub async fn get_stats_all_years(
    Path(specialite): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<BulkVilleAllYearsResponse>, StatusCode> {

    if !is_safe_param(&specialite) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mode_str = params.get("mode")
        .map(|s| s.as_str())
        .unwrap_or("brute");
    let mode = Mode::from_str(mode_str);

    // --- Récupère TOUTES les années disponibles
    let years = vec![2019, 2020, 2021, 2022, 2023, 2024, 2025];

    let mut results = Vec::new();

    for year in years {
        let records = get_records_by_specialite_mode(&specialite, Some(year), mode.clone());

        // group by ville
        use std::collections::HashMap;
        let mut map: HashMap<String, Vec<&Record>> = HashMap::new();

        for r in &records {
            map.entry(r.ville.to_uppercase())
                .or_default()
                .push(r);
        }

        // compute stats
        for (ville, recs) in map {
            let stats = calculate_statistics(&recs);
            results.push(VilleYearStats {
                ville,
                annee: year,
                rang_min: stats.rang_min,
                rang_max: stats.rang_max,
            });
        }
    }

    Ok(Json(BulkVilleAllYearsResponse {
        specialite,
        mode: mode_str.to_string(),
        data: results,
    }))
}