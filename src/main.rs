#![deny(clippy::all, clippy::nursery, clippy::unwrap_used)]

use bevy::{prelude::*, utils::HashSet, window::close_on_esc};
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;
use mouse_tracking::{MousePosition, MouseTrackingPlugin};
use rand::Rng;

mod mouse_tracking;
const Z: f32 = 1.;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Stacker".to_string(),
            width: 800.,
            height: 800.,
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::default())
        .insert_resource(Spawner(Timer::from_seconds(3., true)))
        .insert_resource(GrabbedItem::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(MouseTrackingPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        .add_plugin(RapierDebugRenderPlugin::default())
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
                .with_system(setup_ground),
        )
        .add_system_set(
            SystemSet::on_update(GameState::GamePlay)
                .with_system(spawn_incoming_items)
                .with_system(drag_and_drop_item)
                .with_system(combine_items),
        )
        .add_system(close_on_esc)
        .run();
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
}

fn setup_ground(mut commands: Commands) {
    commands
        .spawn()
        .insert(Name::new("Ground"))
        .insert(Collider::cuboid(250., 20.))
        .insert(Friction::new(1.2))
        .insert_bundle(SpriteBundle {
            transform: Transform::from_xyz(0., -250., Z),
            sprite: Sprite {
                color: Color::rgb(0.8, 0.6, 0.3), // Brown
                custom_size: Some(Vec2::new(250., 20.) * 2.),
                ..default()
            },
            ..default()
        });
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Item {
    // Basic
    Rice,
    SeaWeed,
    Avocado,
    Fish,
    // Level 1
    Onigiri,
    Maki,
    Sushi,
    // Level 2
    MakiSushiTray,
}

impl Item {
    const BASIC: [Self; 4] = [Self::Rice, Self::SeaWeed, Self::Avocado, Self::Fish];

    fn can_combine(item1: Self, item2: Self) -> Option<Self> {
        use Item::*;

        match sorted([item1, item2]) {
            [Rice, SeaWeed] => Some(Onigiri),
            [Rice, Fish] => Some(Sushi),
            [Avocado, Onigiri] => Some(Maki),
            [Maki, Sushi] => Some(MakiSushiTray),
            _ => None,
        }
    }
}

impl From<Item> for Collider {
    fn from(item: Item) -> Self {
        let size = Vec2::from(item) * 0.4;

        Self::cuboid(size.x, size.y)
    }
}

impl From<Item> for Vec2 {
    fn from(item: Item) -> Self {
        match item {
            Item::Rice => Self::new(200., 212.) * 0.5,
            Item::SeaWeed => Self::new(200., 186.) * 0.5,
            Item::Avocado => Self::new(200., 212.) * 0.5,
            Item::Fish => Self::new(200., 113.) * 0.65,
            Item::Onigiri => Self::new(200., 197.) * 0.60,
            Item::Maki => Self::new(200., 205.) * 0.65,
            Item::Sushi => Self::new(200., 180.) * 0.75,
            Item::MakiSushiTray => Self::new(400., 336.) * 0.65,
        }
    }
}

#[derive(AssetCollection)]
struct ItemAssets {
    // Sprites
    #[asset(path = "sprites/rice.png")]
    rice_sprite: Handle<Image>,
    #[asset(path = "sprites/sea-weed.png")]
    sea_weed_sprite: Handle<Image>,
    #[asset(path = "sprites/fish.png")]
    fish_sprite: Handle<Image>,
    #[asset(path = "sprites/avocado.png")]
    avocado_sprite: Handle<Image>,
    #[asset(path = "sprites/onigiri.png")]
    onigiri_sprite: Handle<Image>,
    #[asset(path = "sprites/maki.png")]
    maki_sprite: Handle<Image>,
    #[asset(path = "sprites/sushi.png")]
    sushi_sprite: Handle<Image>,
    #[asset(path = "sprites/maki-sushi-tray.png")]
    maki_sushi_tray_sprite: Handle<Image>,
    // Sounds
    #[asset(path = "audio/rice.ogg")]
    rice_sound: Handle<AudioSource>,
    #[asset(path = "audio/sea-weed.ogg")]
    sea_weed_sound: Handle<AudioSource>,
    #[asset(path = "audio/fish.ogg")]
    fish_sound: Handle<AudioSource>,
    #[asset(path = "audio/avocado.ogg")]
    avocado_sound: Handle<AudioSource>,
    #[asset(path = "audio/onigiri.ogg")]
    onigiri_sound: Handle<AudioSource>,
    #[asset(path = "audio/maki.ogg")]
    maki_sound: Handle<AudioSource>,
    #[asset(path = "audio/sushi.ogg")]
    sushi_sound: Handle<AudioSource>,
    #[asset(path = "audio/maki-sushi-tray.ogg")]
    maki_sushi_tray_sound: Handle<AudioSource>,
}

impl ItemAssets {
    fn sprite_for(&self, item: Item) -> Handle<Image> {
        match item {
            Item::Rice => self.rice_sprite.clone(),
            Item::SeaWeed => self.sea_weed_sprite.clone(),
            Item::Avocado => self.avocado_sprite.clone(),
            Item::Fish => self.fish_sprite.clone(),
            Item::Onigiri => self.onigiri_sprite.clone(),
            Item::Maki => self.maki_sprite.clone(),
            Item::Sushi => self.sushi_sprite.clone(),
            Item::MakiSushiTray => self.maki_sushi_tray_sprite.clone(),
        }
    }

    fn sound_for(&self, item: Item) -> Handle<AudioSource> {
        match item {
            Item::Rice => self.rice_sound.clone(),
            Item::SeaWeed => self.sea_weed_sound.clone(),
            Item::Avocado => self.avocado_sound.clone(),
            Item::Fish => self.fish_sound.clone(),
            Item::Onigiri => self.onigiri_sound.clone(),
            Item::Maki => self.maki_sound.clone(),
            Item::Sushi => self.sushi_sound.clone(),
            Item::MakiSushiTray => self.maki_sushi_tray_sound.clone(),
        }
    }
}

struct Spawner(Timer);

fn spawn_incoming_items(
    mut commands: Commands,
    mut spawner: ResMut<Spawner>,
    time: Res<Time>,
    item_assets: Res<ItemAssets>,
) {
    if spawner.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        let item = Item::BASIC[rng.gen_range(0..Item::BASIC.len())];

        let side = [-1., 1.][rng.gen_range(0..2)]; // left or right

        let translation = Vec2::new(450. * side, rng.gen_range(0.0..300.0));

        let velocity = Velocity {
            linvel: Vec2::new(-side, 1.) * rng.gen_range(150.0..200.0),
            angvel: rng.gen_range(-10.0..10.0),
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
        .insert(GravityScale(3.))
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
            let (.., mut velocity) = items.get_mut(item).expect("item has body");
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
        let (.., transform, mut velocity) = items.get_mut(item).expect("item has body");
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
        }
    }
}

fn sorted<const N: usize, T: Ord>(mut array: [T; N]) -> [T; N] {
    array.sort();
    array
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_can_combine() {
        use Item::*;

        let result = Item::can_combine(Rice, Fish);

        assert_eq!(result, Some(Sushi));
        assert_eq!(result, Item::can_combine(Fish, Rice));
        //                                   ^ swapped items
    }
}
