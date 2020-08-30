use std::error::Error;
use std::time::Instant;
use std::sync::{Arc, RwLock};
use rand::Rng;
use radiate::prelude::*;

#[test]
fn helloworld() -> Result<(), Box<dyn Error>> {
    let thread_time = Instant::now();
    let (top, _) = Population::<Hello, HelloEnv, World>::new()
        .size(100)
        .populate_base()
        .dynamic_distance(true)
        .configure(Config {
            inbreed_rate: 0.001,
            crossover_rate: 0.75,
            distance: 0.5,
            species_target: 5
        })
        .stagnation(10, vec![
            Genocide::KillWorst(0.9)
        ])
        .run(|model, fit, num| {
            println!("Generation: {} score: {:.3?}\t{:?}", num, fit, model.as_string());
            fit == 12.0 || num == 500
        })?;
        

    println!("\nTime in millis: {}, solution: {:?}", thread_time.elapsed().as_millis(), top.as_string());
    Ok(())
}


pub struct World {
    target: Vec<char>
}


impl World {
    pub fn new() -> Self {
        World {
            target: vec!['h', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd', '!']
        }
    }
}


impl Problem<Hello> for World {

    fn empty() -> Self { World::new() }

    fn solve(&self, model: &mut Hello) -> f32 {
        let mut total = 0.0;
        for (index, letter) in self.target.iter().enumerate() {
            if letter == &model.data[index] {
                total += 1.0;
            }
        }
        total        
    }
}


#[derive(Debug, Clone)]
pub struct HelloEnv {
    pub alph: Vec<char>,
}

impl HelloEnv {
    pub fn new() -> Self {
        HelloEnv {
            alph: vec!['!', ' ', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'],
        }
    }
}

impl Envionment for HelloEnv {}
impl Default for HelloEnv {
    fn default() -> Self {
        Self::new()
    }
}



#[derive(Debug, Clone, PartialEq)]
pub struct Hello {
    pub data: Vec<char>
}

impl Hello {
    pub fn new(alph: &[char]) -> Self {
        let mut r = rand::thread_rng();
        Hello { data: (0..12).map(|_| alph[r.gen_range(0, alph.len())]).collect() }
    }

    pub fn as_string(&self) -> String {
        self.data
            .iter()
            .map(|x| String::from(x.to_string()))
            .collect::<Vec<_>>()
            .join("")
    }
}



impl Genome<Hello, HelloEnv> for Hello {

    fn crossover(parent_one: &Hello, parent_two: &Hello, env: Arc<RwLock<HelloEnv>>, crossover_rate: f32) -> Option<Hello> {
        let params = env.read().unwrap();
        let mut r = rand::thread_rng();
        let mut new_data = Vec::new();
        
        if r.gen::<f32>() < crossover_rate {
            for (one, two) in parent_one.data.iter().zip(parent_two.data.iter()) {
                if one != two {
                    new_data.push(*one);
                } else {
                    new_data.push(*two);
                }
            }
        } else {
            new_data = parent_one.data.clone();
            let swap_index = r.gen_range(0, new_data.len());
            new_data[swap_index] = params.alph[r.gen_range(0, params.alph.len())];
        }
        Some(Hello { data: new_data })
    }


    fn distance(one: &Hello, two: &Hello, _: Arc<RwLock<HelloEnv>>) -> f32 {
        let mut total = 0_f32;
        for (i, j) in one.data.iter().zip(two.data.iter()) {
            if i == j {
                total += 1_f32;
            }
        }
        one.data.len() as f32 / total
    }

    fn base(env: &mut HelloEnv) -> Hello {
        Hello::new(&env.alph)
    }
}
