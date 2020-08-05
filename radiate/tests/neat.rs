#![feature(test)]

extern crate test;
use test::Bencher;

use radiate::prelude::*;
use radiate::models::neat::direction::NeuronDirection;

fn add_extra_nodes(neat: &mut Neat, count: usize) {
  // Create a few extra hidden nodes
  let dense: &mut Dense = neat.layers.last_mut().unwrap().as_mut();
  for _ in 0..count {
    dense.add_node(Activation::Sigmoid, NeuronDirection::Forward);
  }
}

fn create_neat(inputs: usize, hidden: usize, outputs: usize, pool: bool) -> Neat {
  let mut neat = Neat::new()
      .input_size(inputs as u32);
  if hidden > 0 {
    if pool {
      neat = neat.dense_pool(hidden as u32, Activation::Sigmoid);
      add_extra_nodes(&mut neat, 2);
    } else {
      neat = neat.dense(hidden as u32, Activation::Sigmoid);
    }
  }
  if pool {
    neat = neat.dense_pool(outputs as u32, Activation::Sigmoid);
    add_extra_nodes(&mut neat, 2);
  } else {
    neat = neat.dense(outputs as u32, Activation::Sigmoid);
  }

  neat
}

fn create_inputs(len: usize) -> Vec<f32> {
  let mut inputs = vec![];
  for val in 0..len {
    inputs.push(val as f32);
  }

  inputs
}

#[test]
fn test_create_neat() {
  let mut neat = create_neat(100, 50, 5, true);

  let inputs = create_inputs(100);
  let outputs = neat.forward(&inputs).expect("failed to run NEAT network");
  println!("outputs = {:?}", outputs);
}

#[bench]
fn bench_neat_dense_pool(b: &mut Bencher) {
  const INPUT_SIZE: usize = 25;
  let mut neat = create_neat(INPUT_SIZE, 0, 5, true);

  let inputs = create_inputs(INPUT_SIZE);

  b.iter(||{
    let n = test::black_box(2000);
    let first = neat.forward(&inputs).expect("failed to run NEAT network");
    for _ in 0..n {
      let check = neat.forward(&inputs).expect("failed to run NEAT network");
      assert_eq!(first, check);
    }
  });
}

#[bench]
fn bench_large_neat_dense_pool_with_hidden(b: &mut Bencher) {
  const INPUT_SIZE: usize = 100;
  let mut neat = create_neat(INPUT_SIZE, 50, 5, true);

  let inputs = create_inputs(INPUT_SIZE);

  b.iter(||{
    let n = test::black_box(200);
    let first = neat.forward(&inputs).expect("failed to run NEAT network");
    for _ in 0..n {
      let check = neat.forward(&inputs).expect("failed to run NEAT network");
      assert_eq!(first, check);
    }
  });
}

#[bench]
fn bench_neat_dense(b: &mut Bencher) {
  const INPUT_SIZE: usize = 25;
  let mut neat = create_neat(INPUT_SIZE, 0, 5, false);

  let inputs = create_inputs(INPUT_SIZE);

  b.iter(||{
    let n = test::black_box(2000);
    let first = neat.forward(&inputs).expect("failed to run NEAT network");
    for _ in 0..n {
      let check = neat.forward(&inputs).expect("failed to run NEAT network");
      assert_eq!(first, check);
    }
  });
}

#[bench]
fn bench_large_neat_dense_with_hidden(b: &mut Bencher) {
  const INPUT_SIZE: usize = 100;
  let mut neat = create_neat(INPUT_SIZE, 50, 5, false);

  let inputs = create_inputs(INPUT_SIZE);

  b.iter(||{
    let n = test::black_box(200);
    let first = neat.forward(&inputs).expect("failed to run NEAT network");
    for _ in 0..n {
      let check = neat.forward(&inputs).expect("failed to run NEAT network");
      assert_eq!(first, check);
    }
  });
}

