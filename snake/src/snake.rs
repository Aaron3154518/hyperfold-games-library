use std::collections::VecDeque;

use hyperfold_engine::{
    _engine::Entity,
    add_components, components,
    ecs::{entities::NewEntity, events::core::Update},
    framework::{
        event_system::events::Key,
        physics::{BoundaryCollision, HitBox, PhysicsData, Position},
        render_system::{
            render_data::{Animation, RenderAsset, RenderDataBuilderTrait},
            AssetManager, Camera, Elevation, RenderComponent, Renderer,
        },
    },
    sdl2::SDL_KeyCode,
    utils::{
        rect::{Align, PointF, Rect},
        timer::{Timer, TimerTrait},
    },
};

use crate::{
    _engine::{Components, Events},
    elevations::Elevations,
    fruit::SpawnFruit,
    snake_body::{SnakeBody, SnakeBodyAnim, SnakeBodyPos, SNAKE_HB_W, SNAKE_W},
    GameOver, Playing, W_F,
};

#[derive(Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn velocity(&self, speed: f32) -> PointF {
        let (x, y) = match self {
            Direction::Up => (0.0, -speed),
            Direction::Down => (0.0, speed),
            Direction::Left => (-speed, 0.0),
            Direction::Right => (speed, 0.0),
        };
        PointF { x, y }
    }

    pub fn rotation(&self, base_angle: f64) -> f64 {
        -base_angle
            + match self {
                Direction::Up => 90.0,
                Direction::Down => 270.0,
                Direction::Left => 0.0,
                Direction::Right => 180.0,
            }
    }
}

#[hyperfold_engine::component(Singleton)]
struct Snake {
    pub body_count: usize,
    pub pivot_offset: usize,
    pub pivots: VecDeque<(PointF, Direction)>,
}

#[hyperfold_engine::system]
fn new_snake(
    _: &Playing::OnEnter,
    entities: &mut dyn Components,
    events: &mut dyn Events,
    r: &Renderer,
    am: &mut AssetManager,
    camera: &mut Camera,
) {
    camera.0.set_pos(0.0, 0.0, Align::Center, Align::Center);

    let e = Entity::new();
    let anim = Animation::new(8, 150);
    add_components!(
        entities,
        e,
        Playing::Label,
        Snake {
            body_count: 0,
            pivot_offset: 0,
            pivots: VecDeque::new()
        },
        Elevation(Elevations::Snake as u8),
        RenderComponent::new(
            RenderAsset::from_file("res/snake/snake_ss.png", r, am).with_animation(anim)
        ),
        HitBox(Rect::from_center(0.0, 0.0, SNAKE_HB_W, SNAKE_HB_W)),
        Position(Rect::from_center(0.0, 0.0, SNAKE_W, SNAKE_W)),
        PhysicsData {
            v: PointF::new(),
            a: PointF::new(),
            boundary: Some(Rect::from(0.0, 0.0, W_F, W_F, Align::Center, Align::Center))
        },
        Speed(100.0),
        anim
    );

    // Snake body animator
    let e = Entity::new();
    add_components!(
        entities,
        e,
        Playing::Label,
        SnakeBodyAnim {
            timer: Timer::new(150),
            frame: 0,
        }
    );

    events.new_event(SpawnFruit);
}

#[hyperfold_engine::component]
struct Speed(pub f32);

components!(
    labels(Snake),
    SnakePos,
    pos: &'a Position,
    hit_box: &'a HitBox,
);

components!(SnakePivots, pivots: &'a Snake, speed: &'a Speed);
components!(SnakePivotsMut, pivots: &'a mut Snake, speed: &'a Speed);

components!(
    SnakePhysics,
    pos: &'a Position,
    physics: &'a mut PhysicsData,
    speed: &'a Speed,
    tex: &'a mut RenderComponent,
    snake: &'a mut Snake,
    body: Option<&'a SnakeBody>,
);

#[hyperfold_engine::system]
fn move_snake(key: &Key, snake: SnakePhysics, entities: &mut dyn Components) {
    if key.0.pressed() {
        let direction = match key.0.key {
            SDL_KeyCode::SDLK_a => Direction::Left,
            SDL_KeyCode::SDLK_d => Direction::Right,
            SDL_KeyCode::SDLK_w => Direction::Up,
            SDL_KeyCode::SDLK_s => Direction::Down,
            _ => return,
        };

        // First pivot
        if snake.body.is_none() {
            entities.add_component(
                *snake.eid,
                SnakeBody {
                    direction,
                    snake_idx: snake.snake.body_count,
                    pivot_idx: (snake.snake.pivot_offset + snake.snake.pivots.len()).max(1) - 1,
                },
            );
            snake.snake.body_count += 1;
        }

        snake
            .snake
            .pivots
            .push_back((snake.pos.0.center(), direction));
    }
}

#[hyperfold_engine::system]
fn collide_snake(_: &Update, snake: SnakePos, bodies: Vec<SnakeBodyPos>, events: &mut dyn Events) {
    if bodies
        .iter()
        .find(|body| body.eid != snake.eid && body.hit_box.0.intersects(&snake.hit_box.0))
        .is_some()
    {
        events.set_state(GameOver::Data);
    }
}

#[hyperfold_engine::system]
fn collide_wall(collide: &BoundaryCollision, snake: SnakePos, events: &mut dyn Events) {
    if collide.0 == *snake.eid {
        events.set_state(GameOver::Data);
    }
}
