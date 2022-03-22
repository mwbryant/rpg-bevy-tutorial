use bevy::prelude::*;

use crate::{ascii::AsciiSheet, GameState};

pub struct FadeoutPlugin;

#[derive(Component)]
struct ScreenFade {
    alpha: f32,
    sent: bool,
    next_state: GameState,
}

impl Plugin for FadeoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(fadeout);
    }
}

fn fadeout(
    mut commands: Commands,
    mut fade_query: Query<(Entity, &mut ScreenFade, &mut Timer, &mut TextureAtlasSprite)>,
    mut state: ResMut<State<GameState>>,
    time: Res<Time>,
) {
    for (entity, mut fade, mut timer, mut sprite) in fade_query.iter_mut() {
        timer.tick(time.delta());
        if timer.percent() < 0.5 {
            fade.alpha = timer.percent() * 2.0;
        } else {
            fade.alpha = timer.percent_left() * 2.0;
        }
        sprite.color.set_a(fade.alpha);

        if timer.percent() > 0.5 && !fade.sent {
            state.set(fade.next_state).unwrap();
            fade.sent = true;
        }

        if timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn create_fadeout(commands: &mut Commands, next_state: GameState, ascii: &Res<AsciiSheet>) {
    let mut sprite = TextureAtlasSprite::new(0);
    sprite.color = Color::rgba(0.1, 0.1, 0.15, 0.0);
    sprite.custom_size = Some(Vec2::splat(100000.0));

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: sprite,
            texture_atlas: ascii.0.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 999.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Timer::from_seconds(1.0, false))
        .insert(ScreenFade {
            alpha: 0.0,
            sent: false,
            next_state: next_state,
        })
        .insert(Name::new("Fadeout"));
}
