#![allow(unused)]

mod types;
mod data_loaders;
mod plotting;
use serde_json::{Value, json};
use polars::prelude::*;
use polars::datatypes::Float64Type;
use ndarray::{Array1,Array2, s};
use plotly::{Plot, Scatter, layout, common::*, plot::Trace, plot::Traces};
use plotly::color::{Color};
use plotly::layout::{Axis, Layout, Legend, Slider, SliderStep, Frame, SliderMethod, SliderCurrentValue};
use plotly::layout::update_menu::{Button, ButtonMethod, UpdateMenu, UpdateMenuDirection, UpdateMenuType};

use types::*;
use data_loaders::*;
use plotting::*;

fn main() {
    // LOADING LOGIC
    let rotation = load_rotation("data/rotation.json");
    let track_angle =  (rotation / 180.0) * std::f64::consts::PI;
    let positional_data: PositionalData = load_positional_data("data/positional_data_dict.json");
    let fastest_lap_data = load_fastest_lap_data("data/fastest_lap_pos.json");
    let abv = load_abbrevations("data/abv.json");
    let team_colors = load_team_colors("data/team_colors.json");
    let winner_abv = load_winner_abv("data/winner.json");
    // println!("Loaded positional data for {}", positional_data.get(abv.get("VER").unwrap()).unwrap());

    let x_series = positional_data.get(abv.get(winner_abv.as_str()).unwrap()).unwrap().column("X").unwrap();
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
    
    let frames_vec: Vec<i32> = df_with_pos.column("Frame").unwrap().unique().unwrap().sort(SortOptions::default()).unwrap().i32().unwrap().into_no_null_iter().collect();
    // let frames_vec: Vec<i32> = (0..5000).collect();
    let rotated = rotate2d(xy, track_angle);  // or track_angle if in radians
    df_with_pos.replace("X", Series::new("X".into(), rotated.slice(s![..,0]).to_vec())).unwrap();
    df_with_pos.replace("Y", Series::new("Y".into(), rotated.slice(s![..,1]).to_vec())).unwrap();
    // df_with_pos = df_with_pos.lazy().filter(col("Frame").lt(lit(5000))).collect().unwrap();

    // println!("Loaded positional data for {:?}", df_with_pos);
    
    let track = fastest_lap_data.to_ndarray::<Float64Type>(IndexOrder::Fortran).unwrap();
    let rotated_track = rotate2d(track, track_angle);
    // plot_track(rotated_track);
    let mut plot = Plot::new();
    let mut track_trace = Scatter::new(rotated_track.slice(s![.., 0]).to_vec(),
                                                     rotated_track.slice(s![.., 1]).to_vec());
    let line_trace = Line::new().width(0.9).color("red");
    track_trace = track_trace.show_legend(false)
                .hover_info(HoverInfo::None)
                .line(line_trace);
    let cloned_track_trace = track_trace.clone();


    let mut layout = Layout::new();
    layout = layout.x_axis(layout::Axis::new().visible(false))
                    .y_axis(layout::Axis::new().visible(false))
                    .title("Live Grand Prix")
                    .legend(Legend::new().title("Driver"));
    plot.add_trace(track_trace);
    
    // plot.show();
    


    // let frames_vec: Vec<i32> = df_with_pos.column("Frame").unwrap().unique().unwrap().sort(SortOptions::default()).unwrap().i32().unwrap().into_no_null_iter().collect();
    
    let min_frame = *frames_vec.first().unwrap_or(&0);

    let mut base_traces: Vec<Box<dyn Trace>> = Vec::new();
    for driver in abv.keys() {
        let filtered = df_with_pos.clone().lazy()
            .filter(col("Frame").eq(lit(min_frame)).and(col("Driver").eq(lit(driver.as_str()))))
            .select(&[col("X"), col("Y")])
            .collect().unwrap();
        let x: Vec<f64> = filtered.column("X").unwrap().f64().unwrap().into_no_null_iter().collect();
        let y: Vec<f64> = filtered.column("Y").unwrap().f64().unwrap().into_no_null_iter().collect();

        let trace: Box<dyn Trace> = Scatter::new(x, y)
            .mode(Mode::Markers)
            .text_array(vec![driver.clone()])
            .marker(Marker::new().color(team_colors.get(driver).unwrap().clone()))
            .name(driver.clone())
            .legend_group(driver.clone())
            .show_legend(true)
            .ids(vec![driver.clone()]);
        base_traces.push(trace);
    }
    plot.add_traces(base_traces);

    for &frame_val in &frames_vec {
        let mut frame_traces: Traces = Traces::new();
        frame_traces.push(cloned_track_trace.clone());
        for driver in abv.keys() {
            let filtered = df_with_pos.clone().lazy()
                .filter(col("Frame").eq(lit(frame_val)).and(col("Driver").eq(lit(driver.as_str()))))
                .select(&[col("X"), col("Y")])
                .collect().unwrap();
            let x: Vec<f64> = filtered.column("X").unwrap().f64().unwrap().into_no_null_iter().collect();
            let y: Vec<f64> = filtered.column("Y").unwrap().f64().unwrap().into_no_null_iter().collect();

            let trace: Box<dyn Trace> = Scatter::new(x, y)
                    .mode(Mode::Markers)
                    .text_array(vec![driver.clone()])
                    .marker(Marker::new().color(team_colors.get(driver).unwrap().clone()))
                    .name(driver.clone())
                    .legend_group(driver.clone())
                    .show_legend(true)
                    .ids(vec![driver.clone()]);
            frame_traces.push(trace);
        }
        let frame = Frame::new()
            .name(frame_val.to_string())
            .data(frame_traces);  // Adds the traces to the frame's data.
        plot.add_frame(frame);
    }

    let play_button = Button::new()
        .label("▶")
        .method(ButtonMethod::Animate)
        .args(json!([null,{"frame": {"duration": 10, "redraw": false}, "mode": "next", "fromcurrent": true, "transition": {"duration": 2, "easing": "linear"}}]));
    let pause_button = Button::new()
        .label("⏸")
        .method(ButtonMethod::Animate)
        .args(json!([[null],{"frame": {"duration": 0, "redraw": false}, "mode": "immediate", "fromcurrent": true, "transition": {"duration": 0, "easing": "linear"}}]));
    let update_menu = UpdateMenu::new()
        .buttons(vec![play_button, pause_button])
        .direction(UpdateMenuDirection::Left)
        .pad(Pad::new(70, 0, 10))
        .show_active(false)
        .ty(UpdateMenuType::Buttons)
        .x(0.1)
        .x_anchor(Anchor::Right)
        .y(0.0)
        .y_anchor(Anchor::Top);
    layout = layout.update_menus(vec![update_menu]);

    let mut slider_steps: Vec<SliderStep> = Vec::new();
    for &frame_val in &frames_vec {
        let step = SliderStep::new()
            .args(json!([[frame_val.to_string()],{"frame": {"duration": 0, "redraw": false}, "mode": "immediate", "fromcurrent": true, "transition": {"duration": 0, "easing": "linear"}}]))
            .label(frame_val.to_string())
            .method(SliderMethod::Animate);
        slider_steps.push(step);
    }
    let slider = Slider::new()
        .active(0)
        .current_value(SliderCurrentValue::new().prefix("Frame="))
        .length(0.9)
        .pad(Pad::new(60,10,0))
        .steps(slider_steps)
        .x(0.1)
        .x_anchor(Anchor::Left)
        .y(0.0)
        .y_anchor(Anchor::Top);
    layout = layout.sliders(vec![slider]);

    plot.set_layout(layout);
    // plot.show();
    plot.write_html("track.html");
    

}
