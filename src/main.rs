#![allow(unused)]

mod types;
mod data_loaders;
mod plotting;

use polars::prelude::*;
use polars::datatypes::Float64Type;
use ndarray::{Array1, s};
use indexmap::IndexMap;

use types::*;
use data_loaders::*;
use plotting::*;

fn main() {
    // LOADING LOGIC
    let rotation = load_rotation("rotation.json");
    let track_angle =  (rotation / 180.0) * std::f64::consts::PI;
    let positional_data: PositionalData = load_positional_data("positional_data_dict.json");
    let fastest_lap_data = load_fastest_lap_data("fastest_lap_pos.json");
    let abv = load_abbrevations("abv.json");
    // println!("Loaded positional data for {}", positional_data.get(abv.get("VER").unwrap()).unwrap());

    let x_series = positional_data.get(abv.get("VER").unwrap()).unwrap().column("X").unwrap();
    let x_vec: Vec<f64> = x_series.f64().unwrap().into_no_null_iter().collect();
    let x_ndarray = Array1::from(x_vec);
    let min_flat = 100;
    let diff: Vec<f64> = (0..x_ndarray.len() - 1)
        .map(|i| x_ndarray[i + 1] - x_ndarray[i])
        .collect();
    
    
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
    
    let track = fastest_lap_data.to_ndarray::<Float64Type>(IndexOrder::Fortran).unwrap();
    let rotated_track = rotate2d(track, track_angle);
    plot_track(rotated_track);
}
