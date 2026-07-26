// Headless Flappy Bird simulation. No radiate or bevy dependencies here on
// purpose — this is pure game logic that both the fitness function and the
// (separate) renderer read from.
//
// This is the best way to go about these things in radiate. Create a shared 'blueprint' of the simulation that
// both radiate and your renderer/world/environment can use. This keep the simulation logic in one place and
// allows you to swap out the renderer or the simulation engine without having to rewrite the other. You'll see
// this same sort of logic in other examples (snake.py/.rs).

pub const WORLD_HALF_HEIGHT: f32 = 300.0;
pub const GROUND_Y: f32 = -WORLD_HALF_HEIGHT;
pub const CEILING_Y: f32 = WORLD_HALF_HEIGHT;

pub const BIRD_X: f32 = -150.0;
pub const BIRD_RADIUS: f32 = 12.0;

pub const PIPE_WIDTH: f32 = 60.0;
pub const PIPE_GAP: f32 = 150.0;
pub const PIPE_SPACING: f32 = 260.0;
pub const PIPE_SPEED: f32 = 180.0;
pub const FIRST_PIPE_X: f32 = 400.0;

pub const GRAVITY: f32 = -900.0;
pub const FLAP_VELOCITY: f32 = 320.0;
pub const MAX_FALL_SPEED: f32 = -600.0;
pub const DT: f32 = 1.0 / 60.0;

pub const PIPE_BONUS: f32 = 100.0;
pub const MAX_PIPES_DECAY: u32 = 60; // Every 15 `MAX_PIPES_DECAY` is equivalent to 10 game pipes (15 = 10 pipes, 30 = 20 pipes, 60 = 40 pipes, etc., etc.).
pub const MAX_GAME_SECONDS: u32 = 60; // 60 (game) seconds cap per generation 
// The max fitness score allowed before the fitness is terminated.
// This is to prevent the evolution from taking too long.
// The speed control makes this not real-time, but simulated seconds so evolution still happens very quick.
pub const MAX_TICKS: u32 = MAX_GAME_SECONDS * MAX_PIPES_DECAY;

#[derive(Clone, Copy)]
pub struct Bird {
    pub y: f32,
    pub vy: f32,
    pub alive: bool,
    pub ticks_alive: u32,
    pub pipes_passed: u32,
}

impl Bird {
    fn new() -> Self {
        Bird {
            y: 0.0,
            vy: 0.0,
            alive: true,
            ticks_alive: 0,
            pipes_passed: 0,
        }
    }

    pub fn fitness(&self) -> f32 {
        self.ticks_alive as f32 + self.pipes_passed as f32 * PIPE_BONUS
    }
}

#[derive(Clone, Copy)]
pub struct Pipe {
    pub x: f32,
    pub gap_top: f32,
    pub gap_bottom: f32,
    passed: bool,
}

pub struct World {
    pub birds: Vec<Bird>,
    pub pipes: Vec<Pipe>,
    pub tick: u32,
    rng_state: u64,
}

impl World {
    pub fn new(num_birds: usize, seed: u64) -> Self {
        let mut world = World {
            birds: vec![Bird::new(); num_birds],
            pipes: Vec::new(),
            tick: 0,
            rng_state: seed,
        };

        let mut x = FIRST_PIPE_X;
        for _ in 0..4 {
            world.spawn_pipe(x);
            x += PIPE_SPACING;
        }

        world
    }

    fn next_random(&mut self) -> f32 {
        // xorshift64 — deterministic, no external rand dependency needed here.
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        (x % 10_000) as f32 / 10_000.0
    }

    fn spawn_pipe(&mut self, x: f32) {
        let usable_half = WORLD_HALF_HEIGHT - PIPE_GAP / 2.0 - 20.0;
        let r = self.next_random();
        let center = (r * 2.0 - 1.0) * usable_half;
        self.pipes.push(Pipe {
            x,
            gap_top: center + PIPE_GAP / 2.0,
            gap_bottom: center - PIPE_GAP / 2.0,
            passed: false,
        });
    }

    pub fn all_dead(&self) -> bool {
        self.birds.iter().all(|b| !b.alive)
    }

    /// Normalized Graph input vector for bird `i`:
    /// [bird_y, bird_vy, dx_to_next_pipe, gap_top, gap_bottom]
    pub fn bird_inputs(&self, i: usize) -> [f32; 5] {
        let bird = &self.birds[i];
        let next_pipe = self
            .pipes
            .iter()
            .find(|p| p.x + PIPE_WIDTH / 2.0 >= BIRD_X)
            .unwrap_or(&self.pipes[0]);

        [
            bird.y / WORLD_HALF_HEIGHT,
            (bird.vy / -MAX_FALL_SPEED).clamp(-1.0, 1.0),
            ((next_pipe.x - BIRD_X) / PIPE_SPACING).clamp(-1.0, 2.0),
            next_pipe.gap_top / WORLD_HALF_HEIGHT,
            next_pipe.gap_bottom / WORLD_HALF_HEIGHT,
        ]
    }

    pub fn step(&mut self, flap_decisions: &[bool]) {
        self.tick += 1;

        for (bird, &flap) in self.birds.iter_mut().zip(flap_decisions) {
            if !bird.alive {
                continue;
            }

            if flap {
                bird.vy = FLAP_VELOCITY;
            } else {
                bird.vy = (bird.vy + GRAVITY * DT).max(MAX_FALL_SPEED);
            }
            bird.y += bird.vy * DT;

            let mut died = bird.y <= GROUND_Y || bird.y >= CEILING_Y;

            if !died {
                for pipe in &self.pipes {
                    let within_x = (BIRD_X + BIRD_RADIUS) >= (pipe.x - PIPE_WIDTH / 2.0)
                        && (BIRD_X - BIRD_RADIUS) <= (pipe.x + PIPE_WIDTH / 2.0);
                    if within_x && (bird.y >= pipe.gap_top || bird.y <= pipe.gap_bottom) {
                        died = true;
                        break;
                    }
                }
            }

            if died {
                bird.alive = false;
            } else {
                bird.ticks_alive += 1;
            }
        }

        for pipe in &mut self.pipes {
            pipe.x -= PIPE_SPEED * DT;
            if !pipe.passed && pipe.x + PIPE_WIDTH / 2.0 < BIRD_X {
                pipe.passed = true;
                for bird in self.birds.iter_mut().filter(|b| b.alive) {
                    bird.pipes_passed += 1;
                }
            }
        }

        if let Some(last) = self.pipes.last().map(|p| p.x) {
            if last < FIRST_PIPE_X + PIPE_SPACING * 3.0 {
                self.spawn_pipe(last + PIPE_SPACING);
            }
        }

        self.pipes.retain(|p| p.x + PIPE_WIDTH > BIRD_X - 400.0);
    }
}
