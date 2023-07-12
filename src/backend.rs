use std::{env, fs};
use anyhow::Result;
use egui_extras::{image::RetainedImage};
use surrealdb::{Surreal, engine::local::{Db, File}};

use crate::{Pokemon, parser::*};

static DB: Surreal<Db> = Surreal::init();

pub async fn connec_to_db() -> Result<()> {
    DB.connect::<File>("temp.db").await?;
    DB.use_ns("Luminescent").use_db("Pokedex").await?;

    Ok(())
}

pub async fn find_pokemon(species: &str) -> Result<Option<Pokemon>> {
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

pub async fn find_by_move(attack: &str) -> Result<Vec<Pokemon>> {
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

pub async fn get_image(pokemon: &str) -> Result<RetainedImage> {
    let url = format!("https://img.pokemondb.net/artwork/large/{}.jpg", pokemon.to_lowercase());
    let bytes = reqwest::get(&url).await?.bytes().await?;
    let image = RetainedImage::from_image_bytes(&url, &bytes).map_err(anyhow::Error::msg);
    image
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

pub async fn store_pokedex() -> Result<()> {
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
