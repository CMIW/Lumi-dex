// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use eframe::egui;
use anyhow::Result;
use poll_promise::Promise;
use egui_extras::{image::RetainedImage};
use std::{borrow::BorrowMut, fs, error::Error};
use egui_dock::{DockArea, NodeIndex, Style, Tree};
use egui::{RichText, FontFamily::*, FontId, TextStyle, ScrollArea, vec2};

use lumi_dex::{stats_bar, typing_widget, Pokemon, backend::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = connec_to_db().await?;

    // Parse the the arguments for the CLI
    let args = Args::parse();

    if args.load_dex {
        store_pokedex().await?;
    }

    if let Some(species) = args.species {
        match find_pokemon(&species).await? {
            Some(pokemon) => {
                fs::write("pokemons.txt", format!("{}", pokemon))?;
            },
            None => {println!("Could not find {}", &species);},
        }
        return Ok(());
    }

    if let Some(attack) = args.attack {
        let pokemons: Vec<Pokemon> = find_by_move(&attack).await?;
        // Map each pokemon into a string, collect that into a vector, then join each element
        let pokemons = pokemons.iter().map(|x| format!("{}",x)).collect::<Vec<String>>().join("\n\n");
        fs::write("pokemons.txt", pokemons)?;
        return Ok(());
    }

    let options = eframe::NativeOptions {
        min_window_size: Some([690.0, 880.0].into()),
        ..Default::default()
    };
    eframe::run_native(
        "Pokemon Luminescent Platinum Pokedex",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )?;

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
	#[arg(short, long)]
	pub species: Option<String>,

    #[arg(short, long)]
	pub attack: Option<String>,

    #[clap(long, short)]
	pub load_dex: bool,
}

struct MyApp {
    tree: Tree<TabContext>,
}

struct TabViewer<'a> {
    added_nodes: &'a mut Vec<NodeIndex>,
}

#[derive(Default)]
struct TabContext {
    pub searched: bool,
    pub search_text: String,
    pub pokemon: Option<Promise<Result<Option<Pokemon>>>>,
    pub pokemon_image: Option<Promise<Result<RetainedImage>>>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = TabContext;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let pokemon_promise = tab.pokemon.get_or_insert_with(|| Promise::spawn_async(async move {Ok(None)}));

        let search_bar_size = vec2(ui.available_width(), ui.available_height() * 0.05);
        let search_bar_layout = egui::Layout::right_to_left(egui::Align::Min);
        ui.allocate_ui_with_layout(search_bar_size, search_bar_layout, |ui| {
            ui.label("🔍");
            let response = ui.add(egui::TextEdit::singleline(&mut tab.search_text));
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                tab.pokemon_image = None;
                let search = tab.search_text.clone();
                *pokemon_promise = Promise::spawn_async(async move {
                    find_pokemon(&search).await
                });
                tab.searched = true;
            }
        });

        match pokemon_promise.ready(){
            None => {
                ui.horizontal_centered(|ui| {
                    ui.label("Searching....");
                    ui.spinner();
                });
            }
            Some(Err(err)) => {
                ui.colored_label(ui.visuals().error_fg_color, format!("{}", err));
            }
            Some(Ok(None)) => {
                if tab.searched {
                    ui.colored_label(ui.visuals().error_fg_color, format!("Did not find {}", &tab.search_text));
                    tab.searched = false;
                }
            }
            Some(Ok(Some(result))) => {
                let pokemon = result.clone();
                let species = pokemon.species.clone().replace(" ","-").replace(".","");
                let promise = tab.pokemon_image.get_or_insert_with(|| Promise::spawn_async(async move { get_image(&species).await })).borrow_mut();

                // Display main pokemon info
                let general_info_size = vec2(ui.available_width(), ui.available_height() * 0.45);
                let general_info_layout = egui::Layout::left_to_right(egui::Align::Center);
                ui.allocate_ui_with_layout(general_info_size, general_info_layout, |ui| {
                    match promise.ready() {
                        None => {
                            ui.spinner(); // still loading
                        }
                        Some(Err(err)) => {
                            ui.colored_label(ui.visuals().error_fg_color, format!("{}", err)); // something went wrong
                        }
                        Some(Ok(image)) => {
                            image.show_scaled(ui,0.50);
                        }
                    }

                    ui.vertical(|ui| {
                        let _dex_num_label = ui.label(format!("National № {:04}", pokemon.dex_num));
                        ui.horizontal(|ui| {
                            let _pokemon_species_label = ui.hyperlink_to(RichText::new(&pokemon.species).heading(), format!("https://pokemondb.net/pokedex/{}", &pokemon.species.to_lowercase()));
                            let _pokemon_type_label = match &pokemon.new_pokemon_type {
                                Some(new_type) => {ui.add(typing_widget(&new_type));},
                                None => {ui.add(typing_widget(&pokemon.pokemon_type));},
                            };
                        });
                        let _pokemon_abilities_label = ui.label(format!("Abilities: \n\t\t{} (hidden ability)", pokemon.abilities.join("\n\t\t")));
                        ui.add(stats_bar(&pokemon.get_stats()));
                    });
                });

                // Display pokemon moves
                let moves_info_size = vec2(ui.available_width(), ui.available_height());
                let moves_info_layout = egui::Layout::left_to_right(egui::Align::Center);
                ui.allocate_ui_with_layout(moves_info_size, moves_info_layout, |ui| {
                    ui.vertical(|ui| {
                        ui.strong("Level up moves");
                        ui.push_id(0, |ui| {
                            ScrollArea::vertical()
                            .max_height(ui.available_height())
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    for attack in pokemon.lvl_up_moves{
                                        ui.label(format!("{}",attack));
                                    }
                                });
                            });
                        });
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.strong("TM moves");
                        ui.push_id(1, |ui| {
                            ScrollArea::vertical()
                            .max_height(ui.available_height())
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    for attack in pokemon.tms{
                                        ui.label(format!("{}",attack));
                                    }
                                });
                            });
                        });
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.strong("Egg moves");
                        ui.push_id(3, |ui| {
                            ScrollArea::vertical()
                            .max_height(ui.available_height())
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    for attack in pokemon.egg_moves{
                                        ui.label(format!("{}",attack));
                                    }
                                });
                            });
                        });
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.strong("Locations");
                        ui.push_id(4, |ui| {
                            ScrollArea::vertical()
                            .max_height(ui.available_height())
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    for location in pokemon.locations{
                                        ui.label(format!("{}",location));
                                    }
                                });
                            });
                        });
                    });

                });
            }
        }
    }

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {
        format!("Tab").into()
    }

    fn on_add(&mut self, node: NodeIndex) {
        self.added_nodes.push(node);
    }
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::new(30.0, Proportional)),
            (TextStyle::Body, FontId::new(18.0, Proportional)),
            (TextStyle::Monospace, FontId::new(14.0, Proportional)),
            (TextStyle::Button, FontId::new(14.0, Proportional)),
            (TextStyle::Small, FontId::new(10.0, Proportional)),
        ].into();
        cc.egui_ctx.set_style(style);

        let tree = Tree::new(vec![TabContext::default()]);

        Self { tree }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut added_nodes = Vec::new();
        DockArea::new(&mut self.tree)
            .show_add_buttons(true)
            .draggable_tabs(false)
            .style({
                let mut style = Style::from_egui(ctx.style().as_ref());
                style.tabs.fill_tab_bar = true;
                style
            })
            .show(
                ctx,
                &mut TabViewer {
                    added_nodes: &mut added_nodes,
                },
            );

        added_nodes.drain(..).for_each(|node| {
            self.tree.set_focused_node(node);
            self.tree.push_to_focused_leaf(TabContext::default());
        });
    }
}
