use radiate::random_provider;

const NUM_STEPS: usize = 100;

pub fn behavior_descriptor(pattern: &Vec<f32>) -> Vec<f32> {
    let trajectory = simulate_movement(pattern);
    if trajectory.is_empty() {
        return vec![0.0; 6];
    }

    let xs: Vec<f32> = trajectory.iter().map(|(x, _)| *x).collect();
    let ys: Vec<f32> = trajectory.iter().map(|(_, y)| *y).collect();

    // 1) Total distance traveled
    let mut total_distance = 0.0;
    for i in 1..trajectory.len() {
        let dx = xs[i] - xs[i - 1];
        let dy = ys[i] - ys[i - 1];
        total_distance += (dx * dx + dy * dy).sqrt();
    }

    // 2) Final distance from start
    let final_distance = (xs.last().unwrap().powi(2) + ys.last().unwrap().powi(2)).sqrt();

    // 3) Area covered (bounding box)
    let min_x = xs
        .iter()
        .fold(f32::INFINITY, |acc, &v| if v < acc { v } else { acc });
    let max_x = xs
        .iter()
        .fold(f32::NEG_INFINITY, |acc, &v| if v > acc { v } else { acc });
    let min_y = ys
        .iter()
        .fold(f32::INFINITY, |acc, &v| if v < acc { v } else { acc });
    let max_y = ys
        .iter()
        .fold(f32::NEG_INFINITY, |acc, &v| if v > acc { v } else { acc });
    let area = (max_x - min_x) * (max_y - min_y);

    // 4) Directional bias
    let mut dx_total = 0.0;
    let mut dy_total = 0.0;
    for i in 1..trajectory.len() {
        dx_total += xs[i] - xs[i - 1];
        dy_total += ys[i] - ys[i - 1];
    }
    let net = (dx_total * dx_total + dy_total * dy_total).sqrt();
    let directional_bias = if total_distance > 0.0 {
        net / total_distance
    } else {
        0.0
    };

    // 5) Path complexity (how "wiggly" the path is)
    let path_complexity = trajectory.len() as f32 / (total_distance + 1.0);

    // 6) Return-to-start tendency
    let return_to_start = 1.0 / (final_distance + 1.0);

    vec![
        total_distance,
        final_distance,
        area,
        directional_bias,
        path_complexity,
        return_to_start,
    ]
}

/// Simulate robot movement and return its trajectory.
pub fn simulate_movement(movement_pattern: &[f32]) -> Vec<(f32, f32)> {
    let mut x = 0.0_f32;
    let mut y = 0.0_f32;

    let step_size = movement_pattern.get(0).copied().unwrap_or(1.0);
    let angle_change = movement_pattern.get(1).copied().unwrap_or(0.1);
    let direction_bias = movement_pattern.get(2).copied().unwrap_or(0.0);
    let noise_level = movement_pattern.get(3).copied().unwrap_or(0.1);

    let mut current_angle = 0.0_f32;
    let mut trajectory = Vec::with_capacity(NUM_STEPS + 1);
    trajectory.push((x, y));

    for _ in 0..NUM_STEPS {
        let angle_noise = random_provider::range(0.0..noise_level.abs());

        current_angle += angle_change + angle_noise + direction_bias;

        let dx = step_size * current_angle.cos();
        let dy = step_size * current_angle.sin();

        x += dx;
        y += dy;
        trajectory.push((x, y));
    }

    trajectory
}
