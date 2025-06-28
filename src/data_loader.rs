use std::collections::HashMap;
use once_cell::sync::Lazy;
use crate::models::{Record, Mode};
use anyhow::Result;
use std::sync::Mutex;
type Year = u16;

pub static MODE_YEAR_CACHE: Lazy<Mutex<HashMap<(Mode, Year), Vec<Record>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});
// Données chargées une seule fois au démarrage (mode brute par défaut)
pub static DATA: Lazy<Vec<Record>> = Lazy::new(|| {
   load_csv_data().expect("Erreur lors du chargement des données")
});

// Index pour accès rapide par spécialité
pub static SPECIALITE_INDEX: Lazy<HashMap<String, Vec<usize>>> = Lazy::new(|| {
   create_specialite_index(&DATA)
});

// Index pour accès rapide par ville
pub static VILLE_INDEX: Lazy<HashMap<String, Vec<usize>>> = Lazy::new(|| {
   create_ville_index(&DATA)
});

fn load_csv_data() -> Result<Vec<Record>> {
   println!("Chargement des données CSV...");
   
   let mut reader = csv::Reader::from_path("data/donnees_medicales_enrichies2_brute.csv")?;
   let mut records = Vec::new();
   
   for result in reader.deserialize() {
       let record: Record = result?;
       records.push(record);
   }
   
   records.sort_by_key(|r| r.rang);
   
   Ok(records)
}

// Charger CSV selon le mode
fn load_csv_data_by_mode(mode: Mode, annee: u16) -> Result<Vec<Record>> {
   let suffix = mode.get_filename_suffix();
   let filename = format!("data/donnees_medicales_enrichies2_{}.csv", suffix);
   let mut reader = csv::Reader::from_path(filename)?;
   let mut records = Vec::new();
   
   for result in reader.deserialize() {
       let record: Record = result?;
       if record.annee == annee {
           records.push(record);
       }
   }
   
   records.sort_by_key(|r| r.rang);
   Ok(records)
}

fn create_specialite_index(data: &[Record]) -> HashMap<String, Vec<usize>> {
   let mut index = HashMap::new();
   
   for (i, record) in data.iter().enumerate() {
       index
           .entry(record.specialite.clone())
           .or_insert_with(Vec::new)
           .push(i);
   }
   
   index
}

fn create_ville_index(data: &[Record]) -> HashMap<String, Vec<usize>> {
   let mut index = HashMap::new();
   
   for (i, record) in data.iter().enumerate() {
       index
           .entry(record.ville.clone())
           .or_insert_with(Vec::new)
           .push(i);
   }
   
   index
}

// Nouvelle fonction avec mode
pub fn get_records_by_specialite_mode(specialite: &str, annee: Option<u16>, mode: Mode) -> Vec<Record> {
   let year = annee.unwrap_or(2024);
   
   if let Ok(data) = load_csv_data_by_mode(mode, year) {
       data.into_iter()
           .filter(|record| record.specialite == specialite)
           .collect()
   } else {
       Vec::new()
   }
}

// Accès rapide aux données par spécialité et année (ancienne fonction gardée)
pub fn get_records_by_specialite(specialite: &str, annee: Option<u16>) -> Vec<&Record> {
   if let Some(indices) = SPECIALITE_INDEX.get(specialite) {
       indices
           .iter()
           .map(|&i| &DATA[i])
           .filter(|record| {
               if let Some(year) = annee {
                   record.annee == year
               } else {
                   true
               }
           })
           .collect()
   } else {
       Vec::new()
   }
}

pub fn get_records_by_ville(ville: &str, annee: Option<u16>) -> Vec<&Record> {
   if let Some(indices) = VILLE_INDEX.get(ville) {
       indices
           .iter()
           .map(|&i| &DATA[i])
           .filter(|record| {
               if let Some(year) = annee {
                   record.annee == year
               } else {
                   true
               }
           })
           .collect()
   } else {
       Vec::new()
   }
}

pub fn get_all_specialites() -> Vec<String> {
   SPECIALITE_INDEX.keys().cloned().collect()
}

pub fn get_all_villes() -> Vec<String> {
   VILLE_INDEX.keys().cloned().collect()
}

pub fn get_last_rank_for_specialite(specialite: &str, annee: u16) -> Option<u32> {
   get_records_by_specialite(specialite, Some(annee))
       .iter()
       .map(|r| r.rang)
       .max()
}

pub fn get_min_max_by_specialite_ville_annee(specialite: &str, ville: &str, annee: u16) -> Option<(u32, u32)> {
   if let Some(indices) = SPECIALITE_INDEX.get(specialite) {
       let rangs: Vec<u32> = indices
           .iter()
           .map(|&i| &DATA[i])
           .filter(|record| {
               record.annee == annee && 
               record.ville.to_uppercase() == ville.to_uppercase()
           })
           .map(|record| record.rang)
           .collect();
       
       if rangs.is_empty() {
           return None;
       }
       
       let min = *rangs.iter().min().unwrap();
       let max = *rangs.iter().max().unwrap();
       
       Some((min, max))
   } else {
       None
   }
}