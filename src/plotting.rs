use plotly::{Plot, Scatter, layout, layout::Layout, common::*, layout::*, layout::update_menu::*};
use plotly::color::{Color};
use plotly::plot::{Trace, Traces};
use ndarray::{Array2, s, array};
use polars::prelude::*;
use indexmap::IndexMap;
use serde_json::json;
use rayon::prelude::*;


pub fn plot_track(rotated_track: Array2<f64>) {
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

pub fn create_animated_race_plot(
    df_with_pos: &DataFrame,
    frames_vec: &[i32],
    rotated_track: Array2<f64>,
    abv: &IndexMap<String, String>,
    team_colors: &IndexMap<String, String>,
) -> Plot {
    #[derive(Clone)]
    struct DriverTraceData {
        x: Vec<f64>,
        y: Vec<f64>,
        driver: String,
    }
    let mut plot = Plot::new();
    // Create track trace
    let track_trace = Scatter::new(rotated_track.slice(s![.., 0]).to_vec(),
                                   rotated_track.slice(s![.., 1]).to_vec())
        .show_legend(false)
        .hover_info(HoverInfo::None)
        .line(Line::new().width(0.9).color("red"));

    let min_frame = frames_vec[0];

    // Base layout
    let mut layout = Layout::new();
    layout = layout.x_axis(Axis::new().visible(false))
                    .y_axis(Axis::new().visible(false))
                    .title("Live Grand Prix")
                    .width(9).height(7)
                    .plot_background_color("black")
                    .margin(Margin::new().left(1).right(1).top(1).bottom(1));;

    // Add track trace
    plot.add_trace(track_trace.clone());

    // Add base traces for all drivers at min_frame
    for driver in abv.keys() {
        let filtered = df_with_pos.clone().lazy()
            .filter(col("Frame").eq(lit(min_frame)).and(col("Driver").eq(lit(driver.as_str()))))
            .select(&[col("X"), col("Y")])
            .collect().unwrap();
        let x: Vec<f64> = filtered.column("X").unwrap().f64().unwrap().into_no_null_iter().collect();
        let y: Vec<f64> = filtered.column("Y").unwrap().f64().unwrap().into_no_null_iter().collect();

        let trace: Box<dyn Trace> = Scatter::new(x, y)
            .mode(Mode::MarkersText)
            .text(driver.clone())
            .text_position(Position::TopRight)
            .text_font(Font::new().color("white"))
            .marker(Marker::new().color(team_colors.get(driver).unwrap().clone()))
            .name(driver.clone())
            .legend_group(driver.clone())
            .show_legend(false)
            .ids(vec![driver.clone()]);
        plot.add_trace(trace);
    }

    // Add frames
    println!("Plotting total frames {}", frames_vec.len());
    let chunk_size = 50;
    let frames_data: Vec<Vec<DriverTraceData>> = (0..frames_vec.len()).into_par_iter().step_by(chunk_size).flat_map(|start| {
        let end = (start + chunk_size).min(frames_vec.len());
        let mut result = vec![];
        for i in start..end {
            let frame_val = frames_vec[i];
            let frame_data: Vec<DriverTraceData> = abv.keys().map(|driver| {
                let filtered = df_with_pos.clone().lazy()
                    .filter(col("Frame").eq(lit(frame_val)).and(col("Driver").eq(lit(driver.as_str()))))
                    .select(&[col("X"), col("Y")])
                    .collect().unwrap();
                let x: Vec<f64> = filtered.column("X").unwrap().f64().unwrap().into_no_null_iter().collect();
                let y: Vec<f64> = filtered.column("Y").unwrap().f64().unwrap().into_no_null_iter().collect();
                DriverTraceData {
                    x,
                    y,
                    driver: driver.clone(),
                }
            }).collect();
            result.push(frame_data);
        }
        result
    }).collect();

    for (frame_val, frame_data) in frames_vec.iter().zip(frames_data.iter()) {
        let mut frame_traces: Traces = Traces::new();
        frame_traces.push(track_trace.clone());
        for data in frame_data {
            let trace: Box<dyn Trace> = Scatter::new(data.x.clone(), data.y.clone())
                .mode(Mode::MarkersText)
                .text_array(vec![data.driver.clone()])
                .text_position(Position::TopRight)
                .text_font(Font::new().color("white"))
                .marker(Marker::new().color(team_colors.get(&data.driver).unwrap().clone()))
                .name(data.driver.clone())
                .legend_group(data.driver.clone())
                .show_legend(true)
                .ids(vec![data.driver.clone()]);
            frame_traces.push(trace);
        }
        let frame = Frame::new()
            .name(frame_val.to_string())
            .data(frame_traces);
        plot.add_frame(frame);
    }
    println!("Adding controls");

    // Add controls
    let play_button = Button::new()
        .label("▶")
        .method(ButtonMethod::Animate)
        .args(json!([null,{"frame": {"duration": 200, "redraw": false}, "mode": "next", "fromcurrent": true, "transition": {"duration": 200, "easing": "linear"}}]));
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
    for &frame_val in frames_vec {
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
    plot
}
