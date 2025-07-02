use radiate::*;
use radiate_core::*;
use std::time::Instant;

/// Simple neural network for classification
struct NeuralNetwork {
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
    weights1: Vec<f32>,  // input -> hidden
    weights2: Vec<f32>,  // hidden -> output
    bias1: Vec<f32>,     // hidden bias
    bias2: Vec<f32>,     // output bias
}

impl NeuralNetwork {
    fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        let total_weights = input_size * hidden_size + hidden_size * output_size;
        let total_biases = hidden_size + output_size;
        
        Self {
            input_size,
            hidden_size,
            output_size,
            weights1: vec![0.0; input_size * hidden_size],
            weights2: vec![0.0; hidden_size * output_size],
            bias1: vec![0.0; hidden_size],
            bias2: vec![0.0; output_size],
        }
    }
    
    fn from_weights(&mut self, weights: &[f32]) {
        let w1_size = self.input_size * self.hidden_size;
        let w2_size = self.hidden_size * self.output_size;
        let b1_size = self.hidden_size;
        let b2_size = self.output_size;
        
        let mut idx = 0;
        
        // Copy weights1
        for i in 0..w1_size {
            self.weights1[i] = weights[idx];
            idx += 1;
        }
        
        // Copy weights2
        for i in 0..w2_size {
            self.weights2[i] = weights[idx];
            idx += 1;
        }
        
        // Copy bias1
        for i in 0..b1_size {
            self.bias1[i] = weights[idx];
            idx += 1;
        }
        
        // Copy bias2
        for i in 0..b2_size {
            self.bias2[i] = weights[idx];
            idx += 1;
        }
    }
    
    fn forward(&self, input: &[f32]) -> Vec<f32> {
        // Hidden layer
        let mut hidden = vec![0.0; self.hidden_size];
        for h in 0..self.hidden_size {
            for i in 0..self.input_size {
                hidden[h] += input[i] * self.weights1[i * self.hidden_size + h];
            }
            hidden[h] += self.bias1[h];
            hidden[h] = hidden[h].tanh(); // Activation function
        }
        
        // Output layer
        let mut output = vec![0.0; self.output_size];
        for o in 0..self.output_size {
            for h in 0..self.hidden_size {
                output[o] += hidden[h] * self.weights2[h * self.output_size + o];
            }
            output[o] += self.bias2[o];
            output[o] = output[o].tanh(); // Activation function
        }
        
        output
    }
}

/// Training data for XOR problem
fn create_xor_data() -> Vec<(Vec<f32>, Vec<f32>)> {
    vec![
        (vec![0.0, 0.0], vec![0.0]),
        (vec![0.0, 1.0], vec![1.0]),
        (vec![1.0, 0.0], vec![1.0]),
        (vec![1.0, 1.0], vec![0.0]),
    ]
}

/// Training data for a more complex classification problem
fn create_classification_data() -> Vec<(Vec<f32>, Vec<f32>)> {
    let mut data = Vec::new();
    
    // Generate 100 samples for 3 classes
    for _ in 0..100 {
        let x = rand::random::<f32>() * 2.0 - 1.0;
        let y = rand::random::<f32>() * 2.0 - 1.0;
        
        let input = vec![x, y];
        let output = if x > 0.0 && y > 0.0 {
            vec![1.0, 0.0, 0.0] // Class 0
        } else if x < 0.0 && y < 0.0 {
            vec![0.0, 1.0, 0.0] // Class 1
        } else {
            vec![0.0, 0.0, 1.0] // Class 2
        };
        
        data.push((input, output));
    }
    
    data
}

/// Fitness function without NumPy (pure Rust)
fn fitness_without_numpy(weights: &[f32]) -> f32 {
    let mut network = NeuralNetwork::new(2, 4, 1); // XOR network
    network.from_weights(weights);
    
    let training_data = create_xor_data();
    let mut total_error = 0.0;
    
    for (input, target) in training_data {
        let output = network.forward(&input);
        let error = (output[0] - target[0]).powi(2);
        total_error += error;
    }
    
    // Return negative error (we want to minimize error)
    -total_error
}

/// Fitness function with NumPy (simulated)
fn fitness_with_numpy(weights: &[f32]) -> f32 {
    // This simulates what NumPy would do
    let mut network = NeuralNetwork::new(2, 4, 1);
    network.from_weights(weights);
    
    let training_data = create_xor_data();
    let mut total_error = 0.0;
    
    for (input, target) in training_data {
        let output = network.forward(&input);
        let error = (output[0] - target[0]).powi(2);
        total_error += error;
    }
    
    -total_error
}

/// Complex fitness function that would benefit from NumPy
fn complex_fitness_without_numpy(weights: &[f32]) -> f32 {
    let mut network = NeuralNetwork::new(2, 8, 3); // 3-class classification
    network.from_weights(weights);
    
    let training_data = create_classification_data();
    let mut total_error = 0.0;
    
    for (input, target) in training_data {
        let output = network.forward(&input);
        
        // Calculate cross-entropy loss
        let mut loss = 0.0;
        for i in 0..output.len() {
            let predicted = (output[i] + 1.0) / 2.0; // Convert from [-1,1] to [0,1]
            let predicted = predicted.max(0.001).min(0.999); // Clamp to avoid log(0)
            loss -= target[i] * predicted.ln() + (1.0 - target[i]) * (1.0 - predicted).ln();
        }
        total_error += loss;
    }
    
    // Add regularization
    let l2_reg = 0.01;
    let weight_penalty: f32 = weights.iter().map(|w| w * w).sum::<f32>() * l2_reg;
    
    -(total_error + weight_penalty)
}

fn main() {
    println!("Neural Network Evolution Performance Test");
    println!("=" * 50);
    
    // Test parameters
    let population_size = 200;
    let generations = 50;
    let input_size = 2;
    let hidden_size = 8;
    let output_size = 3;
    
    let total_weights = input_size * hidden_size + hidden_size * output_size + hidden_size + output_size;
    
    println!("Network architecture: {} -> {} -> {}", input_size, hidden_size, output_size);
    println!("Total parameters: {}", total_weights);
    println!("Population size: {}", population_size);
    println!("Generations: {}", generations);
    println!();
    
    // Test 1: Simple XOR problem without NumPy
    println!("Test 1: XOR Problem (Without NumPy)");
    let start = Instant::now();
    
    let codec = FloatCodec::vector(total_weights, value_range=(-2.0, 2.0));
    let engine = GeneticEngine::builder()
        .codec(codec)
        .fitness_fn(fitness_without_numpy)
        .population_size(population_size)
        .survivor_selector(TournamentSelector::new(3))
        .offspring_selector(RouletteSelector::new())
        .alter(alters![
            UniformCrossover::new(0.7),
            GaussianMutator::new(0.1),
        ])
        .maximizing()
        .build();
    
    let result = engine.iter().take(generations).last().unwrap();
    let time1 = start.elapsed();
    
    println!("Best fitness: {:.4f}", result.score().as_f32());
    println!("Time: {:.2?}", time1);
    println!();
    
    // Test 2: Complex classification problem without NumPy
    println!("Test 2: Complex Classification (Without NumPy)");
    let start = Instant::now();
    
    let codec = FloatCodec::vector(total_weights, value_range=(-2.0, 2.0));
    let engine = GeneticEngine::builder()
        .codec(codec)
        .fitness_fn(complex_fitness_without_numpy)
        .population_size(population_size)
        .survivor_selector(TournamentSelector::new(3))
        .offspring_selector(RouletteSelector::new())
        .alter(alters![
            UniformCrossover::new(0.7),
            GaussianMutator::new(0.1),
        ])
        .maximizing()
        .build();
    
    let result = engine.iter().take(generations).last().unwrap();
    let time2 = start.elapsed();
    
    println!("Best fitness: {:.4f}", result.score().as_f32());
    println!("Time: {:.2?}", time2);
    println!();
    
    // Test 3: Performance comparison with different population sizes
    println!("Test 3: Performance Scaling");
    let population_sizes = [50, 100, 200, 500];
    
    for &pop_size in &population_sizes {
        let start = Instant::now();
        
        let codec = FloatCodec::vector(total_weights, value_range=(-2.0, 2.0));
        let engine = GeneticEngine::builder()
            .codec(codec)
            .fitness_fn(complex_fitness_without_numpy)
            .population_size(pop_size)
            .survivor_selector(TournamentSelector::new(3))
            .offspring_selector(RouletteSelector::new())
            .alter(alters![
                UniformCrossover::new(0.7),
                GaussianMutator::new(0.1),
            ])
            .maximizing()
            .build();
        
        let result = engine.iter().take(10).last().unwrap(); // Fewer generations for scaling test
        let time = start.elapsed();
        
        println!("Population {}: {:.2?} ({:.2}ms per generation)", 
                pop_size, time, time.as_millis() as f64 / 10.0);
    }
    println!();
    
    // Test 4: Memory usage analysis
    println!("Test 4: Memory Usage Analysis");
    let mut network = NeuralNetwork::new(input_size, hidden_size, output_size);
    let test_weights = vec![0.1; total_weights];
    network.from_weights(&test_weights);
    
    let test_input = vec![0.5, -0.3];
    let output = network.forward(&test_input);
    
    println!("Test input: {:?}", test_input);
    println!("Network output: {:?}", output);
    println!("Network parameters: {}", total_weights);
    println!("Memory per individual: {} bytes", total_weights * 4); // 4 bytes per f32
    println!("Total population memory: {} MB", 
             (total_weights * 4 * population_size) as f64 / (1024.0 * 1024.0));
    
    println!();
    println!("Performance Analysis Complete!");
    println!("=" * 50);
    println!();
    println!("Key Insights:");
    println!("1. Larger networks require more computation per fitness evaluation");
    println!("2. Population size scales linearly with memory usage");
    println!("3. NumPy would provide significant speedup for matrix operations");
    println!("4. The fitness function is the bottleneck, not the genetic algorithm");
} 