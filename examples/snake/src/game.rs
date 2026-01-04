use crate::{FOOD_POSITIONS, INPUT_SIZE, MAX_STEPS};

#[derive(Clone)]
pub struct SnakeGame {
    pub width: i32,
    pub height: i32,
    pub snake: Vec<(i32, i32)>,
    pub direction: (i32, i32),
    pub food: (i32, i32),
    pub food_index: usize,
    pub score: i32,
    pub steps: usize,
    pub game_over: bool,
}

impl SnakeGame {
    pub fn new(width: i32, height: i32) -> Self {
        let game = SnakeGame {
            width,
            height,
            snake: vec![(width / 2, height / 2)],
            direction: (1, 0),
            food: FOOD_POSITIONS[0],
            food_index: 0,
            score: 0,
            steps: 0,
            game_over: false,
        };

        game
    }

    pub fn get_state(&self) -> [f32; INPUT_SIZE] {
        // - 4 dangers (up/right/down/left)
        // - 2 food direction
        // - 4 one-hot for current direction
        // - 3 game stats (score, steps, snake length)

        let (head_x, head_y) = self.snake[0];

        let directions = [(0, -1), (1, 0), (0, 1), (-1, 0)]; // Up, Right, Down, Left

        let mut state_vec = Vec::with_capacity(INPUT_SIZE);

        // Danger detection in 4 directions
        for (dx, dy) in directions.iter().copied() {
            let mut distance = 1_i32;

            let mut nx = head_x + dx;
            let mut ny = head_y + dy;

            if nx < 0
                || nx >= self.width
                || ny < 0
                || ny >= self.height
                || self.snake.contains(&(nx, ny))
            {
                state_vec.push(1.0);
            } else {
                while nx >= 0
                    && nx < self.width
                    && ny >= 0
                    && ny < self.height
                    && !self.snake.contains(&(nx, ny))
                {
                    nx += dx;
                    ny += dy;
                    distance += 1;
                }
                state_vec.push(1.0 / (distance as f32));
            }
        }

        // Food direction
        let food_dx = self.food.0 - head_x;
        let food_dy = self.food.1 - head_y;
        let food_distance = i32::max(food_dx.abs(), food_dy.abs());

        if food_distance > 0 {
            state_vec.push(food_dx as f32 / food_distance as f32);
            state_vec.push(food_dy as f32 / food_distance as f32);
        } else {
            state_vec.push(0.0);
            state_vec.push(0.0);
        }

        // Current direction one-hot
        let current_dir_idx = directions
            .iter()
            .position(|&d| d == self.direction)
            .unwrap_or(1);

        for i in 0..4 {
            state_vec.push(if i == current_dir_idx { 1.0 } else { 0.0 });
        }

        // Game stats
        state_vec.push(self.score as f32 / 10.0);
        state_vec.push(self.steps as f32 / MAX_STEPS as f32);
        state_vec.push(self.snake.len() as f32 / (self.width * self.height) as f32);

        debug_assert_eq!(state_vec.len(), INPUT_SIZE);
        state_vec.try_into().unwrap()
    }

    pub fn step(&mut self, action: usize) -> bool {
        if self.game_over {
            return false;
        }

        let directions = [(0, -1), (1, 0), (0, 1), (-1, 0)];
        let new_direction = directions[action];

        // no 180° turn
        if new_direction != (-self.direction.0, -self.direction.1) {
            self.direction = new_direction;
        }

        let (hx, hy) = self.snake[0];
        let new_x = hx + self.direction.0;
        let new_y = hy + self.direction.1;

        if new_x < 0 || new_x >= self.width || new_y < 0 || new_y >= self.height {
            self.game_over = true;
            return false;
        }

        let new_head = (new_x, new_y);

        if self.snake.contains(&new_head) {
            self.game_over = true;
            return false;
        }

        self.snake.insert(0, new_head);

        if new_head == self.food {
            self.score += 1;
            self.food_index += 1;
            if self.food_index < FOOD_POSITIONS.len() {
                self.food = FOOD_POSITIONS[self.food_index];
            } else {
                self.game_over = true;
                return false;
            }
        } else {
            self.snake.pop();
        }

        self.steps += 1;
        if self.steps >= MAX_STEPS {
            self.game_over = true;
            return false;
        }

        true
    }
}
