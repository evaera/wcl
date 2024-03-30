use std::collections::HashMap;

use crate::warcraft_logs::{
    combat_log::{CombatLog, EventType},
    report::PlayerClass,
};

fn get_default_spells(player_class: PlayerClass) -> Vec<i64> {
    match player_class {
        PlayerClass::Priest => vec![194509, 123040, 47536, 246287, 271466],
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
            .map(|dt| {
                format!(
                    ",{}:{}:{}",
                    dt.ty.as_abbreviation(),
                    dt.spell_id,
                    dt.counter
                )
            })
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
    mut dynamic_timers: Vec<DynamicTimer>,
) -> anyhow::Result<Vec<Assignment>> {
    let actor_index = combat_log
        .actors
        .iter()
        .enumerate()
        .find(|(_, actor)| actor.name == player_name && actor.server.is_some())
        .map(|(idx, _)| idx)
        .ok_or(anyhow::anyhow!("Player not found"))? as i64;

    let assigned_spells = get_default_spells(PlayerClass::Priest);

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
