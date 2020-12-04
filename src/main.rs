use std::cmp::Ordering;

use all_pong_my_own::random_polar;
use bevy::{
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};

/// Pong, but on your own.
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_event::<ScoreEvent>()
        .add_event::<GameOverEvent>()
        .add_resource(PlayState::Playing)
        .add_resource(Scoreboard { score: 0 })
        .add_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        .add_startup_system(setup)
        .add_system(playing.chain(paddle_movement_system))
        .add_system(playing.chain(ball_collision_system))
        .add_system(playing.chain(ball_movement_system))
        .add_system(playing.chain(handle_score_event))
        .add_system(playing.chain(handle_game_over_event))
        .add_system(won.chain(animate_won_message))
        .add_system(lost.chain(animate_lost_message))
        .run();
}

struct Paddle {
    speed: f32,
}

struct Ball {
    velocity: Vec3,
}

enum Collider {
    Solid,
    Scorable,
    Paddle,
}

struct Scoreboard {
    score: isize,
}

struct ScoreEvent(isize);

struct ScoreboardMessage;

enum GameOverEvent {
    Win,
    Lose,
}

struct GameOverMessage;

enum PlayState {
    Playing,
    Won,
    Lost,
}

fn setup(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Add the game's entities to our world
    commands
        // cameras
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default())
        // paddle bottom
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, -215.0, 0.0)),
            sprite: Sprite::new(Vec2::new(120.0, 30.0)),
            ..Default::default()
        })
        .with(Paddle { speed: 500.0 })
        .with(Collider::Paddle)
        // paddle top
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 215.0, 0.0)),
            sprite: Sprite::new(Vec2::new(120.0, 30.0)),
            ..Default::default()
        })
        .with(Paddle { speed: 500.0 })
        .with(Collider::Paddle)
        // ball
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
            transform: Transform::from_translation(Vec3::new(0.0, -50.0, 1.0)),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            ..Default::default()
        })
        .with(Ball {
            velocity: 600.0 * Vec3::new(0.5, -0.5, 0.0).normalize(),
        })
        // scoreboard
        .spawn(TextBundle {
            text: Text {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                value: "Score:".to_string(),
                style: TextStyle {
                    color: Color::rgb(0.2, 0.2, 0.2),
                    font_size: 40.0,
                    ..Default::default()
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(ScoreboardMessage)
        // game over
        .spawn(TextBundle {
            text: Text {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                value: "".to_string(),
                style: TextStyle {
                    color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                    alignment: TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        ..Default::default()
                    },
                    font_size: 40.0,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    right: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(GameOverMessage);

    // Add walls
    let wall_material = materials.add(Color::rgb(0.8, 0.8, 0.8).into());
    let wall_material2 = materials.add(Color::rgb(1.0, 0.5, 0.5).into());
    let wall_thickness = 10.0;
    let bounds = Vec2::new(900.0, 600.0);

    commands
        // left
        .spawn(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_translation(Vec3::new(-bounds.x / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y + wall_thickness)),
            ..Default::default()
        })
        .with(Collider::Solid)
        // right
        .spawn(SpriteBundle {
            material: wall_material,
            transform: Transform::from_translation(Vec3::new(bounds.x / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y + wall_thickness)),
            ..Default::default()
        })
        .with(Collider::Solid)
        // bottom
        .spawn(SpriteBundle {
            material: wall_material2.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, -bounds.y / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(bounds.x + wall_thickness, wall_thickness)),
            ..Default::default()
        })
        .with(Collider::Scorable)
        // top
        .spawn(SpriteBundle {
            material: wall_material2,
            transform: Transform::from_translation(Vec3::new(0.0, bounds.y / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(bounds.x + wall_thickness, wall_thickness)),
            ..Default::default()
        })
        .with(Collider::Scorable);
}

fn paddle_movement_system(
    In(playing): In<bool>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Paddle, &mut Transform)>,
) {
    if !playing {
        return;
    }
    for (paddle, mut transform) in query.iter_mut() {
        let mut direction = 0.0;
        if keyboard_input.pressed(KeyCode::Left) {
            direction -= 1.0;
        }

        if keyboard_input.pressed(KeyCode::Right) {
            direction += 1.0;
        }

        let translation = &mut transform.translation;
        // move the paddle horizontally
        translation.x += time.delta_seconds() * direction * paddle.speed;
        // bound the paddle within the walls
        translation.x = translation.x.min(380.0).max(-380.0);
    }
}

fn ball_movement_system(
    In(playing): In<bool>,
    time: Res<Time>,
    mut ball_query: Query<(&Ball, &mut Transform)>,
) {
    if !playing {
        return;
    }
    // clamp the timestep to stop the ball from escaping when the game starts
    let delta_seconds = f32::min(0.2, time.delta_seconds());

    for (ball, mut transform) in ball_query.iter_mut() {
        transform.translation += ball.velocity * delta_seconds;
    }
}

fn ball_collision_system(
    In(playing): In<bool>,
    mut ball_query: Query<(&mut Ball, &Transform, &Sprite)>,
    collider_query: Query<(&Collider, &Transform, &Sprite)>,
    mut score_events: ResMut<Events<ScoreEvent>>,
) {
    if !playing {
        return;
    }
    for (mut ball, ball_transform, sprite) in ball_query.iter_mut() {
        let ball_size = sprite.size;
        let velocity = &mut ball.velocity;

        // check collision with walls
        for (collider, transform, sprite) in collider_query.iter() {
            let collision = collide(
                ball_transform.translation,
                ball_size,
                transform.translation,
                sprite.size,
            );
            if let Some(collision) = collision {
                // reflect the ball when it collides
                let mut reflect_x = false;
                let mut reflect_y = false;

                // only reflect if the ball's velocity is going in the opposite direction of the collision
                match collision {
                    Collision::Left => reflect_x = velocity.x > 0.0,
                    Collision::Right => reflect_x = velocity.x < 0.0,
                    Collision::Top => reflect_y = velocity.y < 0.0,
                    Collision::Bottom => reflect_y = velocity.y > 0.0,
                }

                // reflect velocity on the x-axis if we hit something on the x-axis
                if reflect_x {
                    velocity.x = -velocity.x;
                }

                // reflect velocity on the y-axis if we hit something on the y-axis
                if reflect_y {
                    velocity.y = -velocity.y;
                }

                // break if this collide is on a solid, otherwise continue check whether a solid is also in collision
                match *collider {
                    Collider::Solid => {
                        break;
                    }
                    Collider::Scorable => {
                        score_events.send(ScoreEvent(-1));
                    }
                    Collider::Paddle => {
                        score_events.send(ScoreEvent(1));
                    }
                }
            }
        }
    }
}

fn handle_score_event(
    In(playing): In<bool>,
    mut event_reader: Local<EventReader<ScoreEvent>>,
    events: Res<Events<ScoreEvent>>,
    mut scoreboard: ResMut<Scoreboard>,
    mut ball_query: Query<(&mut Ball, &mut Transform)>,
    mut game_over_events: ResMut<Events<GameOverEvent>>,
    mut text_query: Query<&mut Text, With<ScoreboardMessage>>,
) {
    if !playing {
        return;
    }
    for event in event_reader.iter(&events) {
        scoreboard.score += event.0;
        for mut text in text_query.iter_mut() {
            text.value = format!("Score: {}", scoreboard.score);
            text.style.color = match scoreboard.score.cmp(&0) {
                Ordering::Greater => Color::rgb(0.5, 0.5, 1.0),
                Ordering::Less => Color::rgb(1.0, 0.5, 0.5),
                Ordering::Equal => Color::rgb(0.2, 0.2, 0.2),
            }
        }
        if scoreboard.score >= 20 {
            game_over_events.send(GameOverEvent::Win);
            return;
        }
        if scoreboard.score <= -20 {
            game_over_events.send(GameOverEvent::Lose);
            return;
        }
        for (mut ball, mut transform) in ball_query.iter_mut() {
            if event.0 < 0 {
                transform.translation = random_polar(0.0, 100.0, 0.0, 1.0);
            }
            let (min, max) = if ball.velocity.y > 0.0 {
                (0.1, 0.4)
            } else {
                (0.6, 0.9)
            };
            ball.velocity = random_polar(400.0, 1000.0, min, max);
        }
    }
}

fn handle_game_over_event(
    In(playing): In<bool>,
    mut event_reader: Local<EventReader<GameOverEvent>>,
    events: Res<Events<GameOverEvent>>,
    mut play_state: ResMut<PlayState>,
    mut query: Query<&mut Text, With<GameOverMessage>>,
) {
    if !playing {
        return;
    }
    for event in event_reader.iter(&events) {
        for mut text in query.iter_mut() {
            text.value = match event {
                GameOverEvent::Win => "YOU WIN!".into(),
                GameOverEvent::Lose => "You lose,\nLoser.".into(),
            };
        }
        *play_state = match event {
            GameOverEvent::Win => PlayState::Won,
            GameOverEvent::Lose => PlayState::Lost,
        };
    }
}

fn playing(play_state: Res<PlayState>) -> bool {
    matches!(*play_state, PlayState::Playing)
}

fn won(play_state: Res<PlayState>) -> bool {
    matches!(*play_state, PlayState::Won)
}

fn lost(play_state: Res<PlayState>) -> bool {
    matches!(*play_state, PlayState::Lost)
}

fn animate_won_message(
    In(won): In<bool>,
    time: Res<Time>,
    mut prog: Local<f32>,
    mut query: Query<(&mut Text, &mut Style), With<GameOverMessage>>,
) {
    if !won {
        return;
    }
    const DURATION: f32 = 2.0;
    *prog += time.delta_seconds();
    if *prog >= DURATION {
        *prog -= DURATION;
    }
    let prog = *prog / DURATION;
    for (mut text, mut _style) in query.iter_mut() {
        if (0.0..0.5).contains(&prog) {
            text.style.color = Color::rgb(0.1 + 0.5 * (0.5 - prog), 0.1 + 0.5 * (0.5 - prog), 1.0);
            text.style.font_size = 40.0 + 40.0 * (prog - 0.5);
        } else if (0.5..1.0).contains(&prog) {
            text.style.color = Color::rgb(0.1 + 0.5 * (prog - 0.5), 0.1 + 0.5 * (prog - 0.5), 1.0);
            text.style.font_size = 40.0 + 40.0 * (0.5 - prog);
        }
    }
}

fn animate_lost_message(
    In(lost): In<bool>,
    time: Res<Time>,
    mut prog: Local<f32>,
    mut query: Query<(&mut Text, &mut Style), With<GameOverMessage>>,
) {
    if !lost {
        return;
    }
    const DURATION: f32 = 2.0;
    *prog += time.delta_seconds();
    if *prog >= DURATION {
        *prog -= DURATION;
    }
    let prog = *prog / DURATION;
    for (mut text, mut _style) in query.iter_mut() {
        if (0.0..0.5).contains(&prog) {
            text.style.color = Color::rgb(1.0, 0.1 + 0.5 * (0.5 - prog), 0.1 + 0.5 * (0.5 - prog));
            text.style.font_size = 40.0 + 40.0 * (prog - 0.5);
        } else if (0.5..1.0).contains(&prog) {
            text.style.color = Color::rgb(1.0, 0.1 + 0.5 * (prog - 0.5), 0.1 + 0.5 * (prog - 0.5));
            text.style.font_size = 40.0 + 40.0 * (0.5 - prog);
        }
    }
}
