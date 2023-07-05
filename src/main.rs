use egui::{RichText, FontFamily::*, FontId, TextStyle, ScrollArea, vec2};
use surrealdb::{Surreal, engine::local::{Db, File}};
use std::{borrow::BorrowMut, env, fs, error::Error};
use egui_extras::{image::RetainedImage};
use poll_promise::Promise;
use anyhow::Result;
use eframe::egui;
use clap::Parser;

use lumi_dex::{stats_bar, typing_widget, Pokemon, parser::*};

static DB: Surreal<Db> = Surreal::init();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    DB.connect::<File>("temp.db").await?;
    DB.use_ns("Luminescent").use_db("Pokedex").await?;

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
    /*let pokedex = load_pokedex()?;
    for pokemon in pokedex {
        println!("{}",pokemon);
    }*/
    Ok(())
}

fn load_pokedex() -> Result<Vec<Pokemon>> {
    let files = [
        "pokedex/Lumi Pokédex 001-151 Kanto Pokémon.txt",
        "pokedex/Lumi Pokédex 152-251 Johto Pokémon.txt",
        "pokedex/Lumi Pokédex 252-386 Hoenn Pokémon.txt",
        "pokedex/Lumi Pokédex 387-493 Sinnoh Pokémon.txt",
        "pokedex/Lumi Pokédex xxx 2.0 Add-Ons + Form Pokémon.txt"
    ];

    let mut pokedex = Vec::new();

    for file in files {
        let mut path = env::current_dir()?;
        path.push(file);
        let pokedex_string = fs::read_to_string(path)?;
        let (_, mut parsed_pokedex) = parse_pokedex(&pokedex_string)?;
        pokedex.append(&mut parsed_pokedex);
    }

    Ok(pokedex)
}

async fn store_pokedex() -> Result<()> {
    let pokedex = load_pokedex()?;

    for mut pokemon in pokedex {
        match find_pokemon(&pokemon.species).await? {
            Some(found) => {println!("{}", found.species);},
            None => {
				pokemon.species = pokemon.species
                .replace("Cloak","")
                .replace("Form","")
                .replace("-A"," Alolan")
                .replace("-G"," Galarian")
                .replace("-H"," Hisuian")
                .trim().to_string();
				let _created: Pokemon = DB.create("pokemon").content(pokemon).await?;
			},
        }
    }
    Ok(())
}

async fn get_image(pokemon: &str) -> Result<RetainedImage> {
    let url = format!("https://img.pokemondb.net/artwork/large/{}.jpg", pokemon.to_lowercase());
    let bytes = reqwest::get(&url).await?.bytes().await?;
    let image = RetainedImage::from_image_bytes(&url, &bytes).map_err(anyhow::Error::msg);
    image
}

async fn find_pokemon(species: &str) -> Result<Option<Pokemon>> {
    let mut response = DB
        .query(
            r#"
        SELECT
            dex_num, species, pokemon_type, new_pokemon_type,
            stats, new_stats, abilities, lvl_up_moves, tms, egg_moves, locations
        FROM pokemon
        WHERE string::lowercase(species) = string::lowercase($species)
        "#,
        )
        .bind(("species", species))
        .await?;
    let pokemon: Option<Pokemon> = response.take(0)?;
    Ok(pokemon)
}

async fn find_by_move(attack: &str) -> Result<Vec<Pokemon>> {
    let mut response = DB
        .query(
            r#"
        SELECT
            dex_num, species, pokemon_type, new_pokemon_type,
            stats, new_stats, abilities, lvl_up_moves, tms, egg_moves, locations
        FROM pokemon
        WHERE string::lowercase(lvl_up_moves.*.name) CONTAINS string::lowercase($value)
		OR string::lowercase(tms.*.name) CONTAINS string::lowercase($value)
		OR string::lowercase(egg_moves) CONTAINS string::lowercase($value)
		ORDER BY dex_num ASC
        "#,
        )
        .bind(("value", attack))
        .await?;

    let pokemons: Vec<Pokemon> = response.take(0)?;

    Ok(pokemons)
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

#[derive(Default)]
struct MyApp {
    /// `None` when download hasn't started yet.
    searched: bool,
    search_text: String,
    promise: Option<Promise<Result<RetainedImage>>>,
    pokemon: Option<Promise<Result<Option<Pokemon>>>>,
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

        Self {
            search_text: "Search".to_owned(),
            ..Default::default()
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let pokemon_promise = self.pokemon.get_or_insert_with(|| Promise::spawn_async(async move {Ok(None)}));

        egui::CentralPanel::default().show(ctx, |ui| {
            let frame_size = frame.info().window_info.size;
            // Display search bar
            let search_bar_size = vec2(ui.available_width(), frame_size.y * 0.05);
            let search_bar_layout = egui::Layout::right_to_left(egui::Align::Min);
            ui.allocate_ui_with_layout(search_bar_size, search_bar_layout, |ui| {
                ui.label("🔍");
                let response = ui.add(egui::TextEdit::singleline(&mut self.search_text));

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.promise = None;
                    let search = self.search_text.clone();
                    *pokemon_promise = Promise::spawn_async(async move {
                        find_pokemon(&search).await
                    });
                    self.searched = true;
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
                    if self.searched {
                        ui.colored_label(ui.visuals().error_fg_color, format!("Did not find {}", &self.search_text));
                        self.searched = false;
                    }
                }
                Some(Ok(Some(result))) => {
                    let pokemon = result.clone();
                    let species = pokemon.species.clone().replace(" ","-").replace(".","");
                    let promise = self.promise.get_or_insert_with(|| Promise::spawn_async(async move { get_image(&species).await })).borrow_mut();

                    // Display main pokemon info
                    let general_info_size = vec2(ui.available_width(), frame_size.y * 0.45);
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
                                image.show_scaled(ui,0.46);
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
                    let moves_info_size = vec2(frame_size.x, frame_size.y * 0.49);
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
        });
    }
}
