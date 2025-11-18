use polars::prelude::*;
use polars::datatypes::Float64Type;
use ndarray::{Array1, Array2, s, array};
use indexmap::IndexMap;
use crate::types::*;
use crate::data_loaders::*;


pub fn rotate2d(xy: Array2<f64>, angle: f64) -> Array2<f64>{
    let rot_mat = array![[angle.cos(), angle.sin()], [-angle.sin(), angle.cos()]];
    xy.dot(&rot_mat)
}

pub fn trim_lap_data(
    positional_data: &PositionalData,
    abv: &IndexMap<String, String>,
    winner_abv: &str,
) -> (usize, usize) {
    // Get winner's X data
    let winner_num = abv.get(winner_abv).unwrap();
    let winner_df = positional_data.get(winner_num).unwrap();
    let x_series = winner_df.column("X").unwrap();
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

    (start, end)
}

pub fn build_frame_dataframe(
    positional_data: &PositionalData,
    abv: &IndexMap<String, String>,
    start: usize,
    end: usize,
) -> DataFrame {
    let mut x_vec: Vec<f64> = Vec::new();
    let mut y_vec: Vec<f64> = Vec::new();
    let mut driver_vec: Vec<String> = Vec::new();

    for (driver_abbr, driver_num) in abv {
        if let Some(data_df) = positional_data.get(driver_num) {
            let x_series: Vec<f64> = data_df.column("X").unwrap().f64().unwrap().into_no_null_iter().collect();
            let y_series: Vec<f64> = data_df.column("Y").unwrap().f64().unwrap().into_no_null_iter().collect();
            let segment = &x_series[start..start + (end - start)];
            x_vec.extend_from_slice(segment);
            let segment_y = &y_series[start..start + (end - start)];
            y_vec.extend_from_slice(segment_y);
            driver_vec.extend(std::iter::repeat(driver_abbr.clone()).take(end - start));
        }
    }

    let num_drivers = abv.len() as i32;
    let len_per_driver = end as i32 - start as i32;
    let frame_base: Vec<i32> = (0..len_per_driver).collect();
    let frame_vec: Vec<i32> = frame_base.iter().cloned().cycle().take(x_vec.len()).collect();

    DataFrame::new(vec![
        Column::new("X".into(), x_vec),
        Column::new("Y".into(), y_vec),
        Column::new("Driver".into(), driver_vec),
        Column::new("Frame".into(), frame_vec),
    ]).unwrap()
}

pub fn get_frame_values(df: &DataFrame) -> Vec<i32> {
    df.column("Frame").unwrap()
        .unique().unwrap()
        .sort(SortOptions::default()).unwrap()
        .i32().unwrap()
        .into_no_null_iter()
        .filter(|&value| value % 10 == 0)
        .collect()
}

pub fn apply_rotation(df: &mut DataFrame, track_angle: f64) {
    let xy = df.select(["X", "Y"]).unwrap().to_ndarray::<Float64Type>(IndexOrder::Fortran).unwrap();
    let rotated = rotate2d(xy, track_angle);
    df.replace("X", Series::new("X".into(), rotated.slice(s![..,0]).to_vec())).unwrap();
    df.replace("Y", Series::new("Y".into(), rotated.slice(s![..,1]).to_vec())).unwrap();
}
