use std::collections::{HashMap, HashSet};

use crate::warcraft_logs::combat_log::{CombatLog, EventType};

use std::fs;
use std::io;

pub fn load_spell_map(file_path: &str) -> io::Result<HashMap<i64, String>> {
    let content = fs::read_to_string(file_path)?;
    let mut spell_map = HashMap::new();

    for line in content.lines() {
        if line.starts_with("Name") {
            if let Some((id, name)) = parse_spell_line(line) {
                spell_map.insert(id, name);
            }
        }
    }

    Ok(spell_map)
}

fn parse_spell_line(line: &str) -> Option<(i64, String)> {
    // Find the "id=" part of the line
    let id_start = line.find("id=")?;
    let id_end = line[id_start + 3..].find(')')?;
    let id: i64 = line[id_start + 3..id_start + 3 + id_end].parse().ok()?;

    // Extract the name part, removing `: ` and any additional descriptors like `(desc=...)`
    let name_start = line.find(':')? + 2; // Skip the `: ` prefix
    let name_end = line[name_start..id_start]
        .trim()
        .find(" (")
        .unwrap_or(id_start - name_start);
    let name = line[name_start..name_start + name_end].trim();

    Some((id, name.to_string()))
}

const HEALER_SPECS: [i64; 7] = [
    105,  // Restoration Druid
    270,  // Mistweaver Monk
    65,   // Holy Paladin
    256,  // Discipline Priest
    257,  // Holy Priest
    264,  // Restoration Shaman
    1468, // Preservation Evoker
];

pub fn calculate_heal_correctness(combat_log: &CombatLog) -> anyhow::Result<()> {
    let spell_names = load_spell_map("generated/allspells.txt").unwrap();

    let mut health_map: HashMap<i64, i64> = HashMap::new();
    let mut correctness_map: HashMap<i64, (f64, i64)> = HashMap::new();
    let mut healer_ids: HashSet<i64> = HashSet::new();
    let mut valid_heal_spells: HashSet<String> = HashSet::new();

    valid_heal_spells.insert("Atonement".to_owned());

    let mut player_ids: HashSet<i64> = HashSet::new();

    // Populate healer_ids using CombatantInfo events
    for event in &combat_log.events {
        match &event.ty {
            EventType::EncounterStart { .. } => continue,
            EventType::CombatantInfo {
                source_id, spec_id, ..
            } => {
                if HEALER_SPECS.contains(spec_id) {
                    healer_ids.insert(*source_id);
                }
                player_ids.insert(*source_id);
            }
            _ => break,
        }
    }

    // Pre-process valid heal spells
    for event in &combat_log.events {
        if let EventType::Cast {
            source_id,
            target_id,
            ability_game_id,
            ..
        } = &event.ty
        {
            if healer_ids.contains(source_id) && player_ids.contains(target_id) {
                valid_heal_spells.insert(spell_names.get(ability_game_id).unwrap().clone());
            }
        }
    }

    valid_heal_spells.remove("Power Infusion");
    valid_heal_spells.remove("Blessing of Autumn");
    valid_heal_spells.remove("Cleanse");
    valid_heal_spells.remove("Naturalize");
    valid_heal_spells.remove("Blessing of Winter");

    let mut alive_players: HashSet<i64> = player_ids.iter().cloned().collect();

    for player in &player_ids {
        println!(
            "Player: {}-{}",
            combat_log.actors[*player as usize].name,
            combat_log.actors[*player as usize].server.as_ref().unwrap()
        );
    }

    for event in &combat_log.events {
        match &event.ty {
            EventType::Damage { target_id, .. } => {
                if let Some(resources) = &event.resources {
                    health_map.insert(*target_id, resources.hit_points);
                }
            }
            EventType::ResourceChange { target_id, .. } => {
                if let Some(resources) = &event.resources {
                    health_map.insert(*target_id, resources.hit_points);
                }
            }
            EventType::Heal {
                source_id,
                target_id,
                ability_game_id,
                overheal,
                ..
            } => {
                // Only process correctness for valid heal spells
                if healer_ids.contains(source_id)
                    && valid_heal_spells.contains(
                        spell_names
                            .get(ability_game_id)
                            .unwrap_or(&"Unknown".to_string()),
                    )
                    && alive_players.contains(target_id)
                    && overheal.is_none()
                {
                    let mut health_rank: Vec<_> = alive_players
                        .iter()
                        .map(|&id| (id, *health_map.get(&id).unwrap_or(&0)))
                        .collect();

                    health_rank.sort_by_key(|&(_, health)| health);

                    if health_rank.len() > 1 {
                        if let Some(position) =
                            health_rank.iter().position(|&(id, _)| id == *target_id)
                        {
                            let correctness = 100.0
                                - ((position as f64 / (health_rank.len() - 1) as f64) * 100.0);

                            let (total_correctness, heal_count) =
                                correctness_map.entry(*source_id).or_insert((0.0, 0));
                            *total_correctness += correctness;
                            *heal_count += 1;
                        }
                    }
                }

                if let Some(resources) = &event.resources {
                    health_map.insert(*target_id, resources.hit_points);
                }
            }
            EventType::Cast { source_id, .. } => {
                if let Some(resources) = &event.resources {
                    health_map.insert(*source_id, resources.hit_points);
                }
            }
            EventType::Death { target_id, .. } => {
                alive_players.remove(target_id);
            }
            _ => {}
        }
    }

    dbg!(valid_heal_spells);

    correctness_map
        .into_iter()
        .filter_map(|(healer_id, (total_correctness, heal_count))| {
            combat_log.actors.get(healer_id as usize).map(|actor| {
                let average_correctness = total_correctness / heal_count as f64;
                println!(
                    "{}: {:.2}% ({})",
                    actor.name, average_correctness, heal_count
                );
            })
        })
        .for_each(drop);

    Ok(())
}
