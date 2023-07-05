/// This library holds the parsing funtions used to convert the pokemon data form strings in a file
/// into rust structs that can be manipulated. The parser starts on the function "parse_pokedex".
use nom::bytes::complete::{take, take_until, take_while1};
use nom::character::complete::{char, digit1};
use nom::combinator::{map_res, rest};
use nom::sequence::separated_pair;
use nom::{branch::alt,IResult};
use anyhow::{anyhow, Result};
use std::str::FromStr;

use crate::pokemon::*;

pub fn parse_pokedex(input: &str) -> Result<(&str, Vec<Pokemon>)> {
    let mut input = input;
    let mut pokedex = vec![];
    loop {
        if input.is_empty() {
            break;
        }
        let (output, result) = pokemon(input).map_err(|e| anyhow!("{}", e))?;
        pokedex.push(result);
        input = output;
    }
    Ok((input, pokedex))
}

/// The pokedex entry has the following format:
/// 001- Bulbasaur
/// Stats: 45 HP/49 Atk/49 Def/65 SpA/65 SpD/45 Spe/318 BST
/// Type: Grass/Poison
/// Abilities: Chlorophyll/Overgrow/Grassy Surge
/// Location:
/// * Jubilife City Pokémon Center (Gift)
/// Level Up:
/// 1 - Tackle
/// TMs:
/// TM06: Toxic
/// Egg Moves:
/// Skull Bash

/// With the following variations:
/// 006-Charizard
/// Stats: 78 HP/84 Atk/78 Def/109>110 SpA/85 SpD/100 Spe/534>535 BST
/// Type: Fire/Flying>Fire/Dragon
/// Abilities:Levitate/Blaze/Drought
/// Location:
/// * Evolve Charmeleon (Level 36)
/// Level Up:
/// 1 - Dragon Claw
/// TMs:
/// TM01: Focus Punch
/// Egg Moves:
/// Belly Drum

/// 151-Mew
/// Stats: 100 HP/100 Atk/100 Def/100 SpA/100 SpD/100 Spe/600 BST
/// Type: Psychic
/// Abilities: Synchronize/Synchronize/Trace
/// Wild Held Items: Lum Berry (95%)
/// Location:
///       * Route 201 (Static Encounter)
/// Level Up:
/// 1 - Reflect TypeTMs:
/// TM01: Focus Punch
/// Egg Moves:
pub fn pokemon(input: &str) -> IResult<&str, Pokemon> {
    let (input, dex_num) = dex_num(input)?;
    let (input, species) = species(input)?;
    let (input, (old_stats, new_stats)) = stats(input)?;
    let (input, (old_type, new_type)) = p_type(input)?;
    let (input, abilities) = ability(input)?;
    let (input, locations) = location(input)?;
    let (input, moves) = moves(input)?;
    let (input, tm_moves) = tm_moves(input)?;
    let (input, egg_moves) = egg_moves(input)?;
    let pokemon = Pokemon {
        dex_num: dex_num,
        species: species.trim().to_string(),
        pokemon_type: old_type,
        new_pokemon_type: new_type,
        stats: old_stats,
        new_stats: new_stats,
        abilities: abilities,
        locations: locations,
        lvl_up_moves: moves,
        tms: tm_moves,
        egg_moves: egg_moves,
    };

    Ok((input, pokemon))
}

pub fn location(input: &str) -> IResult<&str, Vec<String>> {
    let (input, _) = take_until("Location")(input)?;
    let (input, result) = take_until("Level Up")(input)?;

    let locations = result.replace("Location:", "");
    let locations: Vec<String> = locations
        .lines()
        .filter_map(|s| {
            if !s.is_empty() {
                Some(s.replace("*", "").trim().to_string())
            } else {
                None
            }
        })
        .collect();

    Ok((input, locations))
}

pub fn dex_num(input: &str) -> IResult<&str, u32> {
    let mut input = input;
    let mut result = "";
    loop {
        let ch = input.chars().next().unwrap();
        if ch == '-' {
            break;
        }
        let (inputl, resultl) = alt((
            take_while1(|c: char| !c.is_ascii_digit()),
            take_while1(|c: char| c.is_ascii_digit()),
        ))(input)?;
        input = inputl;
        result = resultl;
    }
    let (_, result) = map_res(digit1, u32::from_str)(result)?;

    Ok((input, result))
}

pub fn species(input: &str) -> IResult<&str, String> {
    let (input, _) = take_while1(|c: char| !c.is_alphabetic())(input)?;
    let (input, result) = take_until("\n")(input)?;

    Ok((input, result.to_string()))
}

pub fn hp(input: &str) -> IResult<&str, (u16, Option<u16>)> {
    let (input, result) = take_until("/")(input)?;
    let (input, _) = take(1usize)(input)?;

    let (_, result) = take_until("HP")(result)?;
    let output = match result.contains(">") {
        true => {
            let (_, (old, new)) = separated_pair(digit1, char('>'), digit1)(result)?;
            let (_, old) = map_res(digit1, u16::from_str)(old)?;
            let (_, new) = map_res(digit1, u16::from_str)(new)?;
            (old, Some(new))
        }
        false => {
            let (_, result) = digit1(result)?;
            let (_, result) = map_res(digit1, u16::from_str)(result)?;
            (result, None)
        }
    };

    Ok((input, output))
}

pub fn atk(input: &str) -> IResult<&str, (u16, Option<u16>)> {
    let (input, result) = take_until("/")(input)?;
    let (input, _) = take(1usize)(input)?;

    let (_, result) = take_until("Atk")(result)?;
    let output = match result.contains(">") {
        true => {
            let (_, (old, new)) = separated_pair(digit1, char('>'), digit1)(result)?;
            let (_, old) = map_res(digit1, u16::from_str)(old)?;
            let (_, new) = map_res(digit1, u16::from_str)(new)?;
            (old, Some(new))
        }
        false => {
            let (_, result) = digit1(result)?;
            let (_, result) = map_res(digit1, u16::from_str)(result)?;
            (result, None)
        }
    };

    Ok((input, output))
}

pub fn def(input: &str) -> IResult<&str, (u16, Option<u16>)> {
    let (input, result) = take_until("/")(input)?;
    let (input, _) = take(1usize)(input)?;

    let (_, result) = take_until("Def")(result)?;
    let output = match result.contains(">") {
        true => {
            let (_, (old, new)) = separated_pair(digit1, char('>'), digit1)(result)?;
            let (_, old) = map_res(digit1, u16::from_str)(old)?;
            let (_, new) = map_res(digit1, u16::from_str)(new)?;
            (old, Some(new))
        }
        false => {
            let (_, result) = digit1(result)?;
            let (_, result) = map_res(digit1, u16::from_str)(result)?;
            (result, None)
        }
    };

    Ok((input, output))
}

pub fn spa(input: &str) -> IResult<&str, (u16, Option<u16>)> {
    let (input, result) = take_until("/")(input)?;
    let (input, _) = take(1usize)(input)?;

    let (_, result) = take_until("SpA")(result)?;
    let output = match result.contains(">") {
        true => {
            let (_, (old, new)) = separated_pair(digit1, char('>'), digit1)(result)?;
            let (_, old) = map_res(digit1, u16::from_str)(old)?;
            let (_, new) = map_res(digit1, u16::from_str)(new)?;
            (old, Some(new))
        }
        false => {
            let (_, result) = digit1(result)?;
            let (_, result) = map_res(digit1, u16::from_str)(result)?;
            (result, None)
        }
    };

    Ok((input, output))
}

pub fn spd(input: &str) -> IResult<&str, (u16, Option<u16>)> {
    let (input, result) = take_until("/")(input)?;
    let (input, _) = take(1usize)(input)?;

    let (_, result) = take_until("SpD")(result)?;
    let output = match result.contains(">") {
        true => {
            let (_, (old, new)) = separated_pair(digit1, char('>'), digit1)(result)?;
            let (_, old) = map_res(digit1, u16::from_str)(old)?;
            let (_, new) = map_res(digit1, u16::from_str)(new)?;
            (old, Some(new))
        }
        false => {
            let (_, result) = digit1(result)?;
            let (_, result) = map_res(digit1, u16::from_str)(result)?;
            (result, None)
        }
    };

    Ok((input, output))
}

pub fn spe(input: &str) -> IResult<&str, (u16, Option<u16>)> {
    let (input, result) = match input.contains("/") {
        true => {
            let (input, result) = take_until("/")(input)?;
            let (input, _) = take(1usize)(input)?;
            (input, result)
        }
        false => ("", input),
    };

    let (_, result) = take_until("Spe")(result)?;
    let output = match result.contains(">") {
        true => {
            let (_, (old, new)) = separated_pair(digit1, char('>'), digit1)(result)?;
            let (_, old) = map_res(digit1, u16::from_str)(old)?;
            let (_, new) = map_res(digit1, u16::from_str)(new)?;
            (old, Some(new))
        }
        false => {
            let (_, result) = digit1(result)?;
            let (_, result) = map_res(digit1, u16::from_str)(result)?;
            (result, None)
        }
    };

    Ok((input, output))
}

pub fn bst(input: &str) -> IResult<&str, (String, Option<String>)> {
    let (_, result) = take_until("BST")(input)?;
    let output = match result.contains(">") {
        true => {
            let (_, (old, new)) = separated_pair(digit1, char('>'), digit1)(result)?;
            (old.to_string(), Some(new.to_string()))
        }
        false => {
            let (_, result) = digit1(result)?;
            (result.to_string(), None)
        }
    };

    Ok((input, output))
}

pub fn stats(input: &str) -> IResult<&str, (Stats, Option<Stats>)> {
    let (input, _) = take_until("Stats")(input)?;
    let (output, result) = take_until("Type")(input)?;
    let (input, _) = take_while1(|c: char| !c.is_ascii_digit())(result)?;

    let (input, (hp, op_hp)) = hp(input)?;
    let (input, (atk, op_atk)) = atk(input)?;
    let (input, (def, op_def)) = def(input)?;
    let (input, (spa, op_spa)) = spa(input)?;
    let (input, (spd, op_spd)) = spd(input)?;
    let (input, (spe, op_spe)) = spe(input)?;
    let (_, _bst) = match input.contains("BST") {
        true => bst(input)?,
        false => ("", (String::from(""), None)),
    };

    let stats = Stats {
        hp: Some(hp),
        atk: Some(atk),
        def: Some(def),
        spa: Some(spa),
        spd: Some(spd),
        spe: Some(spe),
    };
    let new_stats = Stats {
        hp: op_hp,
        atk: op_atk,
        def: op_def,
        spa: op_spa,
        spd: op_spd,
        spe: op_spe,
    };

    Ok((output, (stats, new_stats.validate())))
}

pub fn p_type(input: &str) -> IResult<&str, (PokemonTyping, Option<PokemonTyping>)> {
    let (input, _) = take_until("Type")(input)?;
    let (input, result) = take_until("Abilities")(input)?;

    let output = match result.contains(">") {
        true => {
            let (output, old) = take_until(">")(result)?;
            let (new, _) = take(1usize)(output)?;

            let (_, old) = typing(old)?;
            let (_, new) = typing(new)?;

            let test = (old, Some(new));
            test
        }
        false => {
            let (_, p_type) = typing(result)?;
            (p_type, None)
        }
    };

    Ok((input, output))
}

pub fn typing(input: &str) -> IResult<&str, PokemonTyping> {
    let (type1, type0) = alt((take_until("/"), rest))(input.trim())?;
    let (type1, _) = alt((take(1usize), rest))(type1)?;
    let type1 = match type1 {
        "" => None,
        s => Some(s.to_string()),
    };
    let type0 = type0.replace("Type:", "").trim().to_string();

    let result = PokemonTyping {
        type0: type0,
        type1: type1,
    };

    Ok((input, result))
}

pub fn ability(input: &str) -> IResult<&str, Vec<String>> {
    let (input, _) = take_until("Abilities")(input)?;
    let (_, contains) = contains_item(input)?;
    let (input, result) = match contains {
        true => take_until("Wild Held Items")(input)?,
        false => take_until("Location")(input)?,
    };

    let abilities: Vec<String> = result
        .replace("Abilities:", "")
        .trim()
        .split("/")
        .map(|s| s.to_string())
        .collect();

    Ok((input, abilities))
}

/// This function is used to check if the pokemon has a "Wild Held Items" field
/// It was abstracted to it's own function in the case that "Wild Held Items" is added to the
/// pokemon struct
pub fn contains_item(input: &str) -> IResult<&str, bool> {
    let (_, result) = take_until("Location")(input)?;
    let result = result.contains("Wild Held Items");
    Ok((input, result))
}

pub fn moves(input: &str) -> IResult<&str, Vec<LlvUpMoves>> {
    let (input, _) = take_until("Level Up")(input)?;
    let (input, result) = take_until("TMs")(input)?;

    let moves = result.trim().replace("Level Up:", "");
    let moves: Vec<LlvUpMoves> = moves
        .lines()
        .filter_map(|s| {
            if !s.is_empty() {
                let s: Vec<String> = s.split(":").map(|s| s.to_string()).collect();
                Some(LlvUpMoves {
                    lvl: s[0].trim().parse().ok()?,
                    name: s[1].trim().to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok((input, moves))
}

pub fn tm_moves(input: &str) -> IResult<&str, Vec<TM>> {
    let (input, _) = take_until("TMs")(input)?;
    let (input, result) = take_until("Egg Moves")(input)?;

    let moves = result.replace("TMs:", "");
    let moves: Vec<TM> = moves
        .lines()
        .filter_map(|s| {
            if !s.is_empty() {
                let s: Vec<String> = s.split(":").map(|s| s.to_string()).collect();
                Some(TM {
                    num: s[0].replace("TM", "").parse().ok()?,
                    name: s[1].trim().to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok((input, moves))
}

pub fn egg_moves(input: &str) -> IResult<&str, Vec<String>> {
    let (input, _) = take_until("Egg Moves")(input)?;
    let (input, result) = take_while1(|c: char| !c.is_ascii_digit())(input)?;

    let moves = result.replace("Egg Moves:", "");
    let moves: Vec<String> = moves
        .lines()
        .filter_map(|s| {
            if !s.is_empty() {
                Some(s.to_string())
            } else {
                None
            }
        })
        .collect();

    Ok((input, moves))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_test() {
        let example = r#"029- Nidoran♀
        Stats:"#;
        match species(example) {
            Ok((_,species)) => {
                println!("{}", species);
            },
            _ => {},
        }
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
