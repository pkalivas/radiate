use crate::robot;
use plotly::{
    Plot, Scatter,
    common::{Mode, Title},
    layout::{Axis, GridPattern, Layout, LayoutGrid},
};

pub fn visualize_behaviors_grid(behaviors: &[Vec<f32>]) {
    let n = behaviors.len();
    if n == 0 {
        return;
    }

    let cols = (n as f64).sqrt().ceil() as usize;
    let rows = ((n + cols - 1) / cols).max(1);

    let grid = LayoutGrid::new()
        .rows(rows)
        .columns(cols)
        .pattern(GridPattern::Independent);

    let mut layout = Layout::new().title(Title::new()).grid(grid);

    layout = layout
        .x_axis(Axis::new().title(Title::new()))
        .y_axis(Axis::new().title(Title::new()).scale_anchor("x"));

    let mut plot = Plot::new();
    plot.set_layout(layout);

    for (i, behavior) in behaviors.iter().enumerate() {
        let name = format!("Behavior {}", i + 1);
        add_to_subplot(behavior, &mut plot, i, &name);
    }

    plot.show();
}

fn add_to_subplot(robot: &Vec<f32>, plot: &mut Plot, subplot_index: usize, name: &str) {
    let trajectory = robot::simulate_movement(robot);
    if robot.is_empty() {
        return;
    }

    let x: Vec<f64> = trajectory.iter().map(|(x, _)| *x as f64).collect();
    let y: Vec<f64> = trajectory.iter().map(|(_, y)| *y as f64).collect();

    // Axis names for this subplot
    // subplot_index = 0 -> "x", "y"
    // subplot_index = 1 -> "x2", "y2"
    // subplot_index = 2 -> "x3", "y3", ...
    let axis_suffix = if subplot_index == 0 {
        "".to_string()
    } else {
        (subplot_index + 1).to_string()
    };
    let x_axis_ref = format!("x{}", axis_suffix);
    let y_axis_ref = format!("y{}", axis_suffix);

    // Main path trace
    let path_trace = Scatter::new(x.clone(), y.clone())
        .name(name)
        .mode(Mode::Lines)
        .x_axis(x_axis_ref.as_str())
        .y_axis(y_axis_ref.as_str())
        .line(plotly::common::Line::new().width(2.0))
        .opacity(0.7);

    // Start point
    let start_trace = Scatter::new(vec![x[0]], vec![y[0]])
        .name(format!("{name} start"))
        .mode(Mode::Markers)
        .x_axis(x_axis_ref.as_str())
        .y_axis(y_axis_ref.as_str())
        .marker(plotly::common::Marker::new().size(8).color("green"))
        .show_legend(false);

    // End point
    let end_trace = Scatter::new(vec![*x.last().unwrap()], vec![*y.last().unwrap()])
        .name(format!("{name} end"))
        .mode(Mode::Markers)
        .x_axis(x_axis_ref.as_str())
        .y_axis(y_axis_ref.as_str())
        .marker(plotly::common::Marker::new().size(8).color("red"))
        .show_legend(false);

    // Direction segments every 10 steps
    let mut direction_traces = Vec::new();
    let step = 10usize;
    for i in (0..(x.len() - 1)).step_by(step) {
        let seg = Scatter::new(vec![x[i], x[i + 1]], vec![y[i], y[i + 1]])
            .mode(Mode::Lines)
            .x_axis(x_axis_ref.as_str())
            .y_axis(y_axis_ref.as_str())
            .show_legend(false)
            .line(plotly::common::Line::new().width(1.5).color("red"));
        direction_traces.push(seg);
    }

    plot.add_trace(path_trace);
    plot.add_trace(start_trace);
    plot.add_trace(end_trace);
    for t in direction_traces {
        plot.add_trace(t);
    }
}
