use std::sync::LazyLock;

use rand::random;

use radiate::*;

const KNAPSACK_SIZE: usize = 15;
const MAX_EPOCHS: i32 = 50;

static KNAPSACK: LazyLock<Knapsack> = LazyLock::new(|| Knapsack::new(KNAPSACK_SIZE));

fn main() {
    let codex = SubSetCodex::new(&KNAPSACK.items);

    let engine = GeneticEngine::from_codex(&codex)
        .max_age(MAX_EPOCHS)
        .fitness_fn(move |genotype: Vec<&Item>| Knapsack::fitness(&KNAPSACK.capacity, &genotype))
        .build();

    let result = engine.run(|output| {
        let value_total = Knapsack::value_total(&output.best);
        let weight_total = Knapsack::weight_total(&output.best);

        println!(
            "[ {:?} ]: Value={:?} Weight={:?}",
            output.index, value_total, weight_total
        );

        output.index == MAX_EPOCHS
    });

    println!(
        "Result Value Total=[ {:?} ]",
        Knapsack::value_total(&result.best)
    );
    println!(
        "Result Weigh Total=[ {:?} ]",
        Knapsack::weight_total(&result.best)
    );
    println!("Max Weight=[{:?}]", KNAPSACK.capacity);
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

    pub fn fitness(capacity: &f32, genotype: &Vec<&Item>) -> Score {
        let mut sum = 0_f32;
        let mut weight = 0_f32;
        for item in genotype {
            sum += item.value;
            weight += item.weight;
        }

        if weight > *capacity {
            Score::from_f32(0_f32)
        } else {
            Score::from_f32(sum)
        }
    }

    pub fn value_total(items: &Vec<&Item>) -> f32 {
        items.iter().fold(0_f32, |acc, item| acc + item.value)
    }

    pub fn weight_total(items: &Vec<&Item>) -> f32 {
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

unsafe impl Send for Item {}
unsafe impl Sync for Item {}

impl Item {
    pub fn new(weight: f32, value: f32) -> Self {
        Item { weight, value }
    }

    pub fn random_collection(size: usize) -> Vec<Item> {
        (0..size)
            .map(|_| Item::new(random::<f32>() * 100.0, random::<f32>() * 100.0))
            .collect()
    }
}