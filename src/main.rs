#![allow(unused)]

mod types;
mod data_loaders;
mod data_processing;
mod plotting;
use data_loaders::*;
use data_processing::*;
use plotting::*;

fn main() {
    // Load data
    let rotation = load_rotation("data/rotation.json");
    let track_angle = (rotation / 180.0) * std::f64::consts::PI;
    let positional_data = load_positional_data("data/positional_data_dict.json");
    let fastest_lap_data = load_fastest_lap_data("data/fastest_lap_pos.json");
    let abv = load_abbrevations("data/abv.json");
    let team_colors = load_team_colors("data/team_colors.json");
    let winner_abv = load_winner_abv("data/winner.json");

    // Process data
    let (start, end) = trim_lap_data(&positional_data, &abv, &winner_abv);
    println!("Start: {}, End: {}", start, end);
    let mut df_with_pos = build_frame_dataframe(&positional_data, &abv, start, end);
    apply_rotation(&mut df_with_pos, track_angle);

    let frames_vec = get_frame_values(&df_with_pos);

    // Create track trace data
    let track = fastest_lap_data
        .select(["X", "Y"]).unwrap()
        .to_ndarray::<polars::datatypes::Float64Type>(polars::prelude::IndexOrder::Fortran).unwrap();
    let rotated_track = rotate2d(track, track_angle);

    // Create plot
    let plot = create_animated_race_plot(&df_with_pos, &frames_vec, rotated_track, &abv, &team_colors);
    println!("Generating the HTML");
    plot.write_html("track.html");
}
