// game_client/src/hud.rs

use bevy::prelude::*;

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthText;

/// Marker per il testo del danno (hit marker centrato)
#[derive(Component)]
pub struct HitMarker;

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

/// Stato del hit marker (aggiornato dalla rete quando arriva ProjectileHit)
#[derive(Resource, Default)]
pub struct HitMarkerUI {
    pub damage: f32,
    pub elapsed: f32,
    pub active: bool,
}

pub fn setup_hud(mut commands: Commands) {
    // ── Barra vita (angolo in alto a sinistra) ────────────────────────────────
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
        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        }).with_children(|parent| {
            // Testo HP
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

            // Background barra
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
                // Fill verde
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

    // ── Hit marker centrato (danno inflitto) ──────────────────────────────────
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 44.0,
                    // alpha=0 → invisibile di default
                    color: Color::srgba(1.0, 0.3, 0.0, 0.0),
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::bottom(Val::Px(60.0)), // leggermente sopra il centro
                ..default()
            }),
            HitMarker,
        ));
    });
}

pub fn update_health_ui(
    health: Res<PlayerHealthUI>,
    mut health_bar_query: Query<&mut Style, (With<HealthBar>, Without<HealthText>)>,
    mut health_text_query: Query<&mut Text, With<HealthText>>,
) {
    if health.is_changed() {
        if let Ok(mut style) = health_bar_query.get_single_mut() {
            let percentage = (health.current / health.max) * 100.0;
            style.width = Val::Percent(percentage);
        }
        if let Ok(mut text) = health_text_query.get_single_mut() {
            text.sections[0].value = format!("HP: {:.0}/{:.0}", health.current, health.max);
        }
    }
}

/// Aggiorna il testo danno: appare, poi sfuma in 0.9s
pub fn update_hit_marker(
    mut hit_marker: ResMut<HitMarkerUI>,
    mut query: Query<&mut Text, With<HitMarker>>,
    time: Res<Time>,
) {
    if !hit_marker.active {
        return;
    }

    let total = 0.9_f32;
    hit_marker.elapsed += time.delta_seconds();

    if let Ok(mut text) = query.get_single_mut() {
        let alpha = ((total - hit_marker.elapsed) / total).clamp(0.0, 1.0);
        text.sections[0].value = format!("-{:.0}", hit_marker.damage);
        text.sections[0].style.color = Color::srgba(1.0, 0.3, 0.0, alpha);
    }

    if hit_marker.elapsed >= total {
        hit_marker.active = false;
        if let Ok(mut text) = query.get_single_mut() {
            text.sections[0].style.color = Color::srgba(1.0, 0.3, 0.0, 0.0);
        }
    }
}
