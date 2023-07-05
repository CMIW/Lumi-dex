/// This library holds the structs needed to represent the pokemon basic data, this includes pokedex
/// number, pokemon species, abilities, pokemon type, stats, move and their changes
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pokemon {
    pub dex_num: u32,
    pub species: String,
    pub pokemon_type: PokemonTyping,
    pub new_pokemon_type: Option<PokemonTyping>,
    pub stats: Stats,
    pub new_stats: Option<Stats>,
    pub abilities: Vec<String>,
    pub locations: Vec<String>,
    pub lvl_up_moves: Vec<LlvUpMoves>,
    pub tms: Vec<TM>,
    pub egg_moves: Vec<String>,
}

impl Pokemon {
    pub fn get_stats(&self) -> Stats {
        match &self.new_stats {
            Some(stats) => Stats {
                hp: match stats.hp {
                    Some(hp) => Some(hp),
                    None => self.stats.hp.clone(),
                },
                atk: match stats.atk {
                    Some(atk) => Some(atk),
                    None => self.stats.atk.clone(),
                },
                def: match stats.def {
                    Some(def) => Some(def),
                    None => self.stats.def.clone(),
                },
                spa: match stats.spa {
                    Some(spa) => Some(spa),
                    None => self.stats.spa.clone(),
                },
                spd: match stats.spd {
                    Some(spd) => Some(spd),
                    None => self.stats.spd.clone(),
                },
                spe: match stats.spe {
                    Some(spe) => Some(spe),
                    None => self.stats.spe.clone(),
                },
            },
            None => self.stats.clone(),
        }
    }
}

impl fmt::Display for Pokemon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut display = Vec::new();
        display.push(format!(
            "{:03} {} {}",
            self.dex_num, self.species, self.pokemon_type
        ));

        if let Some(new_pokemon_type) = &self.new_pokemon_type {
            display.push(format!(" => {}", new_pokemon_type));
        }

        display.push(format!("\nAbilities: {}", self.abilities.join("/")));
        display.push(format!("\nLocations: \n{}", self.locations.join("\n")));

        // this section checks for new base stats (base stats changed for Pokemon Luminescent Platinum)
        if let Some(new_stats) = &self.new_stats {
            let mut stats = Vec::new();
            let mut old_bst = Vec::new();
            let mut new_bst = Vec::new();

            match new_stats.hp {
                Some(hp) => {
                    stats.push(format!("HP: {} => {}", &self.stats.hp.unwrap(), &hp));
                    old_bst.push(self.stats.spa.unwrap());
                    new_bst.push(hp);
                }
                None => {
                    stats.push(format!("HP: {}", self.stats.hp.unwrap()));
                    old_bst.push(self.stats.hp.unwrap());
                    new_bst.push(self.stats.hp.unwrap());
                }
            }

            match new_stats.atk {
                Some(atk) => {
                    stats.push(format!("Atk: {} => {}", &self.stats.atk.unwrap(), &atk));
                    old_bst.push(self.stats.atk.unwrap());
                    new_bst.push(atk);
                }
                None => {
                    stats.push(format!("Atk: {}", self.stats.atk.unwrap()));
                    old_bst.push(self.stats.atk.unwrap());
                    new_bst.push(self.stats.atk.unwrap());
                }
            }

            match new_stats.def {
                Some(def) => {
                    stats.push(format!("Def: {} => {}", &self.stats.def.unwrap(), &def));
                    old_bst.push(self.stats.def.unwrap());
                    new_bst.push(def);
                }
                None => {
                    stats.push(format!("Def: {}", self.stats.def.unwrap()));
                    old_bst.push(self.stats.def.unwrap());
                    new_bst.push(self.stats.def.unwrap());
                }
            }

            match new_stats.spa {
                Some(spa) => {
                    stats.push(format!("SpA: {} => {}", &self.stats.spa.unwrap(), &spa));
                    old_bst.push(self.stats.spa.unwrap());
                    new_bst.push(spa);
                }
                None => {
                    stats.push(format!("SpA: {}", self.stats.spa.unwrap()));
                    old_bst.push(self.stats.spa.unwrap());
                    new_bst.push(self.stats.spa.unwrap());
                }
            }

            match new_stats.spd {
                Some(spd) => {
                    stats.push(format!("SpD: {} => {}", &self.stats.spd.unwrap(), &spd));
                    old_bst.push(self.stats.spd.unwrap());
                    new_bst.push(spd);
                }
                None => {
                    stats.push(format!("SpD: {}", self.stats.spd.unwrap()));
                    old_bst.push(self.stats.spd.unwrap());
                    new_bst.push(self.stats.spd.unwrap());
                }
            }

            match new_stats.spe {
                Some(spe) => {
                    stats.push(format!("Spe: {} => {}", &self.stats.spe.unwrap(), &spe));
                    old_bst.push(self.stats.spe.unwrap());
                    new_bst.push(spe);
                }
                None => {
                    stats.push(format!("Spe: {}", self.stats.spe.unwrap()));
                    old_bst.push(self.stats.spe.unwrap());
                    new_bst.push(self.stats.spe.unwrap());
                }
            }

            // sum up base stats
            let old_sum = old_bst.iter().fold(0, |acc, val| acc + val);

            let new_sum = new_bst.iter().fold(0, |acc, val| acc + val);

            stats.push(format!("BST: {} => {}", old_sum, new_sum));

            display.push(format!("\nStats: {}", stats.join(", ")));
        } else {
            display.push(format!("\nStats: {}", self.stats));
        }

        display.push(format!(
            "\n\nLevel Up:\n{}",
            self.lvl_up_moves
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join("\n")
        ));

        display.push(format!(
            "\n\nTMs:\n{}",
            self.tms
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join("\n")
        ));

        display.push(format!("\n\nEgg Moves: \n{}", self.egg_moves.join("\n")));

        // join all display formats and call the write macro
        write!(f, "{}", display.join(""))
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct Stats {
    pub hp: Option<u16>,
    pub atk: Option<u16>,
    pub def: Option<u16>,
    pub spa: Option<u16>,
    pub spd: Option<u16>,
    pub spe: Option<u16>,
}

impl Stats {
    /// This function is used to validate if all the stats, of the new stat changes, are "None"
    /// and collapse the object into None instead of storing the entire object
    pub fn validate(self) -> Option<Self> {
        let fields = [
            &self.hp, &self.atk, &self.def, &self.spa, &self.spd, &self.spe,
        ];
        if fields.iter().all(|f| f.is_some()) {
            None
        } else {
            Some(self)
        }
    }
}

impl Into<Vec<(&'static str, Option<u16>)>> for Stats {
    fn into(self) -> Vec<(&'static str, Option<u16>)> {
        vec![
            ("HP", self.hp),
            ("Attack", self.atk),
            ("Defense", self.def),
            ("Sp. Atk", self.spa),
            ("Sp. Def", self.spd),
            ("Speed", self.spe),
        ]
    }
}

impl Into<Vec<(&'static str, Option<u16>)>> for &Stats {
    fn into(self) -> Vec<(&'static str, Option<u16>)> {
        vec![
            ("HP", self.hp),
            ("Attack", self.atk),
            ("Defense", self.def),
            ("Sp. Atk", self.spa),
            ("Sp. Def", self.spd),
            ("Speed", self.spe),
        ]
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut stats = Vec::new();
        if let Some(hp) = self.hp {
            stats.push(format!("HP: {}", hp));
        }
        if let Some(atk) = self.atk {
            stats.push(format!("Atk: {}", atk));
        }
        if let Some(def) = self.def {
            stats.push(format!("Def: {}", def));
        }
        if let Some(spa) = self.spa {
            stats.push(format!("SpA: {}", spa));
        }
        if let Some(spd) = self.spd {
            stats.push(format!("SpD: {}", spd));
        }
        if let Some(spe) = self.spe {
            stats.push(format!("Spe: {}", spe));
        }

        let fields = [
            &self.hp, &self.atk, &self.def, &self.spa, &self.spd, &self.spe,
        ];
        if fields.iter().all(|f| f.is_some()) {
            let sum = fields
                .iter()
                .filter_map(|&f| *f)
                .fold(0, |acc, val| acc + val);
            stats.push(format!("BST: {}", sum));
        }
        write!(f, "{}", stats.join(", "))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PokemonTyping {
    pub type0: String,
    pub type1: Option<String>,
}

impl fmt::Display for PokemonTyping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.type1 {
            Some(type1) => {
                write!(f, "{}/{}", self.type0, type1)
            }
            None => {
                write!(f, "{}", self.type0)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TM {
    pub num: u16,
    pub name: String,
}

impl fmt::Display for TM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TM{} {}", self.num, self.name)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlvUpMoves {
    pub lvl: u8,
    pub name: String,
}

impl fmt::Display for LlvUpMoves {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lvl{} {}", self.lvl, self.name)
    }
}
