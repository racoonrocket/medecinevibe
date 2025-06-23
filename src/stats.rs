use crate::models::{Record, Statistics, RankingResponse};
use std::collections::HashSet;

pub fn calculate_statistics(records: &[&Record]) -> Statistics {
    if records.is_empty() {
        return Statistics {
            rang_median: 0,
            rang_q1: 0,
            rang_q3: 0,
            rang_max: 0,
            rang_min: 0,
            distribution: Vec::new(),
            total_postes: 0,
            villes_disponibles: Vec::new(),
        };
    }
    
    let mut rangs: Vec<u32> = records.iter().map(|r| r.rang).collect();
    rangs.sort();
    
    // Collecte les villes uniques
    let villes: HashSet<String> = records.iter().map(|r| r.ville.clone()).collect();
    let mut villes_vec: Vec<String> = villes.into_iter().collect();
    villes_vec.sort();
    
    Statistics {
        rang_median: percentile_u32(&rangs, 50.0),
        rang_q1: percentile_u32(&rangs, 25.0),
        rang_q3: percentile_u32(&rangs, 75.0),
        rang_min: rangs[0],                    // 👈 Premier élément (plus petit)
        rang_max: rangs[rangs.len() - 1],     // 👈 AJOUTEZ ÇA
        distribution: create_rank_histogram(&rangs, 10),
        total_postes: records.len(),
        villes_disponibles: villes_vec,
    }
}

pub fn estimate_ranking(user_rank: u32, specialite: &str, records: &[&Record]) -> RankingResponse {
    let mut rangs: Vec<u32> = records.iter().map(|r| r.rang).collect();
    rangs.sort();
    
    // Trouve les villes possibles pour ce rang
    let villes_possibles: Vec<String> = records
        .iter()
        .filter(|r| r.rang >= user_rank)
        .map(|r| r.ville.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    
    // Calcule le percentile (combien de postes sont accessibles)
    let postes_accessibles = records.iter().filter(|r| r.rang >= user_rank).count();
    let percentile = (postes_accessibles as f64 / records.len() as f64) * 100.0;
    
    // Message selon le rang
    let rang_message = if postes_accessibles > 0 {
        format!("Vous pourriez obtenir {} poste(s) en {}", postes_accessibles, specialite)
    } else {
        format!("Rang trop élevé pour {} cette année", specialite)
    };
    
    RankingResponse {
        rang_estime: rang_message,
        percentile,
        villes_possibles,
        stats_specialite: calculate_statistics(records),
    }
}

fn percentile_u32(sorted_data: &[u32], percentile: f64) -> u32 {
    if sorted_data.is_empty() {
        return 0;
    }
    
    let index = (percentile / 100.0) * (sorted_data.len() - 1) as f64;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    
    if lower == upper {
        sorted_data[lower]
    } else {
        let weight = index - lower as f64;
        let result = sorted_data[lower] as f64 * (1.0 - weight) + sorted_data[upper] as f64 * weight;
        result.round() as u32
    }
}

fn create_rank_histogram(data: &[u32], bins: usize) -> Vec<u32> {
    if data.is_empty() {
        return vec![0; bins];
    }
    
    let min = data[0];
    let max = data[data.len() - 1];
    let bin_size = (max - min) as f64 / bins as f64;
    
    let mut histogram = vec![0u32; bins];
    
    for &value in data {
        let bin_index = if bin_size > 0.0 {
            ((value - min) as f64 / bin_size) as usize
        } else {
            0
        };
        let bin_index = bin_index.min(bins - 1);
        histogram[bin_index] += 1;
    }
    
    histogram
}
