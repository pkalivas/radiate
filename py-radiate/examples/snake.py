#!/usr/bin/env python3
"""
Snake AI Evolution - Debug Version

This version includes extensive logging to understand why
the best solution behaves differently after evolution.
"""

import radiate as rd
import numpy as np
import matplotlib.pyplot as plt  # type: ignore
from matplotlib.animation import FuncAnimation  # type: ignore

rd.random.seed(42)
np.random.seed(42)

HEIGHT = 15
WIDTH = 15
MAX_STEPS = 1500
FOOD_POSITIONS = [
    (rd.random.int(0, WIDTH), rd.random.int(0, HEIGHT)) for _ in range(MAX_STEPS * 2)
]
NUM_GAMES = 5
GAMMA = 0.99  # discount factor for per-step rewards
K_POTENTIAL = 2.0  # weight for distance-based shaping (per step)
STEP_PENALTY = -0.02  # mild time penalty to discourage stalling
JITTER_PENALTY = -0.05  # applied when direction changes from previous step
LOOP_WINDOW = 12  # size of recent positions window for loop detection
LOOP_PENALTY = -0.15  # penalty when revisiting within recent window
EXPLORATION_BONUS = 0.2  # per unique cell visited
EAT_REWARD = 200.0  # big hit when food eaten (per event)
SCORE_TERMINAL = 400.0  # terminal bonus * (score^2) to push reliable eating
EARLY_DEATH_PENALTY = -120.0  # if die < min steps and score == 0
MIN_SURVIVAL_STEPS = 50


class SnakeGame:
    """Classic Snake game with detailed logging."""

    def __init__(self, width: int = 20, height: int = 20, debug: bool = False):
        self.width = width
        self.height = height
        self.debug = debug
        self.food_index = 0
        self.reset()

    def reset(self):
        """Reset the game to initial state."""
        self.snake = [(self.width // 2, self.height // 2)]
        self.direction = (1, 0)  # Start moving right
        self.food = self.generate_food()
        self.score = 0
        self.steps = 0
        self.max_steps = 500
        self.game_over = False
        self.food_eaten = 0
        self.steps_without_food = 0

        if self.debug:
            print(f"Game reset: snake={self.snake}, food={self.food}")

    def generate_food(self) -> tuple[int, int]:
        """Generate food at random position."""
        attempts = 0
        while attempts < 100:
            food = FOOD_POSITIONS[self.food_index % len(FOOD_POSITIONS)]
            self.food_index += 1
            if food not in self.snake:
                return food
            attempts += 1

        # Fallback
        for x in range(self.width):
            for y in range(self.height):
                if (x, y) not in self.snake:
                    return (x, y)
        return (0, 0)

    def get_state(self) -> list[float]:
        """Get current game state as neural network input."""
        head_x, head_y = self.snake[0]

        # Direction vectors
        directions = [(0, -1), (1, 0), (0, 1), (-1, 0)]  # Up, Right, Down, Left

        state = []

        # Danger detection in all 4 directions
        for dx, dy in directions:
            danger = 0.0
            distance = 1

            next_x, next_y = head_x + dx, head_y + dy
            if (
                next_x < 0
                or next_x >= self.width
                or next_y < 0
                or next_y >= self.height
                or (next_x, next_y) in self.snake
            ):
                danger = 1.0
            else:
                while (
                    0 <= next_x < self.width
                    and 0 <= next_y < self.height
                    and (next_x, next_y) not in self.snake
                ):
                    next_x += dx
                    next_y += dy
                    distance += 1
                danger = 1.0 / distance

            state.append(danger)

        # Food direction
        food_dx = self.food[0] - head_x
        food_dy = self.food[1] - head_y
        food_distance = max(abs(food_dx), abs(food_dy))

        if food_distance > 0:
            state.extend([food_dx / food_distance, food_dy / food_distance])
        else:
            state.extend([0.0, 0.0])

        # Current direction
        current_dir_idx = directions.index(self.direction)
        for i in range(4):
            state.append(1.0 if i == current_dir_idx else 0.0)

        # Game statistics
        state.extend(
            [
                self.score / 10.0,
                self.steps / self.max_steps,
                len(self.snake) / (self.width * self.height),
            ]
        )

        return state

    def step(self, action: int) -> bool:
        """Execute one game step based on AI decision."""
        if self.game_over:
            return False

        directions = [(0, -1), (1, 0), (0, 1), (-1, 0)]
        new_direction = directions[action]

        # Prevent 180-degree turns
        if new_direction != (-self.direction[0], -self.direction[1]):
            self.direction = new_direction

        head_x, head_y = self.snake[0]
        new_head = (head_x + self.direction[0], head_y + self.direction[1])

        if (
            new_head[0] < 0
            or new_head[0] >= self.width
            or new_head[1] < 0
            or new_head[1] >= self.height
            or new_head in self.snake
        ):
            self.game_over = True
            return False

        # Add new head
        self.snake.insert(0, new_head)

        # Check for food
        if new_head == self.food:
            self.score += 1
            self.food_eaten += 1
            self.steps_without_food = 0
            self.food = self.generate_food()
        else:
            self.snake.pop()
            self.steps_without_food += 1

        self.steps += 1

        # Game over if too many steps or too long without food
        if (
            self.steps >= self.max_steps and not self.debug
        ) or self.steps_without_food >= 100:
            self.game_over = True
            return False

        return True

    def get_fitness(self) -> float:
        """Calculate fitness score for the current game state."""
        score_fitness = self.score * 100.0
        survival_bonus = self.steps * 0.1
        efficiency_bonus = 0.0
        if self.steps > 0:
            efficiency_bonus = (self.score / self.steps) * 50.0

        # Distance to food bonus
        head_x, head_y = self.snake[0]
        food_distance = abs(self.food[0] - head_x) + abs(self.food[1] - head_y)
        distance_bonus = max(0, 10.0 - food_distance)

        total_fitness = (
            score_fitness + survival_bonus + efficiency_bonus + distance_bonus
        )

        if self.steps > 0:
            total_fitness = max(total_fitness, 1.0)

        return total_fitness


class SnakeAI:
    """Neural network AI for Snake game."""

    def __init__(self, graph: rd.Graph):
        self.graph = graph

    def predict(self, state: list[float]) -> int:
        """Predict the best action given current state."""
        output = self.graph.eval([state])
        return np.argmax(output[0])


class SnakeEvolver:
    def __init__(self):
        self.input_size = 13
        self.output_size = 4

    @staticmethod
    def fitness_function(graph: rd.Graph) -> float:
        """
        Dense, shaped fitness:
        - Potential-based reward on Manhattan distance to food (policy-invariant shaping with discount)
        - Small per-step penalty to avoid stalling
        - Exploration bonus (unique cells)
        - Anti-jitter (too many direction changes)
        - Loop penalty (revisiting recent positions)
        - Strong terminal reward for eating; early-death penalty
        """

        total_fitness = 0.0

        game = SnakeGame(debug=False)
        ai = SnakeAI(graph)

        # Tracking
        unique_cells = set([game.snake[0]])
        recent_positions = [game.snake[0]]
        last_direction = game.direction
        discounted_return = 0.0
        discount = 1.0
        last_score = 0

        while not game.game_over:
            state = game.get_state()
            action = ai.predict(state)

            # Distances before move
            hx, hy = game.snake[0]
            old_dist = abs(game.food[0] - hx) + abs(game.food[1] - hy)

            # Take step
            alive = game.step(action)

            # Prepare step reward
            step_reward = 0.0

            if not alive:
                # Early death penalty if no food and died quickly
                if game.steps < MIN_SURVIVAL_STEPS and game.score == 0:
                    step_reward += EARLY_DEATH_PENALTY
                discounted_return += discount * step_reward
                break

            # Distances after move
            nhx, nhy = game.snake[0]
            new_dist = abs(game.food[0] - nhx) + abs(game.food[1] - nhy)

            step_reward += K_POTENTIAL * (old_dist - new_dist)
            step_reward += STEP_PENALTY

            # Jitter penalty for frequent direction changes (avoid “vibrating”)
            if game.direction != last_direction:
                step_reward += JITTER_PENALTY
            last_direction = game.direction

            # Exploration bonus (credit new unique cells)
            if (nhx, nhy) not in unique_cells:
                unique_cells.add((nhx, nhy))
                step_reward += EXPLORATION_BONUS

            # Loop penalty (recent positions window)
            if (nhx, nhy) in recent_positions:
                step_reward += LOOP_PENALTY
            recent_positions.append((nhx, nhy))
            if len(recent_positions) > LOOP_WINDOW:
                recent_positions.pop(0)

            # Eating bonus when score increases
            if game.score > last_score:
                step_reward += EAT_REWARD + 0.5 * len(game.snake)
                last_score = game.score

            # Discounted accumulation
            discounted_return += discount * step_reward
            discount *= GAMMA

        # Terminal score bonus to strongly prefer consistent eaters
        discounted_return += (game.score**2) * SCORE_TERMINAL

        total_fitness += max(discounted_return, 1.0)  # keep positive floor

        return total_fitness / NUM_GAMES

    def test_individual(self, graph: rd.Graph, debug: bool = False) -> dict:
        """Test an individual with detailed logging."""

        game = SnakeGame(debug=debug)
        ai = SnakeAI(graph)

        game_history = []
        while not game.game_over:
            state = game.get_state()
            action = ai.predict(state)

            game_history.append(
                {
                    "snake": game.snake.copy(),
                    "food": game.food,
                    "score": game.score,
                    "steps": game.steps,
                    "action": action,
                    "state": state.copy(),
                }
            )

            game.step(action)

        return {
            "final_score": game.score,
            "final_steps": game.steps,
            "fitness": game.get_fitness(),
            "history": game_history,
        }

    def run_evolution(self, generations: int) -> rd.Generation:
        """Run the evolution process."""
        codec = rd.GraphCodec.weighted_directed(
            shape=(self.input_size, self.output_size),
            vertex=[
                rd.Op.sub(),
                rd.Op.mul(),
                rd.Op.linear(),
                rd.Op.sigmoid(),
                rd.Op.relu(),
                rd.Op.tanh(),
            ],
            edge=rd.Op.weight(),
            output=rd.Op.sigmoid(),
        )

        engine = rd.GeneticEngine(
            codec,
            SnakeEvolver.fitness_function,
            offspring_selector=rd.TournamentSelector(4),
            executor=rd.Executor.WorkerPool(),
            alters=[
                rd.GraphCrossover(0.5, 0.5),
                rd.OperationMutator(0.04, 0.05),
                rd.GraphMutator(0.08, 0.04, True),
            ],
        )

        return engine.run(
            [rd.GenerationsLimit(generations), rd.SecondsLimit(60 * 2)], log=True
        )

    def visualize_best_snake(self, graph: rd.Graph, title: str = "Best Snake AI"):
        """Visualize the best evolved snake playing."""
        test_result = self.test_individual(graph, debug=True)
        fig, ax = plt.subplots(figsize=(10, 8))

        def animate(frame):
            ax.clear()

            if frame < len(test_result["history"]):
                state = test_result["history"][frame]

                # Draw snake
                snake_x = [pos[0] for pos in state["snake"]]
                snake_y = [pos[1] for pos in state["snake"]]
                ax.plot(
                    snake_x, snake_y, "o-", color="green", linewidth=2, markersize=8
                )

                # Draw head
                ax.plot(snake_x[0], snake_y[0], "o", color="darkgreen", markersize=12)

                # Draw food
                ax.plot(
                    state["food"][0], state["food"][1], "s", color="red", markersize=10
                )

                ax.set_xlim(-1, 20)
                ax.set_ylim(-1, 20)
                ax.set_aspect("equal")
                ax.grid(True, alpha=0.3)
                ax.set_title(
                    f"{title}\nScore: {state['score']} | Steps: {state['steps']} | Action: {state['action']}"
                )

            return (ax,)

        anim = FuncAnimation(
            fig, animate, frames=len(test_result["history"]), interval=75, repeat=False
        )
        plt.show()

        return anim


def main():
    """Main function to run Snake AI evolution."""
    evolver = SnakeEvolver()

    # Run evolution
    generation = evolver.run_evolution(generations=250)

    print(generation)

    best_graph = generation.value()
    evolver.visualize_best_snake(best_graph, "Best Evolved Snake AI")


if __name__ == "__main__":
    main()
