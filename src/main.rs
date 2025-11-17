#![allow(unused)]

use polars::datatypes::Float64Type;
use polars::df;
use std::fs;
use serde_json;
use serde::Deserialize;
use polars::prelude::*;
use ndarray::{Array1, Axis, array, Array2, s};
use indexmap::IndexMap;
use plotly::{Plot, Scatter, layout, layout::Layout};
use plotly::common::*;



type PositionalData = IndexMap<String, DataFrame>;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct TempData {
    X: IndexMap<String, f64>,
    Y: IndexMap<String, f64>,
}


fn load_driver_list(drivers_file_path: &str) -> Vec<String>{
    let data = fs::read_to_string(drivers_file_path).expect("Unable to read file");
    let drivers_list: Vec<String> = serde_json::from_str(&data).expect("Unable to parse JSON");
    drivers_list
}

fn load_rotation(rotation_path: &str) -> f64{
    let data = fs::read_to_string(rotation_path).expect("Unable to read file");
    let rotation: f64 = serde_json::from_str(&data).expect("Unable to parse JSON");
    rotation
}

fn load_positional_data(positional_path: &str) -> PositionalData{
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

fn load_fastest_lap_data(fastest_lap_pos_path: &str) -> DataFrame {
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

fn load_abbrevations(abbrevations_path: &str) -> IndexMap<String, String>{
    let data = fs::read_to_string(abbrevations_path).expect("Unable to read file");
    let abvs: IndexMap<String, String> = serde_json::from_str(&data).expect("Unable to parse JSON");
    abvs
}

fn load_team_colors(colors_path: &str) -> IndexMap<String, String>{
    let data = fs::read_to_string(colors_path).expect("Unable to read file");
    let team_colors: IndexMap<String, String> = serde_json::from_str(&data).expect("Unable to parse JSON");
    team_colors
}

fn rotate2d(xy: Array2<f64>, angle: f64) -> Array2<f64>{
    let rot_mat = array![[angle.cos(), angle.sin()], [-angle.sin(), angle.cos()]];
    xy.dot(&rot_mat)
}

fn rotate1d(xy: Array1<f64>, angle: f64) -> Array1<f64>{
    let rot_mat = array![[angle.cos(), angle.sin()], [-angle.sin(), angle.cos()]];
    xy.dot(&rot_mat)
}


fn main() {
    let drivers_list = load_driver_list("drivers_list.json");
    let rotation = load_rotation("rotation.json");
    let track_angle =  (rotation / 180.0) * std::f64::consts::PI;
    let positional_data: PositionalData = load_positional_data("positional_data_dict.json");

    

    // let fastest_lap_pos_path = ;
    let fastest_lap_data = load_fastest_lap_data("fastest_lap_pos.json");
    let abv = load_abbrevations("abv.json");
    let team_colors = load_team_colors("team_colors.json");
    

    // println!("Loaded positional data for {}", positional_data.get(abv.get("VER").unwrap()).unwrap());
    let x_series = positional_data.get(abv.get("VER").unwrap()).unwrap().column("X").unwrap();
    
    let x_vec: Vec<f64> = x_series.f64().unwrap().into_no_null_iter().collect();
    let x_ndarray = Array1::from(x_vec);
    
    let min_flat = 100;
    // Compute differences: diff[i] = x_ndarray[i+1] - x_ndarray[i]
    let diff: Vec<f64> = (0..x_ndarray.len() - 1)
        .map(|i| x_ndarray[i + 1] - x_ndarray[i])
        .collect();
    // let diff = x_ndarray.diff(1, Axis(0));
    
    // Find change indices where |diff| >= 10
    let change_indices: Vec<usize> = diff
        .iter()
        .enumerate()
        .filter(|(_, d)| d.abs() >= 10.0)
        .map(|(i, _)| i + 1)
        .collect();
    
    // Determine start
    let mut start = 0;
    if !change_indices.is_empty() && change_indices[0] >= min_flat {
        start = change_indices[0];
    }
    // Determine end
    let mut end = x_ndarray.len();
    if !change_indices.is_empty() {
        let last_change = change_indices[change_indices.len() - 1];
        let trailing_len = x_ndarray.len() - last_change;
        if trailing_len >= min_flat {
            end = last_change;
        }
    }

    println!("Start: {}, End: {}", start, end);

    let mut x_vec: Vec<f64> = Vec::new();
    let mut y_vec: Vec<f64> = Vec::new();
    let mut driver_vec: Vec<String> = Vec::new();

    for (driver_abbr, driver_num) in &abv {
        if let Some(data_df) = positional_data.get(driver_num) {
            let x_series = data_df.column("X").unwrap().f64().unwrap().into_no_null_iter().collect::<Vec<_>>();
            let y_series = data_df.column("Y").unwrap().f64().unwrap().into_no_null_iter().collect::<Vec<_>>();
            let len = end - start;
            x_vec.extend_from_slice(&x_series[start..start + len]);
            y_vec.extend_from_slice(&y_series[start..start + len]);
            driver_vec.extend(std::iter::repeat(driver_abbr.clone()).take(len));
        }
    }
    let num_drivers = abv.len() as i32;
    let len_per_driver = end as i32 - start as i32;
    let frame_base: Vec<i32> = (0..len_per_driver).collect();
    let frame_vec: Vec<i32> = frame_base.iter().cloned().cycle().take(x_vec.len()).collect();

    let mut df_with_pos = DataFrame::new(vec![
        Column::new("X".into(), x_vec),
        Column::new("Y".into(), y_vec),
        Column::new("Driver".into(), driver_vec),
        Column::new("Frame".into(), frame_vec),
    ]).unwrap();

    let xy = df_with_pos.select(["X", "Y"]).unwrap().to_ndarray::<Float64Type>(IndexOrder::Fortran).unwrap();
    
    let rotated = rotate2d(xy, track_angle);  // or track_angle if in radians
    df_with_pos.replace("X", Series::new("X".into(), rotated.slice(s![..,0]).to_vec())).unwrap();
    df_with_pos.replace("Y", Series::new("Y".into(), rotated.slice(s![..,1]).to_vec())).unwrap();
    df_with_pos = df_with_pos.lazy().filter(col("Frame").lt(lit(1000))).collect().unwrap();

    println!("Loaded positional data for {:?}", df_with_pos);
    // panic!();
    
    let track = fastest_lap_data.to_ndarray::<Float64Type>(IndexOrder::Fortran).unwrap();
    let rotated_track = rotate2d(track, track_angle);
    // println!("fastest_lap_data: {:?}", &rotated_track);
    let mut plot = Plot::new();
    let mut trace = Scatter::new(rotated_track.slice(s![.., 0]).to_vec(),
                                                     rotated_track.slice(s![.., 1]).to_vec());
    let line_trace = Line::new().width(0.9).color("red");
    trace = trace.show_legend(false)
                .hover_info(HoverInfo::None)
                .line(line_trace);

    let mut layout = Layout::new();
    layout = layout.x_axis(layout::Axis::new().visible(false))
                    .y_axis(layout::Axis::new().visible(false));
    plot.add_trace(trace);
    plot.set_layout(layout);
    plot.show();
}
