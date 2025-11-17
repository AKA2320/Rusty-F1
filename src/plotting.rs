use plotly::{Plot, Scatter, layout, layout::Layout, common::*, layout::*};
use ndarray::{Array2, s};

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
