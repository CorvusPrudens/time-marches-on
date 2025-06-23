use bevy::{
    input_focus::{
        InputDispatchPlugin, InputFocus, InputFocusVisible,
        directional_navigation::{DirectionalNavigationMap, DirectionalNavigationPlugin},
    },
    math::CompassOctant,
    platform::collections::HashMap,
    prelude::*,
};
use bevy_enhanced_input::prelude::{Actions, InputContextAppExt};

use crate::player::{Player, PlayerContext};

mod input;
mod item;
mod pause;
mod scroll;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            // Input focus is not enabled by default, so we need to add the corresponding plugins
            .add_plugins((
                scroll::ScrollPlugin,
                InputDispatchPlugin,
                DirectionalNavigationPlugin,
                item::ItemsPlugin,
            ))
            // This resource is canonically used to track whether or not to render a focus indicator
            // It starts as false, but we set it to true here as we would like to see the focus indicator
            .insert_resource(InputFocusVisible(true))
            // We're turning inputs into actions first, then using those actions to determine navigation
            .add_systems(
                Update,
                (
                    invert_focused_text,
                    // We need to show which button is currently focused
                    highlight_focused_element,
                    // We're doing a tiny animation when the button is interacted with,
                    // so we need a timer and a polling mechanism to reset it
                    reset_button_after_interaction,
                ),
            )
            // This observer is added globally, so it will respond to *any* trigger of the correct type.
            // However, we're filtering in the observer's query to only respond to button presses
            .add_observer(universal_button_click_behavior)
            .add_observer(focus_on_hover)
            .add_input_context::<input::InventoryContext>()
            .add_systems(OnEnter(crate::PlayingState::Paused), setup_ui)
            .add_systems(OnExit(crate::PlayingState::Paused), teardown_ui)
            .add_observer(pause::bind)
            .add_observer(pause::pause)
            .add_observer(input::bind)
            .add_observer(input::unpause)
            .add_observer(input::navigate)
            .add_observer(input::interact_with_focused_button);
    }
}

/// The root inventory UI node.
#[derive(Component)]
pub struct InventoryRoot;

#[derive(Component)]
pub struct InventorySlot;

// This observer will be triggered whenever a button is pressed
// In a real project, each button would also have its own unique behavior,
// to capture the actual intent of the user
fn universal_button_click_behavior(
    mut trigger: Trigger<Pointer<Click>>,
    mut button_query: Query<&mut ResetTimer>,
) {
    let button_entity = trigger.target();
    if let Ok(mut reset_timer) = button_query.get_mut(button_entity) {
        // This would be a great place to play a little sound effect too!
        // color.0 = PRESSED_BUTTON.into();
        reset_timer.0 = Timer::from_seconds(0.3, TimerMode::Once);

        // Picking events propagate up the hierarchy,
        // so we need to stop the propagation here now that we've handled it
        trigger.propagate(false);
    }
}

/// Resets a UI element to its default state when the timer has elapsed.
#[derive(Component, Default, Deref, DerefMut)]
struct ResetTimer(Timer);

fn reset_button_after_interaction(
    time: Res<Time>,
    mut query: Query<(&mut ResetTimer, &mut BackgroundColor)>,
) {
    for (mut reset_timer, mut _color) in query.iter_mut() {
        reset_timer.tick(time.delta());
        if reset_timer.just_finished() {
            // color.0 = NORMAL_BUTTON.into();
        }
    }
}

fn teardown_ui(
    player: Single<Entity, With<Player>>,
    inventory: Single<Entity, With<InventoryRoot>>,
    mut commands: Commands,
    mut directional_nav_map: ResMut<DirectionalNavigationMap>,
) {
    directional_nav_map.clear();
    commands
        .entity(*player)
        .insert(Actions::<PlayerContext>::default());
    commands.entity(*inventory).despawn();
}

// We're spawning a simple grid of buttons and some instructions
// The buttons are just colored rectangles with text displaying the button's name
fn setup_ui(
    mut commands: Commands,
    mut directional_nav_map: ResMut<DirectionalNavigationMap>,
    mut input_focus: ResMut<InputFocus>,

    // grab the items from the inventory
    inventory: Single<Option<&Children>, With<item::Inventory>>,
    items: Query<&item::InventoryItem>,

    server: Res<AssetServer>,
) -> Result {
    // const N_ROWS: u16 = 3;
    const N_COLS: u16 = 3;

    // Create a full-screen background node
    let root_node = commands
        .spawn((
            InventoryRoot,
            Actions::<input::InventoryContext>::default(),
            Node {
                margin: UiRect::AUTO,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(25.0),
                ..default()
            },
        ))
        .id();

    let header_node = commands
        .spawn((
            BackgroundColor(Color::BLACK),
            BorderColor(Color::WHITE),
            Node {
                width: Val::Px(708.0),
                height: Val::Px(75.0),
                border: UiRect::all(Val::Px(4.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                ..default()
            },
            children![(
                Text::new("Pockets"),
                TextFont {
                    font: server.load("fonts/raster-forge.ttf"),
                    ..Default::default()
                },
                InvertOnFocus,
            )],
        ))
        .id();

    let outer_content_node = commands
        .spawn((
            BackgroundColor(Color::BLACK),
            BorderColor(Color::WHITE),
            Node {
                width: Val::Px(708.0),
                height: Val::Px(408.0),
                padding: UiRect::horizontal(Val::Px(50.0)),
                border: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .id();

    let num_items = inventory.map(|c| c.len()).unwrap_or_default() as u16;
    let n_rows = num_items.div_ceil(N_COLS);

    let content_node = commands
        .spawn((
            scroll::VerticalScroll::new(n_rows as usize, 200.0),
            BackgroundColor(Color::BLACK),
            BorderColor(Color::WHITE),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
        ))
        .id();

    // Set up the root entity to hold the grid
    let grid_root_entity = commands
        .spawn((Node {
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::auto(N_COLS),
            grid_template_rows: RepeatedGridTrack::auto(n_rows),
            justify_self: JustifySelf::Start,
            align_self: AlignSelf::Start,
            row_gap: Val::ZERO,
            column_gap: Val::ZERO,
            ..default()
        },))
        .id();

    let scroll_up = commands.spawn(scroll::up(content_node, &server)).id();
    let scroll_down = commands.spawn(scroll::down(content_node, &server)).id();
    commands
        .entity(outer_content_node)
        .add_children(&[scroll_up, content_node, scroll_down]);

    // Add the instructions and grid to the root node
    commands
        .entity(root_node)
        .add_children(&[header_node, outer_content_node]);

    commands.entity(content_node).add_child(grid_root_entity);

    let mut item_iter = inventory.iter().flat_map(|i| i.iter());

    let mut button_entities: HashMap<(u16, u16), Entity> = HashMap::default();
    for row in 0..n_rows {
        for col in 0..N_COLS {
            let Some(item) = item_iter.next() else {
                break;
            };
            let item = items.get(item)?;

            let button_entity = commands
                .spawn(inventory_slot(item.name.clone(), &server))
                .id();

            // Add the button to the grid
            commands.entity(grid_root_entity).add_child(button_entity);

            // Keep track of the button entities so we can set up our navigation graph
            button_entities.insert((row, col), button_entity);
        }
    }

    // Connect all of the buttons in the same row to each other,
    // looping around when the edge is reached.
    for row in 0..n_rows {
        let entities_in_row: Vec<Entity> = (0..N_COLS)
            .filter_map(|col| button_entities.get(&(row, col)))
            .copied()
            .collect();
        directional_nav_map.add_looping_edges(&entities_in_row, CompassOctant::East);
    }

    // Connect all of the buttons in the same column to each other,
    // but don't loop around when the edge is reached.
    // While looping is a very reasonable choice, we're not doing it here to demonstrate the different options.
    for col in 0..N_COLS {
        let entities_in_column: Vec<Entity> = (0..n_rows)
            .filter_map(|row| button_entities.get(&(row, col)))
            .copied()
            .collect();

        directional_nav_map.add_edges(&entities_in_column, CompassOctant::South);
    }

    // When changing scenes, remember to set an initial focus!
    if let Some(top_left_entity) = button_entities.get(&(0, 0)) {
        input_focus.set(*top_left_entity);
    }

    Ok(())
}

fn inventory_slot(name: impl Into<String>, server: &AssetServer) -> impl Bundle {
    let button_name = name.into();
    (
        Button,
        InventorySlot,
        Node {
            width: Val::Px(200.0),
            height: Val::Px(150.0),
            // // Add a border so we can show which element is focused
            // border: UiRect::all(Val::Px(4.0)),
            // Center the button's text label
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            // Center the button within the grid cell
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            ..default()
        },
        ResetTimer::default(),
        BackgroundColor::from(Color::NONE),
        Name::new(button_name.clone()),
        children![(
            Text::new(button_name),
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
            TextFont {
                font: server.load("fonts/raster-forge.ttf"),
                ..Default::default()
            },
            InvertOnFocus,
        )],
    )
}

/// A marker indicating that this UI element should invert its
/// primary color when it or some ancestor is focused.
#[derive(Component)]
struct InvertOnFocus;

fn invert_focused_text(
    input_focus: Res<InputFocus>,
    ancestors: Query<&ChildOf>,
    mut text: Query<(Entity, &mut TextColor), With<InvertOnFocus>>,
) {
    for (text_entity, mut text_color) in text.iter_mut() {
        if input_focus.0 == Some(text_entity)
            || ancestors
                .iter_ancestors(text_entity)
                .any(|a| input_focus.0 == Some(a))
        {
            text_color.0 = TEXT_FOCUSED;
        } else {
            text_color.0 = TEXT_NORMAL;
        }
    }
}

const TEXT_NORMAL: Color = Color::WHITE;
const TEXT_FOCUSED: Color = Color::BLACK;

fn highlight_focused_element(
    input_focus: Res<InputFocus>,
    // While this isn't strictly needed for the example,
    // we're demonstrating how to be a good citizen by respecting the `InputFocusVisible` resource.
    input_focus_visible: Res<InputFocusVisible>,
    mut query: Query<(Entity, &mut BackgroundColor), With<InventorySlot>>,
) {
    for (entity, mut background_color) in query.iter_mut() {
        if input_focus.0 == Some(entity) && input_focus_visible.0 {
            background_color.0 = TEXT_NORMAL;
        } else {
            background_color.0 = Color::NONE;
        }
    }
}

fn focus_on_hover(
    trigger: Trigger<Pointer<Over>>,
    focus_targets: Query<(), With<InventorySlot>>,
    mut input_focus: ResMut<InputFocus>,
) {
    if focus_targets.get(trigger.target()).is_err() {
        return;
    }

    input_focus.set(trigger.target());
}
