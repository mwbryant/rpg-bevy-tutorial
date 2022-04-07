use bevy::prelude::*;

pub struct GraphicsPlugin;

pub struct CharacterSheet {
    handle: Handle<TextureAtlas>,
    bat_frames: [usize; 3],
}

#[derive(Component)]
pub struct FrameAnimation {
    timer: Timer,
    frames: Vec<usize>,
    current_frame: usize,
}

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, Self::load_graphics)
            .add_system(Self::frame_animation);
    }
}

pub fn spawn_bat_sprite(
    commands: &mut Commands,
    characters: &CharacterSheet,
    translation: Vec3,
) -> Entity {
    let mut sprite = TextureAtlasSprite::new(characters.bat_frames[0]);
    sprite.custom_size = Some(Vec2::splat(0.5));

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: sprite,
            texture_atlas: characters.handle.clone(),
            transform: Transform {
                translation: translation,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FrameAnimation {
            timer: Timer::from_seconds(0.2, true),
            frames: characters.bat_frames.to_vec(),
            current_frame: 0,
        })
        .id()
}

impl GraphicsPlugin {
    fn load_graphics(
        mut commands: Commands,
        assets: Res<AssetServer>,
        mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    ) {
        let image = assets.load("characters.png");
        let atlas =
            TextureAtlas::from_grid_with_padding(image, Vec2::splat(16.0), 12, 8, Vec2::splat(2.0));
        let atlas_handle = texture_atlases.add(atlas);

        commands.insert_resource(CharacterSheet {
            handle: atlas_handle,
            bat_frames: [12 * 4 + 3, 12 * 4 + 4, 12 * 4 + 5],
        });
    }

    fn frame_animation(
        mut sprites_query: Query<(&mut TextureAtlasSprite, &mut FrameAnimation)>,
        time: Res<Time>,
    ) {
        for (mut sprite, mut animation) in sprites_query.iter_mut() {
            animation.timer.tick(time.delta());
            if animation.timer.just_finished() {
                animation.current_frame = (animation.current_frame + 1) % animation.frames.len();
                sprite.index = animation.frames[animation.current_frame];
            }
        }
    }
}
