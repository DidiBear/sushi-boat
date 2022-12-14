#![deny(clippy::all, clippy::nursery, clippy::unwrap_used)]

use bevy::{prelude::*, utils::HashSet, window::close_on_esc};
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;
use item::{Item, ItemAssets};
use mouse_tracking::{MousePosition, MouseTrackingPlugin};
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::Rng;

mod item;
mod mouse_tracking;

const Z: f32 = 1.;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "SushiBoat".to_string(),
        width: 800.,
        height: 800.,
        ..default()
    })
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(Msaa::default())
    .insert_resource(Spawner::default())
    .insert_resource(GrabbedItem::default())
    .add_plugins(DefaultPlugins)
    .add_plugin(MouseTrackingPlugin)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
    .add_state(GameState::AssetLoading)
    .add_loading_state(
        LoadingState::new(GameState::AssetLoading)
            .continue_to_state(GameState::GamePlay)
            .with_collection::<ItemAssets>()
            .with_collection::<GameAssets>(),
    )
    .add_system_set(
        SystemSet::on_enter(GameState::GamePlay)
            .with_system(setup_camera_and_background)
            .with_system(setup_ground_and_ceiling),
    )
    .add_system_set(
        SystemSet::on_update(GameState::GamePlay)
            .with_system(spawn_incoming_items)
            .with_system(drag_and_drop_item)
            .with_system(despawn_felt_items)
            .with_system(combine_items),
    );

    if cfg!(debug_assertions) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(RapierDebugRenderPlugin::default())
            .add_system(close_on_esc);
    }
    app.run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    GamePlay,
}

#[derive(AssetCollection)]
struct GameAssets {
    #[asset(path = "sprites/background.jpg")]
    background: Handle<Image>,
    #[asset(path = "FiraSans-Bold.ttf")]
    font: Handle<Font>,
}

fn setup_camera_and_background(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands
        .spawn()
        .insert(Name::new("Background"))
        .insert_bundle(SpriteBundle {
            texture: game_assets.background.clone(),
            ..default()
        });

    commands.spawn_bundle(
        TextBundle::from_section(
            "Goal: Create the boat !",
            TextStyle {
                font: game_assets.font.clone(),
                font_size: 36.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            margin: UiRect {
                left: Val::Percent(3.),
                top: Val::Percent(2.),
                ..default()
            },
            ..default()
        }),
    );
}

fn setup_ground_and_ceiling(mut commands: Commands) {
    let ground_color = Color::rgb(0.8, 0.6, 0.3); // Brown
    commands
        .spawn()
        .insert(Name::new("Ground"))
        .insert(Collider::cuboid(300., 25.))
        .insert(Friction::new(1.2))
        .insert_bundle(SpriteBundle {
            transform: Transform::from_xyz(0., -300., Z),
            sprite: Sprite {
                color: ground_color,
                custom_size: Some(Vec2::new(300., 25.) * 2.),
                ..default()
            },
            ..default()
        });

    for side in [1., -1.] {
        commands
            .spawn()
            .insert(Name::new("Ground border"))
            .insert(Collider::cuboid(15., 30.))
            .insert(Friction::new(1.2))
            .insert_bundle(SpriteBundle {
                transform: Transform::from_xyz(285. * side, -275., Z),
                sprite: Sprite {
                    color: ground_color,
                    custom_size: Some(Vec2::new(15., 30.) * 2.),
                    ..default()
                },
                ..default()
            });
    }

    commands
        .spawn()
        .insert(Name::new("Ceiling"))
        .insert(Collider::cuboid(800., 20.))
        .insert(Friction::new(1.2))
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            0., 420., Z,
        )));
}

struct Spawner {
    timer: Timer,
    first_items: HashSet<Item>,
}

impl Default for Spawner {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.5, true),
            first_items: Item::BASIC.into_iter().collect(),
        }
    }
}

impl Spawner {
    fn generate_items(&mut self, rng: &mut impl Rng) -> Item {
        if self.first_items.is_empty() {
            *Item::BASIC.choose(rng).expect("basic item")
        } else {
            let item = self
                .first_items
                .iter()
                .choose(rng)
                .expect("first item")
                .clone();
            self.first_items.take(&item).expect("first item")
        }
    }
}

fn spawn_incoming_items(
    mut commands: Commands,
    mut spawner: ResMut<Spawner>,
    time: Res<Time>,
    item_assets: Res<ItemAssets>,
) {
    if spawner.timer.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        let item = spawner.generate_items(&mut rng);

        let side = [-1., 1.].choose(&mut rng).expect("random side");

        let translation = Vec2::new(450. * side, rng.gen_range(0.0..300.0));

        let velocity = Velocity {
            linvel: Vec2::new(-side, 1.) * rng.gen_range(150.0..200.0),
            angvel: rng.gen_range(-5.0..5.0),
        };

        spawn_item(&mut commands, item, translation, velocity, &item_assets);
    }
}

fn spawn_item(
    commands: &mut Commands,
    item: Item,
    translation: Vec2,
    velocity: Velocity,
    item_assets: &Res<ItemAssets>,
) {
    let transform = Transform::from_translation(Vec3::from((translation, Z)));

    let texture = item_assets.sprite_for(item);
    let sprite = Sprite {
        custom_size: Some(Vec2::from(item)),
        ..default()
    };

    commands
        .spawn()
        .insert(Name::new("Item"))
        .insert(item)
        .insert_bundle(SpriteBundle {
            texture,
            transform,
            sprite,
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::from(item))
        .insert(velocity)
        .insert(Ccd::enabled())
        .insert(GravityScale(3.5))
        .insert(ActiveEvents::COLLISION_EVENTS);
}

#[derive(Debug, Default)]
struct GrabbedItem(Option<Entity>);

fn drag_and_drop_item(
    mouse: Res<Input<MouseButton>>,
    mouse_position: Res<MousePosition>,
    mut items: Query<(Entity, &Item, &Collider, &Transform, &mut Velocity), With<Item>>,
    mut grabbed_item: ResMut<GrabbedItem>,
    audio: Res<Audio>,
    item_assets: Res<ItemAssets>,
) {
    if mouse.just_released(MouseButton::Left) {
        if let Some(item) = grabbed_item.0.take() {
            let (.., mut velocity) = items.get_mut(item).expect("item body");
            velocity.linvel = velocity.linvel.clamp_length_max(500.); // Cap speed when the player throw the item
        }
        return;
    }
    if mouse.just_pressed(MouseButton::Left) {
        grabbed_item.0 = items
            .iter()
            .find(|(_, _, collider, transform, _)| {
                collider.contains_local_point(mouse_position.0 - transform.translation.truncate())
            })
            .map(|(entity, ..)| entity);

        if let Some(entity) = grabbed_item.0 {
            let item = items
                .get_component::<Item>(entity)
                .expect("entity has item");

            audio.play(item_assets.sound_for(*item));
        }
    }

    if let Some(item) = grabbed_item.0 {
        // Move the grabbed item to the mouse cursor using the velocity
        let (.., transform, mut velocity) = items.get_mut(item).expect("item body");
        velocity.linvel = (mouse_position.0 - transform.translation.truncate()) * 10.;
        velocity.angvel *= 0.9; // Smoothly decelerate the rotations
    }
}

fn combine_items(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    items: Query<(Entity, &Item, &Transform)>,
    mut grabbed_item: ResMut<GrabbedItem>,
    item_assets: Res<ItemAssets>,
    game_assets: Res<GameAssets>,
    audio: Res<Audio>,
) {
    let collided_items = collision_events
        .iter()
        .filter_map(|event| match event {
            CollisionEvent::Started(e1, e2, ..) => Some(sorted([e1, e2])),
            _ => None,
        })
        .collect::<HashSet<_>>() // Remove double-counted collisions
        .into_iter()
        .filter_map(|[e1, e2]| items.get_many([*e1, *e2]).ok());

    for [(entity1, item1, transform1), (entity2, item2, transform2)] in collided_items {
        if let Some(combined_item) = Item::can_combine(*item1, *item2) {
            grabbed_item.0 = None;
            commands.entity(entity1).despawn_recursive();
            commands.entity(entity2).despawn_recursive();

            let in_between_translation = (transform1.translation + transform2.translation) / 2.;

            spawn_item(
                &mut commands,
                combined_item,
                in_between_translation.truncate(),
                Velocity::zero(),
                &item_assets,
            );
            audio.play(item_assets.sound_for(combined_item));

            if combined_item == Item::Boat {
                spawn_win_text(&mut commands, &game_assets);
            }
        }
    }
}

fn despawn_felt_items(mut commands: Commands, items: Query<(Entity, &Transform)>) {
    for (item, transform) in &items {
        if transform.translation.y < -600. {
            commands.entity(item).despawn_recursive();
        }
    }
}

fn spawn_win_text(commands: &mut Commands, game_assets: &GameAssets) {
    commands.spawn_bundle(
        TextBundle::from_section(
            "You win !",
            TextStyle {
                font: game_assets.font.clone(),
                font_size: 80.0,
                color: Color::WHITE,
            },
        )
        .with_style(Style {
            align_self: AlignSelf::FlexStart,
            ..default()
        }),
    );
}

fn sorted<const N: usize, T: Ord>(mut array: [T; N]) -> [T; N] {
    array.sort();
    array
}
