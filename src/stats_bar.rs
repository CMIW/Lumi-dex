use egui::{vec2, Color32, Sense, Pos2, Rect};

use crate::pokemon::Stats;

// A wrapper that allows the more idiomatic usage pattern: `ui.add(stats_bar(&stats))`
pub fn stats_bar(stats: &Stats) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| stats_bar_ui(ui, stats)
}

pub fn stats_bar_ui(ui: &mut egui::Ui, stats: &Stats) -> egui::Response {
    let rounding = 50.0;

    let layout = egui::Layout::top_down(egui::Align::Min);
    let response = ui.allocate_ui_with_layout(ui.available_size(), layout, |ui| {
        let stats_vec: Vec<(&str, Option<u16>)> = stats.into();
        for (key, stat) in stats_vec.iter() {
            ui.horizontal(|ui| {
                let stat_value = match stat {
                    Some(value) => value,
                    // This state should never be reached
                    None => &10,
                };
                let key_label = ui.label(format!("{}", key));

                let size = vec2((*stat_value as f32) * 1.7, 10.0);
                let color = match stat_value {
                    val if val >= &150 => Color32::from_rgb(0, 194, 184),
                    val if val >= &120 => Color32::from_rgb(35, 205, 94),
                    val if val >= &90 => Color32::from_rgb(160, 229, 21),
                    val if val >= &60 => Color32::from_rgb(255, 221, 87),
                    _ => Color32::from_rgb(255, 127, 15),
                };

                let bar_rect_min = Pos2{x: key_label.rect.min.x + 75.8, y: key_label.rect.min.y + 5.5};
                let bar_rect_max = bar_rect_min + size;
                let bar_rect = Rect {min: bar_rect_min, max: bar_rect_max};

                let aloc_rect = ui.allocate_rect(bar_rect, Sense::hover());
                ui.painter().rect_filled(
                    aloc_rect.rect,
                    rounding,
                    color
                );

                ui.label(format!("{}", stat_value));
            });
        }
    });

    response.response
}
