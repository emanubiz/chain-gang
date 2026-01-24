// game_client/src/hud.rs

use bevy::prelude::*;

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthText;

#[derive(Resource)]
pub struct PlayerHealthUI {
    pub current: f32,
    pub max: f32,
}

impl Default for PlayerHealthUI {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

pub fn setup_hud(mut commands: Commands) {
    // Root UI container
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        // Health container
        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        }).with_children(|parent| {
            // Health text
            parent.spawn((
                TextBundle::from_section(
                    "HP: 100/100",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                HealthText,
            ));

            // Health bar background
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(200.0),
                    height: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                border_color: BorderColor(Color::WHITE),
                ..default()
            }).with_children(|parent| {
                // Health bar fill
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgb(0.0, 0.8, 0.0)),
                        ..default()
                    },
                    HealthBar,
                ));
            });
        });
    });
}

pub fn update_health_ui(
    health: Res<PlayerHealthUI>,
    mut health_bar_query: Query<&mut Style, (With<HealthBar>, Without<HealthText>)>,
    mut health_text_query: Query<&mut Text, With<HealthText>>,
) {
    if health.is_changed() {
        // Update bar width
        if let Ok(mut style) = health_bar_query.get_single_mut() {
            let percentage = (health.current / health.max) * 100.0;
            style.width = Val::Percent(percentage);
        }

        // Update text
        if let Ok(mut text) = health_text_query.get_single_mut() {
            text.sections[0].value = format!("HP: {:.0}/{:.0}", health.current, health.max);
        }
    }
}