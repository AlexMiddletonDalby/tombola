use crate::settings::Settings;
use crate::size::Size;

use crate::midi;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::math::{Rect, Vec2};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use strum::IntoEnumIterator;

#[derive(Component)]
pub struct BallSelector {
    pub size: Size,
}

impl BallSelector {
    pub fn hitbox_size() -> f32 {
        Size::Large.to_radius() + 10.0
    }
}

#[derive(Bundle)]
pub struct BallSelectorBundle {
    marker: BallSelector,
    transform: Transform,
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
}

impl BallSelectorBundle {
    pub fn new(
        size: Size,
        position: Vec2,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        BallSelectorBundle {
            marker: BallSelector { size },
            transform: Transform::from_xyz(position.x, position.y, 1.0),
            mesh: Mesh2d(meshes.add(Circle::new(size.to_radius()))),
            material: MeshMaterial2d(materials.add(ColorMaterial::from_color(size.to_color()))),
        }
    }
}

#[derive(Component)]
pub struct Highlight;

#[derive(Bundle)]
pub struct HighlightBundle {
    marker: Highlight,
    transform: Transform,
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
}

impl HighlightBundle {
    pub fn new(
        position: Vec2,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        HighlightBundle {
            marker: Highlight,
            transform: Transform::from_xyz(position.x, position.y, 0.0),
            mesh: Mesh2d(meshes.add(Circle::new(BallSelector::hitbox_size()))),
            material: MeshMaterial2d(
                materials.add(ColorMaterial::from_color(Color::srgba(0.3, 0.3, 0.4, 0.2))),
            ),
        }
    }
}

pub fn find_selector_position(
    selectors: &Vec<(&BallSelector, &Transform)>,
    selected: Size,
) -> Option<Vec2> {
    if let Some((_found, transform)) = selectors
        .iter()
        .find(|(selector, _)| selector.size == selected)
    {
        return Some(transform.translation.truncate());
    }

    None
}

pub fn pick_selector(selectors: &Vec<(&BallSelector, &Transform)>, pos: Vec2) -> Option<Size> {
    for (selector, transform) in selectors.iter() {
        let centre = transform.translation.truncate();

        let rect = Rect::new(
            centre.x + BallSelector::hitbox_size(),
            centre.y + BallSelector::hitbox_size(),
            centre.x - BallSelector::hitbox_size(),
            centre.y - BallSelector::hitbox_size(),
        );

        if rect.contains(pos) {
            return Some(selector.size);
        }
    }

    None
}

pub fn show_settings_menu(mut egui: EguiContexts, settings: &mut Settings) -> bool {
    egui::Window::new("Settings")
        .default_open(false)
        .show(egui.ctx_mut(), |ui| {
            ui.collapsing("World", |ui| {
                ui.add(
                    egui::Slider::new(&mut settings.world.tombola_spin, -2.0..=2.0).text("Spin"),
                );
                ui.add(
                    egui::Slider::new(&mut settings.world.bounciness, 0.0..=1.0)
                        .text("Bounciness")
                        .fixed_decimals(2),
                );
                ui.add(
                    egui::Slider::new(&mut settings.world.gravity, 0.0..=1.5)
                        .text("Gravity")
                        .fixed_decimals(2),
                );
            });
            ui.collapsing("MIDI", |ui| {
                ui.label("Notes");
                for (index, current_note) in &mut settings.midi.tombola_notes.iter_mut().enumerate()
                {
                    egui::ComboBox::from_id_salt(index)
                        .selected_text(current_note.to_string())
                        .show_ui(ui, |ui| {
                            for note in midi::Note::iter() {
                                ui.selectable_value(current_note, note, note.to_string());
                            }
                        });
                }

                ui.add_space(10.0);

                ui.checkbox(
                    &mut settings.midi.fixed_note_velocity.enabled,
                    "Fixed Note Velocity",
                );
                if settings.midi.fixed_note_velocity.enabled {
                    ui.add(egui::Slider::new(
                        &mut settings.midi.fixed_note_velocity.value,
                        0..=127,
                    ));
                }

                ui.checkbox(
                    &mut settings.midi.fixed_note_length.enabled,
                    "Fixed Note Length",
                );
                if settings.midi.fixed_note_length.enabled {
                    ui.add(
                        egui::Slider::new(&mut settings.midi.fixed_note_length.value, 10..=1000)
                            .suffix("ms"),
                    );
                }
            });
        });

    egui.ctx_mut().is_pointer_over_area()
}
