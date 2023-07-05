use egui::{Color32, RichText, Label};

use crate::pokemon::PokemonTyping;

// A wrapper that allows the more idiomatic usage pattern: `ui.add(typing(&typing))`
pub fn typing_widget(typing: &PokemonTyping) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| typing_ui(ui, typing)
}

pub fn typing_ui(ui: &mut egui::Ui, typing: &PokemonTyping) -> egui::Response {
    let layout = egui::Layout::left_to_right(egui::Align::Center);
    let response = ui.allocate_ui_with_layout(ui.available_size(), layout, |ui| {
        ui.add(pokemon_type(typing.type0.clone()));
        match typing.type1.clone() {
            Some(type1) => {
                ui.add(pokemon_type(type1));
            }
            None => {}
        }
    });

    response.response
}

pub fn pokemon_type(pokemon_type: String) -> impl egui::Widget + 'static {
    move |ui: &mut egui::Ui| type_ui(ui, pokemon_type)
}

pub fn type_ui(ui: &mut egui::Ui, pokemon_type: String) -> egui::Response {
    let color = match pokemon_type.as_str() {
        "Bug" => {Color32::from_rgb(170, 187, 34)},
        "Fire" => {Color32::from_rgb(255, 68, 34)},
        "Dark" => {Color32::from_rgb(119, 85, 68)},
        "Ice" => {Color32::from_rgb(102, 204, 255)},
        "Rock" => {Color32::from_rgb(187, 170, 102)},
        "Grass" => {Color32::from_rgb(119, 204, 85)},
        "Water" => {Color32::from_rgb(51, 153, 255)},
        "Fairy" => {Color32::from_rgb(238, 153, 238)},
        "Ghost" => {Color32::from_rgb(102, 102, 187)},
        "Steel" => {Color32::from_rgb(170, 170, 187)},
        "Poison" => {Color32::from_rgb(170, 85, 153)},
        "Ground" => {Color32::from_rgb(221, 187, 85)},
        "Normal" => {Color32::from_rgb(170, 170, 153)},
        "Dragon" => {Color32::from_rgb(119, 102, 238)},
        "Flying" => {Color32::from_rgb(136, 153, 255)},
        "Psychic" => {Color32::from_rgb(255, 85, 153)},
        "Fighting" => {Color32::from_rgb(187, 85, 68)},
        "Electric" => {Color32::from_rgb(255, 204, 51)},
        _ => {Color32::TEMPORARY_COLOR},
    };

    let (_id, rect) = ui.allocate_space([80.0,30.0].into());
    ui.painter().rect_filled(
        rect.clone(),
        5.0,
        color
    );
    let response = ui.put(rect, Label::new(RichText::new(pokemon_type.to_uppercase()).color(Color32::BLACK)));

    response
}
