use std::collections::HashMap;

use anyhow::{bail, Context};

use crate::warcraft_logs::{
    combat_log::{CombatLog, EventType},
    report::PlayerClass,
};

pub fn get_default_spells(player_class: PlayerClass) -> Vec<i64> {
    match player_class {
        PlayerClass::Priest => vec![
            194509, // Power Word: Radiance
            123040, // Mindbender
            47536,  // Rapture
            246287, // Evangelism
            271466, // Luminous Barrier
            132603, // Shadowfiend
            62618,  // Power Word: Barrier
            421453, // Ultimate Penitence
        ],
        _ => vec![],
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DynamicTimerType {
    SpellCastSuccess,
    SpellCastStart,
    SpellAuraApplied,
    SpellAuraRemoved,
}

impl DynamicTimerType {
    pub fn does_event_match(&self, event: &EventType, id: i64) -> bool {
        match self {
            DynamicTimerType::SpellCastSuccess => match event {
                EventType::Cast {
                    ability_game_id, ..
                } => ability_game_id == &id,
                _ => false,
            },
            DynamicTimerType::SpellCastStart => match event {
                EventType::BeginCast {
                    ability_game_id, ..
                } => ability_game_id == &id,
                _ => false,
            },
            DynamicTimerType::SpellAuraApplied => match event {
                EventType::ApplyBuff {
                    ability_game_id, ..
                } => ability_game_id == &id,
                EventType::ApplyBuffStack {
                    ability_game_id, ..
                } => ability_game_id == &id,
                EventType::ApplyDebuff {
                    ability_game_id, ..
                } => ability_game_id == &id,
                EventType::ApplyDebuffStack {
                    ability_game_id, ..
                } => ability_game_id == &id,
                _ => false,
            },
            DynamicTimerType::SpellAuraRemoved => match event {
                EventType::RemoveBuff {
                    ability_game_id, ..
                } => ability_game_id == &id,
                EventType::RemoveBuffStack {
                    ability_game_id, ..
                } => ability_game_id == &id,
                EventType::RemoveDebuff {
                    ability_game_id, ..
                } => ability_game_id == &id,
                EventType::RemoveDebuffStack {
                    ability_game_id, ..
                } => ability_game_id == &id,
                _ => false,
            },
        }
    }

    pub fn as_abbreviation(&self) -> &str {
        match self {
            DynamicTimerType::SpellCastSuccess => "SCC",
            DynamicTimerType::SpellCastStart => "SCS",
            DynamicTimerType::SpellAuraApplied => "SAA",
            DynamicTimerType::SpellAuraRemoved => "SAR",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DynamicTimer {
    pub ty: DynamicTimerType,
    pub spell_id: i64,
    pub counter: i64,
}

impl TryFrom<String> for DynamicTimer {
    fn try_from(value: String) -> anyhow::Result<Self> {
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() != 3 {
            bail!("Invalid dynamic timer format");
        }

        let ty = match parts[0] {
            "SCS" => DynamicTimerType::SpellCastStart,
            "SCC" => DynamicTimerType::SpellCastSuccess,
            "SAR" => DynamicTimerType::SpellAuraRemoved,
            "SAA" => DynamicTimerType::SpellAuraApplied,
            _ => bail!("Invalid dynamic timer type"),
        };

        let spell_id = parts[1].parse().context("Invalid Spell ID")?;
        let counter = parts[2].parse().context("Invalid counter")?;

        Ok(DynamicTimer {
            ty,
            spell_id,
            counter,
        })
    }

    type Error = anyhow::Error;
}

impl ToString for DynamicTimer {
    fn to_string(&self) -> String {
        format!(
            ",{}:{}:{}",
            self.ty.as_abbreviation(),
            self.spell_id,
            self.counter
        )
    }
}

pub struct Assignment {
    pub spell_id: i64,
    pub dynamic_timer: Option<DynamicTimer>,
    pub time: i64,
}

impl ToString for Assignment {
    fn to_string(&self) -> String {
        let minutes = self.time / 60;
        let seconds = self.time % 60;

        let dynamic_timer = self
            .dynamic_timer
            .as_ref()
            .map(|dt| dt.to_string())
            .unwrap_or("".to_owned());

        format!(
            "{{time:{:02}:{:02}{}}} - {{spell:{}}}",
            minutes, seconds, dynamic_timer, self.spell_id
        )
    }
}

pub fn extract_assignments(
    combat_log: &CombatLog,
    player_name: &str,
    assigned_spells: Vec<i64>,
    mut dynamic_timers: Vec<DynamicTimer>,
) -> anyhow::Result<Vec<Assignment>> {
    let actor_index = combat_log
        .actors
        .iter()
        .enumerate()
        .find(|(_, actor)| actor.name == player_name && actor.server.is_some())
        .map(|(idx, _)| idx)
        .ok_or(anyhow::anyhow!("Player not found"))? as i64;

    let mut assignments = Vec::new();

    let mut zero_time = combat_log.first_event_timestamp;
    let mut within_dynamic_timer: Option<DynamicTimer> = None;

    let mut dynamic_timer_counter: HashMap<DynamicTimerType, HashMap<i64, i64>> = HashMap::new();

    for event in &combat_log.events {
        for (index, dynamic_timer) in dynamic_timers.iter().enumerate() {
            if dynamic_timer
                .ty
                .does_event_match(&event.ty, dynamic_timer.spell_id)
            {
                let map = dynamic_timer_counter
                    .entry(dynamic_timer.ty.clone())
                    .or_default();

                let counter = map.entry(dynamic_timer.spell_id).or_default();
                *counter += 1;

                if *counter == dynamic_timer.counter {
                    within_dynamic_timer = Some(dynamic_timer.clone());
                    zero_time = event.timestamp;

                    dynamic_timers.remove(index);
                }

                break;
            }
        }

        let EventType::Cast {
            source_id,
            ability_game_id,
            ..
        } = event.ty
        else {
            continue;
        };

        if source_id != actor_index {
            continue;
        }

        if !assigned_spells.contains(&ability_game_id) {
            continue;
        }

        assignments.push(Assignment {
            spell_id: ability_game_id,
            dynamic_timer: within_dynamic_timer.clone(),
            time: (event.timestamp - zero_time) / 1000,
        });
    }

    Ok(assignments)
}
