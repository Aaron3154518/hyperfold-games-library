use crate::{_engine::AddComponent, elevations::Elevations, fruit::EatFruit};
use hyperfold_engine::{
    _engine::Entity,
    add_components, components,
    ecs::{
        entities::{EntityTrash, NewEntity},
        events::core::Update,
    },
    framework::{
        physics::Position,
        render_system::{
            render_data::RenderAsset, AssetManager, Elevation, RenderComponent, Renderer,
        },
    },
    utils::{
        rect::{Align, PointF, Rect},
        timer::{Timer, TimerTrait},
    },
};

pub fn fruit_effect_image(i: u8) -> String {
    format!("res/snake/fruit_effects_{i}.png")
}

#[hyperfold_engine::component]
struct FruitEffect {
    pub fruit: Entity,
    pub img: u8,
}

pub fn new_fruit_effect(
    img: u8,
    fruit: Entity,
    pos: PointF,
    entities: &mut dyn AddComponent,
    r: &Renderer,
    am: &mut AssetManager,
) {
    let e = Entity::new();
    add_components!(
        entities,
        e,
        FruitEffect { fruit, img },
        Timer::new(2000),
        Elevation(Elevations::FruitEffect as u8),
        RenderComponent::new(RenderAsset::from_file(&fruit_effect_image(img), r, am)),
        Position(Rect::from(
            pos.x,
            pos.y,
            25.0,
            25.0,
            Align::Center,
            Align::Center
        ))
    );
}

components!(
    FruitEffects,
    effect: &'a mut FruitEffect,
    timer: &'a mut Timer,
    pos: &'a Position,
);

#[hyperfold_engine::system]
fn update_fruit_effects(
    update: &Update,
    effects: Vec<FruitEffects>,
    trash: &mut EntityTrash,
    entities: &mut dyn AddComponent,
    r: &Renderer,
    am: &mut AssetManager,
) {
    for FruitEffects {
        eid,
        effect,
        timer,
        pos,
    } in effects
    {
        let prev_over = timer.time_left() > 200;
        if timer.add_time(update.0) > 0 {
            trash.0.push(*eid);
        } else if prev_over && timer.time_left() <= 200 {
            new_fruit_effect(
                (effect.img + 1) % 3,
                effect.fruit,
                pos.0.center(),
                entities,
                r,
                am,
            );
        }
    }
}

#[hyperfold_engine::system]
fn on_remove_fruit(fruit: &EatFruit, effects: Vec<FruitEffects>, trash: &mut EntityTrash) {
    trash.0.extend(
        effects
            .into_iter()
            .filter_map(|effect| (effect.effect.fruit == fruit.0).then_some(effect.eid))
            .collect::<Vec<_>>(),
    );
}
