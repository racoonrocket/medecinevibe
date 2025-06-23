use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct Record {
    pub rang: u32,
    pub specialite_ville: String,
    pub annee: u16,
    pub ville: String,
    pub specialite: String,
    pub code_specialite: String,
}

#[derive(Debug, Serialize)]
pub struct Statistics {
    pub rang_median: u32,
    pub rang_q1: u32,
    pub rang_q3: u32,
    pub rang_min: u32,
    pub rang_max: u32,
    pub distribution: Vec<u32>,
    pub total_postes: usize,
    pub villes_disponibles: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RankingResponse {
    pub rang_estime: String,
    pub percentile: f64,
    pub villes_possibles: Vec<String>,
    pub stats_specialite: Statistics,
}