use std::fs;
use serde_json;
use polars::prelude::*;
use ndarray::{Array1, array, Array2};
use indexmap::IndexMap;
use crate::types::*;

pub fn load_driver_list(drivers_file_path: &str) -> Vec<String>{
    let data = fs::read_to_string(drivers_file_path).expect("Unable to read file");
    let drivers_list: Vec<String> = serde_json::from_str(&data).expect("Unable to parse JSON");
    drivers_list
}

pub fn load_rotation(rotation_path: &str) -> f64{
    let data = fs::read_to_string(rotation_path).expect("Unable to read file");
    let rotation: f64 = serde_json::from_str(&data).expect("Unable to parse JSON");
    rotation
}

pub fn load_positional_data(positional_path: &str) -> PositionalData{
    let positional_data_str = fs::read_to_string(positional_path).expect("Unable to read file");
    let temp_positional: IndexMap<String, TempData> = serde_json::from_str(&positional_data_str).expect("Unable to parse JSON");
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
        ]).expect("Failed to create DataFrame");
        positional_data.insert(driver, df);
    }
    positional_data
}

pub fn load_fastest_lap_data(fastest_lap_pos_path: &str) -> DataFrame {
    let fastest_lap_str = fs::read_to_string(fastest_lap_pos_path).expect("Unable to read file");
    let temp: TempData = serde_json::from_str(&fastest_lap_str).expect("Unable to parse JSON");
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
    ]).expect("Failed to create DataFrame");
    fastest_lap_df
}

pub fn load_abbrevations(abbrevations_path: &str) -> IndexMap<String, String>{
    let data = fs::read_to_string(abbrevations_path).expect("Unable to read file");
    let abvs: IndexMap<String, String> = serde_json::from_str(&data).expect("Unable to parse JSON");
    abvs
}

pub fn load_team_colors(colors_path: &str) -> IndexMap<String, String>{
    let data = fs::read_to_string(colors_path).expect("Unable to read file");
    let team_colors: IndexMap<String, String> = serde_json::from_str(&data).expect("Unable to parse JSON");
    team_colors
}

pub fn rotate2d(xy: Array2<f64>, angle: f64) -> Array2<f64>{
    let rot_mat = array![[angle.cos(), angle.sin()], [-angle.sin(), angle.cos()]];
    xy.dot(&rot_mat)
}

pub fn rotate1d(xy: Array1<f64>, angle: f64) -> Array1<f64>{
    let rot_mat = array![[angle.cos(), angle.sin()], [-angle.sin(), angle.cos()]];
    xy.dot(&rot_mat)
}
