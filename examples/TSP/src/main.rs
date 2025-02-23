use plotters::backend::BitMapBackend;
use plotters::chart::ChartBuilder;
use plotters::drawing::IntoDrawingArea;
use plotters::element::Circle;
use plotters::prelude::{BLUE, Color, IntoFont, LineSeries, RED, WHITE};
use radiate::*;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let tsp_file_path = std::env::current_dir()?.join("radiate-examples/TSP/gr17.txt");
    let (distance_matrix, distance_points) = read_tsp_file(&tsp_file_path)?;

    let codex = PermutationCodex::new((0..distance_matrix.len()).collect());

    let engine = GeneticEngine::from_codex(codex)
        .minimizing()
        .population_size(250)
        .alter(alters!(PMXCrossover::new(0.4), SwapMutator::new(0.05)))
        .fitness_fn(move |genotype: Vec<usize>| {
            let mut total_distance = 0.0;
            for i in 0..genotype.len() {
                let j = (i + 1) % genotype.len();
                total_distance += distance_matrix[genotype[i]][genotype[j]];
            }

            total_distance
        })
        .build();

    let result = engine.run(move |ctx| {
        println!("[ {:?} ]: {:?}", ctx.index, ctx.score());
        ctx.index > 2500 || ctx.score().as_usize() == 2085
    });

    plot_tsp_solution(&result.best, &distance_points).unwrap();

    println!("{:?}", result.metrics);

    Ok(())
}

fn read_tsp_file(file_path: &PathBuf) -> io::Result<(Vec<Vec<f32>>, Vec<(f32, f32)>)> {
    let file = File::open(file_path)?;
    let lines = io::BufReader::new(file).lines();

    let mut dimension = 0;
    let mut edge_weights = Vec::new();
    let mut in_edge_weight_section = false;

    for line in lines.map_while(Result::ok) {
        if line.starts_with("DIMENSION") {
            dimension = line
                .split_whitespace()
                .last()
                .unwrap()
                .parse::<usize>()
                .unwrap();
        } else if line.starts_with("EDGE_WEIGHT_SECTION") {
            in_edge_weight_section = true;
        } else if line.starts_with("EOF") {
            break;
        } else if in_edge_weight_section {
            edge_weights.extend(line.split_whitespace().map(|s| s.parse::<f32>().unwrap()));
        }
    }

    let distance_matrix = create_distance_matrix(&edge_weights, dimension);
    let points = create_points_from_distances(&distance_matrix);

    Ok((distance_matrix, points))
}

fn create_distance_matrix(edge_weights: &[f32], dimensions: usize) -> Vec<Vec<f32>> {
    let mut index = 0;
    let mut distance_matrix = vec![vec![0.0; dimensions]; dimensions];

    for i in 0..dimensions {
        for j in 0..=i {
            distance_matrix[i][j] = edge_weights[index];
            distance_matrix[j][i] = edge_weights[index];
            index += 1;
        }
    }

    distance_matrix
}

fn create_points_from_distances(distances: &[Vec<f32>]) -> Vec<(f32, f32)> {
    let n = distances.len();
    let mut points = vec![(0_f32, 0_f32); n];

    for i in 1..n {
        let distance = distances[i][0];
        let angle = (i as f32) * (2.0 * std::f64::consts::PI as f32 / n as f32);
        points[i] = (distance * angle.cos(), distance * angle.sin());
    }

    points
}

fn plot_tsp_solution(
    tour: &[usize],
    points: &[(f32, f32)],
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = std::env::current_dir()?.join("radiate-examples/TSP/tsp_solution.png");
    let root = BitMapBackend::new(&file_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("TSP Solution", ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-1000.0..1000.0, -1000.0..1000.0)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(
        points
            .iter()
            .map(|p| Circle::new((p.0 as f64, p.1 as f64), 5, RED.filled())),
    )?;

    chart.draw_series(LineSeries::new(
        tour.iter()
            .map(|p| (points[*p].0 as f64, points[*p].1 as f64))
            .chain(std::iter::once((
                points[tour[0]].0 as f64,
                points[tour[0]].1 as f64,
            ))),
        &BLUE,
    ))?;

    root.present()?;
    Ok(())
}
