use bevy::prelude::*;

pub struct ScrollPlugin;

impl Plugin for ScrollPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (up_visibility, down_visibility, invert_button, on_click),
                update_scroll,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
pub struct VerticalScroll {
    current_page: usize,
    total_pages: usize,
    scroll_step: f32,
}

impl VerticalScroll {
    pub fn new(total_pages: usize, page_size: f32) -> Self {
        Self {
            current_page: 0,
            total_pages,
            scroll_step: page_size,
        }
    }
}

#[derive(Component)]
struct Scroller(Entity);

#[derive(Component)]
struct Up;

#[derive(Component)]
struct Down;

#[derive(Component)]
struct InventoryButton;

fn button(content: impl Bundle, server: &AssetServer) -> impl Bundle {
    (
        Node {
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(15.0)),
            height: Val::Percent(80.0),
            ..Default::default()
        },
        Button,
        InventoryButton,
        children![(
            content,
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
            TextFont {
                font: server.load("fonts/raster-forge.ttf"),
                ..Default::default()
            },
        )],
    )
}

fn invert_button(
    mut button: Query<
        (&Interaction, &Children, &mut BackgroundColor),
        (With<InventoryButton>, Changed<Interaction>),
    >,
    mut text: Query<&mut TextColor>,
) -> Result {
    for (interaction, children, mut color) in button.iter_mut() {
        let mut text = text.get_mut(children[0])?;

        match interaction {
            Interaction::Hovered | Interaction::Pressed => {
                text.0 = super::TEXT_FOCUSED;
                color.0 = super::TEXT_NORMAL;
            }
            _ => {
                text.0 = super::TEXT_NORMAL;
                color.0 = Color::NONE;
            }
        }
    }

    Ok(())
}

fn update_scroll(mut q: Query<(&VerticalScroll, &mut ScrollPosition), Changed<VerticalScroll>>) {
    for (description, mut position) in q.iter_mut() {
        position.offset_y = description.current_page as f32 * description.scroll_step;
    }
}

fn on_click(
    button: Query<(&Interaction, &ChildOf), (With<InventoryButton>, Changed<Interaction>)>,
    parents: Query<(&Scroller, Has<Up>, Has<Down>)>,
    mut scrollers: Query<&mut VerticalScroll>,
) -> Result {
    for (interaction, parent) in button.iter() {
        if let Interaction::Pressed = interaction {
            let (scroller, up, down) = parents.get(parent.0)?;
            let mut scroller = scrollers.get_mut(scroller.0)?;

            if up {
                scroller.current_page = scroller.current_page.saturating_sub(1);
            } else if down {
                scroller.current_page = (scroller.current_page + 1).min(scroller.total_pages - 1);
            }
        }
    }

    Ok(())
}

fn up_visibility(
    mut up: Query<(&Scroller, &mut Visibility), With<Up>>,
    scroll: Query<&VerticalScroll>,
) -> Result {
    for (scroller, mut visibility) in up.iter_mut() {
        let scroll = scroll.get(scroller.0)?;

        if scroll.current_page == 0 {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }
    }

    Ok(())
}

fn down_visibility(
    mut down: Query<(&Scroller, &mut Visibility), With<Down>>,
    scroll: Query<&VerticalScroll>,
) -> Result {
    for (scroller, mut visibility) in down.iter_mut() {
        let scroll = scroll.get(scroller.0)?;

        if scroll.current_page == scroll.total_pages - 1 {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }
    }

    Ok(())
}

pub fn up(scroller: Entity, server: &AssetServer) -> impl Bundle {
    (
        Up,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(75.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        Scroller(scroller),
        children![button(Text::new("Up"), server)],
        Visibility::Hidden,
    )
}

pub fn down(scroller: Entity, server: &AssetServer) -> impl Bundle {
    (
        Down,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(75.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        Scroller(scroller),
        children![button(Text::new("Down"), server)],
        Visibility::Hidden,
    )
}
