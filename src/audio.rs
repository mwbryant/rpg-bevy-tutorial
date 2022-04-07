use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin, AudioSource};

use crate::combat::{CombatState, FightEvent};
use crate::GameState;

pub struct GameAudioPlugin;

pub struct AudioState {
    bgm_handle: Handle<AudioSource>,
    combat_handle: Handle<AudioSource>,
    hit_handle: Handle<AudioSource>,
    reward_handle: Handle<AudioSource>,

    bgm_channel: AudioChannel,
    combat_channel: AudioChannel,
    sfx_channel: AudioChannel,
    volume: f32,
}

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_startup_system_to_stage(StartupStage::PreStartup, load_audio)
            .add_system_set(SystemSet::on_enter(GameState::Combat).with_system(start_combat_music))
            .add_system_set(SystemSet::on_enter(GameState::Overworld).with_system(resume_bgm_music))
            .add_system_set(SystemSet::on_enter(CombatState::Reward).with_system(play_reward_sfx))
            .add_system(play_hit_sfx)
            .add_system(volume_control)
            .add_startup_system(start_bgm_music);
    }
}
fn play_reward_sfx(audio: Res<Audio>, audio_state: Res<AudioState>) {
    audio.play_in_channel(audio_state.reward_handle.clone(), &audio_state.sfx_channel);
}

fn play_hit_sfx(
    audio: Res<Audio>,
    audio_state: Res<AudioState>,
    mut fight_event: EventReader<FightEvent>,
) {
    if fight_event.iter().count() > 0 {
        audio.play_in_channel(audio_state.hit_handle.clone(), &audio_state.sfx_channel);
    }
}

fn resume_bgm_music(audio: Res<Audio>, audio_state: Res<AudioState>) {
    audio.stop_channel(&audio_state.combat_channel);
    audio.resume_channel(&audio_state.bgm_channel);
}

fn start_combat_music(audio: Res<Audio>, audio_state: Res<AudioState>) {
    audio.pause_channel(&audio_state.bgm_channel);
    audio.play_looped_in_channel(
        audio_state.combat_handle.clone(),
        &audio_state.combat_channel,
    );
}

fn volume_control(
    keyboard: Res<Input<KeyCode>>,
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
) {
    if keyboard.just_pressed(KeyCode::Up) {
        audio_state.volume += 0.10;
    }
    if keyboard.just_pressed(KeyCode::Down) {
        audio_state.volume -= 0.10;
    }
    audio_state.volume = audio_state.volume.clamp(0.0, 1.0);
    audio.set_volume_in_channel(audio_state.volume, &audio_state.bgm_channel);
}

fn start_bgm_music(audio: Res<Audio>, audio_state: Res<AudioState>) {
    audio.play_looped_in_channel(audio_state.bgm_handle.clone(), &audio_state.bgm_channel);
}

fn load_audio(mut commands: Commands, audio: Res<Audio>, assets: Res<AssetServer>) {
    let bgm_handle = assets.load("bip-bop.ogg");
    let combat_handle = assets.load("ganxta.ogg");
    let hit_handle = assets.load("hit.wav");
    let reward_handle = assets.load("reward.wav");

    let bgm_channel = AudioChannel::new("bgm".to_string());
    let combat_channel = AudioChannel::new("combat".to_string());
    let sfx_channel = AudioChannel::new("sfx".to_string());
    let volume = 0.5;

    audio.set_volume_in_channel(volume, &bgm_channel);
    audio.set_volume_in_channel(volume, &combat_channel);
    audio.set_volume_in_channel(volume, &sfx_channel);

    commands.insert_resource(AudioState {
        bgm_handle: bgm_handle,
        combat_handle: combat_handle,
        hit_handle: hit_handle,
        reward_handle: reward_handle,
        bgm_channel,
        combat_channel,
        sfx_channel,
        volume,
    });
}
