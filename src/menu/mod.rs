use crate::loading::{FontAssets, TextureAssets, UiAssets};
use crate::ui::*;
use crate::GameState;
use bevy::prelude::*;

pub mod load_game;
pub mod new_game;
pub mod settings;

pub struct MenuPlugin;

#[derive(Component)]
pub struct MenuBackdrop;

pub type MenuBackdropQuery<'a, 'b> = Query<'a, 'b, Entity, With<MenuBackdrop>>;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<MenuNavState>()
            .add_state_scoped_event::<MenuNavState>(GameState::Menu)
            .add_systems(
                Update,
                handle_menu_nav_events.run_if(on_event::<MenuNavState>),
            )
            .enable_state_scoped_entities::<MenuNavState>()
            .add_systems(
                OnEnter(GameState::Menu),
                (spawn_camera, spawn_menu_backdrop).chain(),
            )
            .add_systems(OnEnter(MenuNavState::Root), (setup,))
            // handle menu navigation
            .add_systems(Update, main_menu_button.run_if(in_state(GameState::Menu)))
            // settings
            .add_plugins((settings::SettingsPlugin,));
    }
}

pub fn spawn_menu_backdrop(mut cmd: Commands, textures: Res<TextureAssets>) {
    cmd.spawn((
        MenuBackdrop,
        StateScoped(GameState::Menu),
        ZIndex::from(ZIndices::Menu),
        Node {
            height: Val::Vh(100.),
            width: Val::Vw(100.),
            ..default()
        },
        BackgroundColor(Color::WHITE),
        ImageNode {
            image: textures.products.clone(),
            image_mode: bevy::ui::widget::NodeImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.,
            },
            ..Default::default()
        },
    ));
}

fn spawn_camera(mut cmd: Commands) {
    cmd.spawn((
        StateScoped(GameState::Menu),
        Camera2d,
        Msaa::Off,
        UiAntiAlias::Off,
    ));
}

fn setup(
    mut cmd: Commands,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    backdrop: Query<Entity, With<MenuBackdrop>>,
) {
    cmd.entity(backdrop.single()).with_children(|parent| {
        parent
            .spawn((
                StateScoped(MenuNavState::Root),
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|parent| {
                // Title text
                parent.spawn((ImageNode {
                    image: ui.title.clone(),
                    ..Default::default()
                },));

                // Main menu buttons
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            height: Val::Auto,
                            width: Val::Auto,
                            row_gap: Val::Px(0.),
                            ..default()
                        },
                        //BorderRadius::all(Val::Px(16.)),
                        BackgroundColor(Color::srgba_u8(255, 255, 255, 20)),
                    ))
                    .with_children(|children| {
                        for (name, game_state, menu_nav) in &[
                            //("Continue", Some(GameState::Playing), None),
                            //("New Game", None, Some(MenuNavState::NewGame)),
                            //("Load Game", None, Some(MenuNavState::LoadGame)),
                            ("Editor", Some(GameState::Playing), None),
                            ("Settings", None, Some(MenuNavState::Settings)),
                            #[cfg(not(target_arch = "wasm32"))]
                            ("Quit", Some(GameState::Quit), None),
                        ] {
                            match (*game_state, *menu_nav) {
                                (Some(game_state), None) => {
                                    spawn_button(
                                        (*name).into(),
                                        ChangeStates(game_state),
                                        children,
                                        &fonts,
                                        &ui,
                                    );
                                }
                                (None, Some(menu_nav)) => {
                                    spawn_button(
                                        (*name).into(),
                                        ChangeStates(menu_nav),
                                        children,
                                        &fonts,
                                        &ui,
                                    );
                                }
                                _ => {}
                            }
                        }
                    });

                let button_skins = ButtonSkins {
                    normal: ui.kofi_donation_link.clone(),
                    hover: ui.kofi_donation_link_dark.clone(),
                    active: ui.kofi_donation_link_red.clone(),
                };

                // Donation link
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            position_type: PositionType::Absolute,
                            right: Val::Px(32.),
                            bottom: Val::Px(48.),
                            height: Val::Px(90.),
                            width: Val::Px(180.),
                            ..default()
                        },
                        BackgroundColor(Color::WHITE),
                        ImageNode {
                            image: ui.kofi_donation_link.clone(),
                            image_mode: bevy::ui::widget::NodeImageMode::Stretch,
                            ..Default::default()
                        },
                        OpenLink(env!("PROJECT_SUPPORT_LINK")),
                        DepressButton::default(),
                        button_skins,
                    ))
                    .with_children(|children| {
                        children.spawn_empty();
                    });
            });
    });
}

#[derive(Component)]
struct ChangeStates<T: States>(T);

#[derive(Component)]
struct OpenLink(&'static str);

/// Menu page navigation
#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash, Copy, Event)]
#[source(GameState = GameState::Menu) ]
pub enum MenuNavState {
    #[default]
    Root,
    //NewGame,
    //LoadGame,
    Settings,
}

fn main_menu_button(
    buttons: Query<
        (
            &DepressButton,
            Option<&OpenLink>,
            Option<&ChangeStates<GameState>>,
            Option<&ChangeStates<MenuNavState>>,
        ),
        Changed<DepressButton>,
    >,
    mut game_state_channel: EventWriter<GameState>,
    mut menu_nav_channel: EventWriter<MenuNavState>,
) {
    for (depress, link, game_state, menu_nav) in buttons.iter() {
        if depress.invoked() {
            link.map(|OpenLink(link)| {
                if let Err(error) = webbrowser::open(link) {
                    warn!("Failed to open link {error:?}");
                }
            });

            game_state.map(|gs| {
                game_state_channel.send(gs.0);
            });

            menu_nav.map(|mn| {
                menu_nav_channel.send(mn.0);
            });
        }
    }
}

pub fn handle_menu_nav_events(
    mut menu_nav_events: EventReader<MenuNavState>,
    mut next_menu_nav: ResMut<NextState<MenuNavState>>,
) {
    for menu_nav in menu_nav_events.read() {
        next_menu_nav.set(*menu_nav);
    }
}
