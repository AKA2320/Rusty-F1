use std::fs;
use serde_json;
use polars::prelude::*;
use ndarray::{Array1, array, Array2};
use indexmap::IndexMap;
use crate::types::*;

pub fn load_driver_list(drivers_file_path: &str) -> Vec<String>{
    let data = fs::read_to_string(drivers_file_path).expect(&format!("Unable to read drivers list file: {drivers_file_path}"));
    let drivers_list: Vec<String> = serde_json::from_str(&data).expect(&format!("Unable to parse drivers list JSON from {drivers_file_path}"));
    drivers_list
}

pub fn load_rotation(rotation_path: &str) -> f64{
    let data = fs::read_to_string(rotation_path).expect(&format!("Unable to read rotation file: {rotation_path}"));
    let rotation: f64 = serde_json::from_str(&data).expect(&format!("Unable to parse rotation JSON from {rotation_path}"));
    rotation
}

pub fn load_positional_data(positional_path: &str) -> PositionalData{
    let positional_data_str = fs::read_to_string(positional_path).expect(&format!("Unable to read positional data file: {positional_path}"));
    let temp_positional: IndexMap<String, TempData> = serde_json::from_str(&positional_data_str).expect(&format!("Unable to parse positional data JSON from {positional_path}"));
    let mut positional_data: PositionalData = IndexMap::new();
    for (driver, temp) in temp_positional.into_iter() {
        let x_keys: Vec<String> = temp.X.keys().cloned().collect();
        let mut x_vec = Vec::new();
        let mut y_vec = Vec::new();
        for key in x_keys.iter() {
            x_vec.push(*temp.X.get(key).unwrap());
            y_vec.push(*temp.Y.get(key).unwrap());
        }
        let df: DataFrame = DataFrame::new(vec![
            Column::new("X".into(), x_vec),
            Column::new("Y".into(), y_vec),
        ]).expect(&format!("Failed to create positional DataFrame for driver {driver}"));
        positional_data.insert(driver, df);
    }
    positional_data
}

pub fn load_fastest_lap_data(fastest_lap_pos_path: &str) -> DataFrame {
    let fastest_lap_str = fs::read_to_string(fastest_lap_pos_path).expect(&format!("Unable to read fastest lap data file: {fastest_lap_pos_path}"));
    let temp: TempData = serde_json::from_str(&fastest_lap_str).expect(&format!("Unable to parse fastest lap data JSON from {fastest_lap_pos_path}"));
    let x_keys: Vec<String> = temp.X.keys().cloned().collect();
    let mut x_vec = Vec::new();
    let mut y_vec = Vec::new();
    for key in x_keys.iter() {
        x_vec.push(*temp.X.get(key).unwrap());
        y_vec.push(*temp.Y.get(key).unwrap());
    }

    let fastest_lap_df: DataFrame = DataFrame::new(vec![
        Column::new("X".into(), x_vec),
        Column::new("Y".into(), y_vec),
    ]).expect("Failed to create fastest lap DataFrame");
    fastest_lap_df
}

pub fn load_abbrevations(abbrevations_path: &str) -> IndexMap<String, String>{
    let data = fs::read_to_string(abbrevations_path).expect(&format!("Unable to read abbreviations file: {abbrevations_path}"));
    let abvs: IndexMap<String, String> = serde_json::from_str(&data).expect(&format!("Unable to parse abbreviations JSON from {abbrevations_path}"));
    abvs
}

pub fn load_team_colors(colors_path: &str) -> IndexMap<String, String>{
    let data = fs::read_to_string(colors_path).expect(&format!("Unable to read team colors file: {colors_path}"));
    let team_colors: IndexMap<String, String> = serde_json::from_str(&data).expect(&format!("Unable to parse team colors JSON from {colors_path}"));
    team_colors
}

pub fn load_winner_abv(winner_path: &str) -> String{
    let data = fs::read_to_string(winner_path).expect(&format!("Unable to read winner abbreviation file: {winner_path}"));
    let winner_abv: String = serde_json::from_str(&data).expect(&format!("Unable to parse winner abbreviation JSON from: {winner_path}"));
    winner_abv
}

pub fn rotate2d(xy: Array2<f64>, angle: f64) -> Array2<f64>{
    let rot_mat = array![[angle.cos(), angle.sin()], [-angle.sin(), angle.cos()]];
    xy.dot(&rot_mat)
}
