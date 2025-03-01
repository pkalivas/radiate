use std::sync::Arc;

use radiate::*;

const KNAPSACK_SIZE: usize = 25;
const MAX_EPOCHS: i32 = 50;

fn main() {
    random_provider::set_seed(12345);
    let knapsack = Knapsack::new(KNAPSACK_SIZE);
    let capacity = knapsack.capacity;
    let codex = SubSetCodex::new(knapsack.items);

    let engine = GeneticEngine::from_codex(codex)
        .max_age(MAX_EPOCHS)
        .fitness_fn(move |genotype: Vec<Arc<Item>>| Knapsack::fitness(&capacity, &genotype))
        .build();

    let result = engine.run(|ctx| {
        let value_total = Knapsack::value_total(&ctx.best);
        let weight_total = Knapsack::weight_total(&ctx.best);

        println!(
            "[ {:?} ]: Value={:?} Weight={:?}",
            ctx.index, value_total, weight_total
        );

        ctx.index == MAX_EPOCHS
    });

    println!(
        "Result Value Total=[ {:?} ]",
        Knapsack::value_total(&result.best)
    );
    println!(
        "Result Weigh Total=[ {:?} ]",
        Knapsack::weight_total(&result.best)
    );
    println!("Max Weight=[{:?}]", capacity);
}

pub struct Knapsack {
    pub capacity: f32,
    pub size: usize,
    pub items: Vec<Item>,
}

impl Knapsack {
    pub fn new(size: usize) -> Self {
        let items = Item::random_collection(size);
        Knapsack {
            capacity: size as f32 * 100_f32 / 3_f32,
            size,
            items,
        }
    }

    pub fn fitness(capacity: &f32, genotype: &Vec<Arc<Item>>) -> f32 {
        let mut sum = 0_f32;
        let mut weight = 0_f32;
        for item in genotype {
            sum += item.value;
            weight += item.weight;
        }

        if weight > *capacity { 0_f32 } else { sum }
    }

    pub fn value_total(items: &Vec<Arc<Item>>) -> f32 {
        items.iter().fold(0_f32, |acc, item| acc + item.value)
    }

    pub fn weight_total(items: &Vec<Arc<Item>>) -> f32 {
        items.iter().fold(0_f32, |acc, item| acc + item.weight)
    }
}

impl std::fmt::Debug for Knapsack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sum = 0_f32;
        for item in &self.items {
            sum += item.value;
        }

        write!(
            f,
            "Knapsack[capacity={:.2}, size={:.2}, sum={:.2}]",
            self.capacity, self.size, sum
        )
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub weight: f32,
    pub value: f32,
}

impl Item {
    pub fn new(weight: f32, value: f32) -> Self {
        Item { weight, value }
    }

    pub fn random_collection(size: usize) -> Vec<Item> {
        (0..size)
            .map(|_| {
                Item::new(
                    random_provider::random::<f32>() * 100.0,
                    random_provider::random::<f32>() * 100.0,
                )
            })
            .collect()
    }
}
