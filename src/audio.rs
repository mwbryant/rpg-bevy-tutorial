use bevy::prelude::*;
use bevy_kira_audio::{AudioChannel, AudioPlugin, AudioSource, AudioApp, AudioControl};

use crate::combat::{CombatState, FightEvent};
use crate::GameState;

pub struct GameAudioPlugin;

struct Background;
struct Effects;
struct Combat;
pub struct AudioState {
    bgm_handle: Handle<AudioSource>,
    combat_handle: Handle<AudioSource>,
    hit_handle: Handle<AudioSource>,
    reward_handle: Handle<AudioSource>,
    volume: f64,
}

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_audio_channel::<Background>()
            .add_audio_channel::<Effects>()
            .add_audio_channel::<Combat>()
            .add_startup_system_to_stage(StartupStage::PreStartup, load_audio)
            .add_system_set(SystemSet::on_enter(GameState::Combat).with_system(start_combat_music))
            .add_system_set(
                SystemSet::on_resume(GameState::Overworld).with_system(resume_bgm_music),
            )
            .add_system_set(SystemSet::on_enter(CombatState::Reward).with_system(play_reward_sfx))
            .add_system(play_hit_sfx)
            .add_system(volume_control)
            .add_startup_system(start_bgm_music);
    }
}
fn play_reward_sfx(audio: Res<AudioChannel<Effects>>, audio_state: Res<AudioState>) {
    audio.play(audio_state.reward_handle.clone());
}

fn play_hit_sfx(
    audio: Res<AudioChannel<Effects>>,
    audio_state: Res<AudioState>,
    mut fight_event: EventReader<FightEvent>,
) {
    if fight_event.iter().count() > 0 {
        audio.play(audio_state.hit_handle.clone());
    }
}

fn resume_bgm_music(
    bg_audio: Res<AudioChannel<Background>>, 
    combat_audio: Res<AudioChannel<Combat>>, 
) {
    combat_audio.stop();
    bg_audio.resume();
}

fn start_combat_music(
    bg_audio: Res<AudioChannel<Background>>, 
    combat_audio: Res<AudioChannel<Combat>>, 
    audio_state: Res<AudioState>
) {
    bg_audio.pause();
    combat_audio.play(audio_state.combat_handle.clone()).looped();
}

fn volume_control(
    keyboard: Res<Input<KeyCode>>,
    bg_audio: Res<AudioChannel<Background>>, 
    mut audio_state: ResMut<AudioState>,
) {
    if keyboard.just_pressed(KeyCode::Up) {
        audio_state.volume += 0.10;
    }
    if keyboard.just_pressed(KeyCode::Down) {
        audio_state.volume -= 0.10;
    }
    audio_state.volume = audio_state.volume.clamp(0.0, 1.0);
    bg_audio.set_volume(audio_state.volume);
}

fn start_bgm_music(
    bg_audio: Res<AudioChannel<Background>>, 
    audio_state: Res<AudioState>
) {
    bg_audio.play(audio_state.bgm_handle.clone()).looped();
}

fn load_audio(
    mut commands: Commands, 
    bg_audio: Res<AudioChannel<Background>>, 
    fx_audio: Res<AudioChannel<Effects>>, 
    cb_audio: Res<AudioChannel<Combat>>, 
    assets: Res<AssetServer>
) {
    let bgm_handle = assets.load("bip-bop.ogg");
    let combat_handle = assets.load("ganxta.ogg");
    let hit_handle = assets.load("hit.wav");
    let reward_handle = assets.load("reward.wav");


    let volume = 0.5;

    bg_audio.set_volume(volume);
    fx_audio.set_volume(volume);
    cb_audio.set_volume(volume);

    commands.insert_resource(AudioState {
        bgm_handle: bgm_handle,
        combat_handle: combat_handle,
        hit_handle: hit_handle,
        reward_handle: reward_handle,
        volume,
    });
}
