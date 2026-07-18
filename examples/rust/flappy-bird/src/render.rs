use crate::game;
use crate::speed::SimSpeed;
use crate::swarm::Snapshot;
use bevy::prelude::*;
use bevy::text::FontSize;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;

/// Owns the receiving end of the worker thread's snapshot channel. Wrapped
/// in a `Mutex` (rather than `NonSend`) so the drain system can be scheduled
/// like any other resource-reading system.
#[derive(Resource)]
pub struct SnapshotReceiver(pub Mutex<Receiver<Snapshot>>);

/// The Bevy-side handle to the speed dial shared with the worker thread.
#[derive(Resource)]
pub struct SimSpeedRes(pub SimSpeed);

/// Marks every entity spawned to represent the current simulation frame
/// (birds + pipes) so it can be cleared wholesale before the next one is
/// drawn — simplest possible sync strategy, and cheap at this entity count.
#[derive(Component)]
pub(crate) struct SimEntity;

#[derive(Component)]
pub(crate) struct GenerationText;

#[derive(Component)]
pub(crate) struct SpeedText;

const BIRD_COLOR_ALIVE: Color = Color::srgb(1.0, 0.85, 0.2);
const BIRD_COLOR_DEAD: Color = Color::srgba(0.5, 0.5, 0.5, 0.25);
const PIPE_COLOR: Color = Color::srgb(0.2, 0.7, 0.3);
const BACKGROUND_COLOR: Color = Color::srgb(0.53, 0.81, 0.92);

pub fn setup(mut commands: Commands, speed: Res<SimSpeedRes>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Text::new("Generation 0"),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::BLACK),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        GenerationText,
    ));

    commands.spawn((
        Text::new(speed_label(&speed.0)),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::BLACK),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        },
        SpeedText,
    ));
}

fn speed_label(speed: &SimSpeed) -> String {
    format!("Speed: {}   [ - slower  /  = faster ]", speed.label())
}

pub fn handle_speed_input(
    keys: Res<ButtonInput<KeyCode>>,
    speed: Res<SimSpeedRes>,
    mut text_query: Query<&mut Text, With<SpeedText>>,
) {
    let mut changed = false;
    if keys.just_pressed(KeyCode::Minus) {
        speed.0.slower();
        changed = true;
    }
    if keys.just_pressed(KeyCode::Equal) {
        speed.0.faster();
        changed = true;
    }

    if changed {
        if let Ok(mut text) = text_query.single_mut() {
            text.0 = speed_label(&speed.0);
        }
    }
}

pub fn sync_snapshot_to_entities(
    receiver: Res<SnapshotReceiver>,
    mut commands: Commands,
    sim_entities: Query<Entity, With<SimEntity>>,
    mut text_query: Query<&mut Text, With<GenerationText>>,
) {
    let snapshot = {
        let rx = receiver.0.lock().unwrap();
        let mut latest = None;
        while let Ok(s) = rx.try_recv() {
            latest = Some(s);
        }
        latest
    };

    let Some(snapshot) = snapshot else {
        return;
    };

    for entity in &sim_entities {
        commands.entity(entity).despawn();
    }

    for &(y, alive) in &snapshot.birds {
        let color = if alive { BIRD_COLOR_ALIVE } else { BIRD_COLOR_DEAD };
        commands.spawn((
            Sprite::from_color(color, Vec2::splat(16.0)),
            Transform::from_xyz(game::BIRD_X, y, 1.0),
            SimEntity,
        ));
    }

    for &(x, gap_top, gap_bottom) in &snapshot.pipes {
        let top_height = (game::WORLD_HALF_HEIGHT - gap_top).max(0.0);
        let top_center_y = gap_top + top_height / 2.0;
        commands.spawn((
            Sprite::from_color(PIPE_COLOR, Vec2::new(game::PIPE_WIDTH, top_height)),
            Transform::from_xyz(x, top_center_y, 0.5),
            SimEntity,
        ));

        let bottom_height = (gap_bottom + game::WORLD_HALF_HEIGHT).max(0.0);
        let bottom_center_y = -game::WORLD_HALF_HEIGHT + bottom_height / 2.0;
        commands.spawn((
            Sprite::from_color(PIPE_COLOR, Vec2::new(game::PIPE_WIDTH, bottom_height)),
            Transform::from_xyz(x, bottom_center_y, 0.5),
            SimEntity,
        ));
    }

    let alive = snapshot.birds.iter().filter(|(_, a)| *a).count();
    if let Ok(mut text) = text_query.single_mut() {
        text.0 = match snapshot.best_score {
            Some(best_score) => format!(
                "Final replay — best genome (score {:.0})  |  tick {}",
                best_score, snapshot.tick
            ),
            None => format!(
                "Generation {}  |  alive {}/{}  |  tick {}",
                snapshot.generation,
                alive,
                snapshot.birds.len(),
                snapshot.tick
            ),
        };
    }
}

pub fn background_color() -> ClearColor {
    ClearColor(BACKGROUND_COLOR)
}
