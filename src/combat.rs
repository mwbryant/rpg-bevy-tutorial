use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

use crate::{
    ascii::{
        spawn_ascii_sprite, spawn_ascii_text, spawn_nine_slice, AsciiSheet, AsciiText, NineSlice,
        NineSliceIndices,
    },
    fadeout::create_fadeout,
    player::Player,
    GameState, RESOLUTION, TILE_SIZE,
};

#[derive(Component, Inspectable)]
pub struct CombatStats {
    //XXX does this need isize, combat does a subtract but I max it
    pub health: isize,
    pub max_health: isize,
    pub attack: isize,
    pub defense: isize,
}

#[derive(Component)]
pub struct Enemy;

struct FightEvent {
    target: Entity,
    damage_amount: isize,
    next_state: CombatState,
}

pub struct CombatPlugin;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum CombatState {
    PlayerTurn,
    EnemyTurn(bool),
    Exiting,
}

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FightEvent>()
            .add_state(CombatState::PlayerTurn)
            //TODO reset
            .insert_resource(CombatMenuSelection {
                selected: CombatMenuOption::Fight,
            })
            .add_system_set(
                SystemSet::on_update(CombatState::EnemyTurn(false)).with_system(process_enemy_turn),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Combat)
                    .with_system(combat_input)
                    .with_system(combat_camera)
                    .with_system(highlight_combat_buttons)
                    .with_system(combat_damage_calc),
            )
            .add_system_set(
                SystemSet::on_enter(GameState::Combat)
                    .with_system(set_starting_state)
                    .with_system(spawn_enemy)
                    .with_system(spawn_combat_menu),
            )
            .add_system_set(
                SystemSet::on_exit(GameState::Combat)
                    .with_system(despawn_enemy)
                    .with_system(despawn_menu),
            );
    }
}

fn set_starting_state(mut state: ResMut<State<CombatState>>) {
    let _ = state.set(CombatState::PlayerTurn);
}

const NUM_MENU_OPTIONS: isize = 2;
#[derive(Component, PartialEq, Clone, Copy)]
pub enum CombatMenuOption {
    Fight,
    Run,
}

pub struct CombatMenuSelection {
    selected: CombatMenuOption,
}

fn process_enemy_turn(
    mut fight_event: EventWriter<FightEvent>,
    mut combat_state: ResMut<State<CombatState>>,
    enemy_query: Query<&CombatStats, With<Enemy>>,
    player_query: Query<Entity, With<Player>>,
) {
    let player_ent = player_query.single();
    let enemy_stats = enemy_query.iter().next().unwrap();
    fight_event.send(FightEvent {
        target: player_ent,
        damage_amount: enemy_stats.attack,
        next_state: CombatState::PlayerTurn,
    });
    combat_state.set(CombatState::EnemyTurn(true)).unwrap();
}

fn despawn_menu(mut commands: Commands, button_query: Query<Entity, With<CombatMenuOption>>) {
    for button in button_query.iter() {
        commands.entity(button).despawn_recursive();
    }
}

fn highlight_combat_buttons(
    menu_state: Res<CombatMenuSelection>,
    button_query: Query<(&Children, &CombatMenuOption)>,
    nine_slice_query: Query<&Children, With<NineSlice>>,
    mut sprites_query: Query<&mut TextureAtlasSprite>,
) {
    for (button_children, button_id) in button_query.iter() {
        for button_child in button_children.iter() {
            if let Ok(nine_slice_children) = nine_slice_query.get(*button_child) {
                for nine_slice_child in nine_slice_children.iter() {
                    if let Ok(mut sprite) = sprites_query.get_mut(*nine_slice_child) {
                        if menu_state.selected == *button_id {
                            sprite.color = Color::RED;
                        } else {
                            sprite.color = Color::WHITE;
                        }
                    }
                }
            }
        }
    }
}

fn spawn_combat_button(
    commands: &mut Commands,
    ascii: &AsciiSheet,
    indices: &NineSliceIndices,
    translation: Vec3,
    text: &str,
    id: CombatMenuOption,
    size: Vec2,
) -> Entity {
    let fight_nine_slice = spawn_nine_slice(commands, ascii, indices, size.x, size.y);

    let x_offset = (-size.x / 2.0 + 1.5) * TILE_SIZE;
    let fight_text = spawn_ascii_text(commands, ascii, text, Vec3::new(x_offset, 0.0, 0.0));

    commands
        .spawn()
        .insert(Transform {
            translation: translation,
            ..Default::default()
        })
        .insert(GlobalTransform::default())
        .insert(Name::new("Button"))
        .insert(id)
        .add_child(fight_text)
        .add_child(fight_nine_slice)
        .id()
}

fn spawn_combat_menu(
    mut commands: Commands,
    ascii: Res<AsciiSheet>,
    nine_slice_indices: Res<NineSliceIndices>,
) {
    let box_height = 3.0;
    let box_center_y = -1.0 + box_height * TILE_SIZE / 2.0;

    let run_text = "Run";
    let run_width = (run_text.len() + 2) as f32;
    let run_center_x = RESOLUTION - (run_width * TILE_SIZE) / 2.0;

    spawn_combat_button(
        &mut commands,
        &ascii,
        &nine_slice_indices,
        Vec3::new(run_center_x, box_center_y, 100.0),
        run_text,
        CombatMenuOption::Run,
        Vec2::new(run_width, box_height),
    );

    let fight_text = "Fight";
    let fight_width = (fight_text.len() + 2) as f32;
    let fight_center_x = RESOLUTION - (run_width * TILE_SIZE) - (fight_width * TILE_SIZE / 2.0);

    spawn_combat_button(
        &mut commands,
        &ascii,
        &nine_slice_indices,
        Vec3::new(fight_center_x, box_center_y, 100.0),
        fight_text,
        CombatMenuOption::Fight,
        Vec2::new(fight_width, box_height),
    );
}

fn combat_damage_calc(
    mut commands: Commands,
    mut fight_event: EventReader<FightEvent>,
    //Not necssacarily enemy
    mut enemy_query: Query<(&Children, &mut CombatStats)>,
    ascii: Res<AsciiSheet>,
    text_query: Query<&AsciiText>,
    mut combat_state: ResMut<State<CombatState>>,
) {
    if let Some(fight_event) = fight_event.iter().next() {
        //Get target stats and children
        let (target_children, mut stats) = enemy_query
            .get_mut(fight_event.target)
            .expect("Fighting enemy without stats");

        //Damage calc
        stats.health = std::cmp::max(
            stats.health - (fight_event.damage_amount - stats.defense),
            0,
        );

        //Update health
        for child in target_children.iter() {
            //See if this child is the health text
            if text_query.get(*child).is_ok() {
                //Delete old text
                commands.entity(*child).despawn_recursive();
                //Create new text
                let new_health = spawn_ascii_text(
                    &mut commands,
                    &ascii,
                    &format!("Health: {}", stats.health as usize),
                    //relative to enemy pos
                    Vec3::new(-4.5 * TILE_SIZE, 2.0 * TILE_SIZE, 100.0),
                );

                commands.entity(fight_event.target).add_child(new_health);
            }
        }

        //Kill enemy if dead
        //TODO support multiple enemies
        //FIXME should create fadeout!
        if stats.health == 0 {
            create_fadeout(&mut commands, GameState::Overworld, &ascii);
            combat_state.set(CombatState::Exiting).unwrap();
        } else {
            combat_state.set(fight_event.next_state).unwrap();
        }
    }
}

fn combat_input(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    player_query: Query<&CombatStats, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut fight_event: EventWriter<FightEvent>,
    mut menu_state: ResMut<CombatMenuSelection>,
    mut combat_state: ResMut<State<CombatState>>,
    ascii: Res<AsciiSheet>,
) {
    if combat_state.current() != &CombatState::PlayerTurn {
        return;
    }

    let player_combat = player_query.single();

    //TODO handle multiple enemies
    let enemy = enemy_query.single();
    let mut new_selection = menu_state.selected as isize;
    if keyboard.just_pressed(KeyCode::A) {
        new_selection -= 1;
    }
    if keyboard.just_pressed(KeyCode::D) {
        new_selection += 1;
    }
    new_selection = (new_selection + NUM_MENU_OPTIONS) % NUM_MENU_OPTIONS;

    menu_state.selected = match new_selection {
        0 => CombatMenuOption::Fight,
        1 => CombatMenuOption::Run,
        _ => unreachable!("Bad menu selection"),
    };

    if keyboard.just_pressed(KeyCode::Space) {
        match menu_state.selected {
            CombatMenuOption::Fight => fight_event.send(FightEvent {
                //TODO select enemy and attack type
                target: enemy,
                damage_amount: player_combat.attack,
                next_state: CombatState::EnemyTurn(false),
            }),
            CombatMenuOption::Run => {
                create_fadeout(&mut commands, GameState::Overworld, &ascii);
                combat_state.set(CombatState::Exiting).unwrap()
            }
        }
    }
}

fn combat_camera(mut camera_query: Query<&mut Transform, With<Camera>>) {
    let mut camera_transform = camera_query.single_mut();
    camera_transform.translation.x = 0.0;
    camera_transform.translation.y = 0.0;
}

fn spawn_enemy(mut commands: Commands, ascii: Res<AsciiSheet>) {
    let enemy_health = 3;
    let health_text = spawn_ascii_text(
        &mut commands,
        &ascii,
        &format!("Health: {}", enemy_health as usize),
        //relative to enemy pos
        Vec3::new(-4.5 * TILE_SIZE, 2.0 * TILE_SIZE, 100.0),
    );
    let sprite = spawn_ascii_sprite(
        &mut commands,
        &ascii,
        'b' as usize,
        Color::rgb(0.8, 0.8, 0.8),
        Vec3::new(0.0, -0.1, 100.0),
        Vec3::splat(3.0),
    );
    commands
        .entity(sprite)
        .insert(Enemy)
        .insert(CombatStats {
            health: enemy_health,
            max_health: enemy_health,
            attack: 2,
            defense: 1,
        })
        .insert(Name::new("Bat"))
        .add_child(health_text);
}

fn despawn_enemy(mut commands: Commands, enemy_query: Query<Entity, With<Enemy>>) {
    for entity in enemy_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
