use serde::{Deserialize, Serialize};

use super::report::Actor;

#[derive(Serialize, Deserialize, Debug)]
pub struct CombatLog {
    pub report_id: String,
    pub fight_id: i64,
    pub events: Vec<CombatLogEvent>,
    pub actors: Vec<Actor>,
    pub first_event_timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReportEventPaginator {
    pub data: Vec<CombatLogEvent>,
    pub next_page_timestamp: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CombatLogEvent {
    pub timestamp: i64,
    pub fight: i64,

    #[serde(flatten)]
    pub ty: EventType,

    pub source_marker: Option<i64>,
    pub target_marker: Option<i64>,

    #[serde(flatten)]
    pub resources: Option<EventResources>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventResources {
    pub hit_points: i64,
    pub max_hit_points: i64,
    pub attack_power: i64,
    pub spell_power: i64,
    pub armor: i64,
    pub absorb: i64,
    pub x: i64,
    pub y: i64,
    pub facing: i64,
    #[serde(rename = "mapID")]
    pub map_id: i64,
    pub versatility: i64,
    pub avoidance: i64,
    pub item_level: i64,
}

// Docs on event types can be found here:
// https://www.warcraftlogs.com/help/pins

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum EventType {
    #[serde(rename_all = "camelCase")]
    ApplyBuff {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        absorb: Option<i64>,
    },
    #[serde(rename_all = "camelCase")]
    ApplyBuffStack {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        stack: i64,
    },
    #[serde(rename_all = "camelCase")]
    ApplyDebuff {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    ApplyDebuffStack {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        stack: i64,
    },
    #[serde(rename_all = "camelCase")]
    AuraBroken {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        #[serde(rename = "extraAbilityGameID")]
        extra_ability_game_id: i64,

        is_buff: bool,
    },
    #[serde(rename_all = "camelCase")]
    BeginCast {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    Cast {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    CombatantInfo {
        #[serde(rename = "sourceID")]
        source_id: i64,

        gear: Vec<Gear>,

        auras: Vec<Aura>,

        expansion: String,

        faction: i64,

        #[serde(rename = "specID")]
        spec_id: i64,

        strength: i64,

        agility: i64,

        stamina: i64,

        intellect: i64,

        dodge: i64,

        parry: i64,

        block: i64,

        armor: i64,

        crit_melee: i64,

        crit_ranged: i64,

        crit_spell: i64,

        haste_melee: i64,

        haste_ranged: i64,

        haste_spell: i64,

        speed: i64,

        leech: i64,

        avoidance: i64,

        mastery: i64,

        versatility_damage_done: i64,

        versatility_healing_done: i64,

        versatility_damage_reduction: i64,

        talent_tree: Vec<TalentTree>,

        talents: Vec<Option<serde_json::Value>>,

        pvp_talents: Vec<PvpTalent>,

        custom_power_set: Vec<Option<serde_json::Value>>,

        secondary_custom_power_set: Vec<Option<serde_json::Value>>,

        tertiary_custom_power_set: Vec<Option<serde_json::Value>>,
    },
    #[serde(rename_all = "camelCase")]
    Damage {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        hit_type: i64,

        amount: i64,

        unmitigated_amount: Option<i64>,
    },
    #[serde(rename_all = "camelCase")]
    EncounterStart {
        #[serde(rename = "encounterID")]
        encounter_id: i64,

        name: String,

        difficulty: i64,

        size: i64,
    },
    #[serde(rename_all = "camelCase")]
    Heal {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        hit_type: i64,

        amount: i64,

        overheal: Option<i64>,

        #[serde(default)]
        tick: bool,
    },
    #[serde(rename_all = "camelCase")]
    RefreshBuff {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    RefreshDebuff {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    RemoveBuff {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    RemoveDebuff {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    ResourceChange {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        resource_change: i64,

        resource_change_type: i64,

        other_resource_change: i64,

        max_resource_amount: i64,

        waste: i64,
    },
    #[serde(rename_all = "camelCase")]
    Summon {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        target_instance: Option<i64>,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    Absorbed {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        #[serde(rename = "attackerID")]
        attacker_id: i64,

        amount: i64,

        #[serde(rename = "extraAbilityGameID")]
        extra_ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    RemoveBuffStack {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        stack: i64,
    },
    #[serde(rename_all = "camelCase")]
    ExtraAttacks {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        extra_attacks: i64,
    },
    #[serde(rename_all = "camelCase")]
    InstaKill {
        #[serde(rename = "sourceID")]
        source_id: i64,

        source_instance: Option<i64>,

        #[serde(rename = "targetID")]
        target_id: i64,

        target_instance: Option<i64>,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    Death {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        target_instance: Option<i64>,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    RemoveDebuffStack {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        stack: i64,
    },
    #[serde(rename_all = "camelCase")]
    EmpowerStart {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    EmpowerEnd {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        empowerment_level: i64,
    },
    #[serde(rename_all = "camelCase")]
    Drain {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        resource_change: i64,

        resource_change_type: i64,

        other_resource_change: i64,

        max_resource_amount: i64,
    },
    #[serde(rename_all = "camelCase")]
    Interrupt {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        target_instance: Option<i64>,

        target_is_friendly: Option<bool>,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        #[serde(rename = "extraAbilityGameID")]
        extra_ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    Destroy {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        target_instance: Option<i64>,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    Dispel {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        #[serde(rename = "extraAbilityGameID")]
        extra_ability_game_id: i64,

        is_buff: bool,
    },
    #[serde(rename_all = "camelCase")]
    Resurrect {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    HealAbsorbed {
        #[serde(rename = "sourceID")]
        source_id: i64,

        #[serde(rename = "targetID")]
        target_id: i64,

        #[serde(rename = "abilityGameID")]
        ability_game_id: i64,

        #[serde(rename = "healerID")]
        healer_id: i64,

        amount: i64,

        #[serde(rename = "extraAbilityGameID")]
        extra_ability_game_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    EncounterEnd {
        #[serde(rename = "encounterID")]
        encounter_id: i64,

        name: String,

        difficulty: i64,

        size: i64,

        kill: bool,
    },
    #[serde(other)]
    Unknown,
}
impl EventType {
    pub fn source_id(&self) -> Option<i64> {
        match &self {
            EventType::ApplyBuff { source_id, .. }
            | EventType::ApplyBuffStack { source_id, .. }
            | EventType::ApplyDebuff { source_id, .. }
            | EventType::ApplyDebuffStack { source_id, .. }
            | EventType::AuraBroken { source_id, .. }
            | EventType::BeginCast { source_id, .. }
            | EventType::Cast { source_id, .. }
            | EventType::CombatantInfo { source_id, .. }
            | EventType::Damage { source_id, .. }
            | EventType::Heal { source_id, .. }
            | EventType::RefreshBuff { source_id, .. }
            | EventType::RefreshDebuff { source_id, .. }
            | EventType::RemoveBuff { source_id, .. }
            | EventType::RemoveDebuff { source_id, .. }
            | EventType::ResourceChange { source_id, .. }
            | EventType::Summon { source_id, .. }
            | EventType::Absorbed { source_id, .. }
            | EventType::RemoveBuffStack { source_id, .. }
            | EventType::ExtraAttacks { source_id, .. }
            | EventType::InstaKill { source_id, .. }
            | EventType::Death { source_id, .. }
            | EventType::RemoveDebuffStack { source_id, .. }
            | EventType::EmpowerStart { source_id, .. }
            | EventType::EmpowerEnd { source_id, .. }
            | EventType::Drain { source_id, .. }
            | EventType::Interrupt { source_id, .. }
            | EventType::Destroy { source_id, .. }
            | EventType::Dispel { source_id, .. }
            | EventType::Resurrect { source_id, .. }
            | EventType::HealAbsorbed { source_id, .. } => Some(*source_id),
            EventType::EncounterStart { .. }
            | EventType::EncounterEnd { .. }
            | EventType::Unknown => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Aura {
    pub source: i64,
    pub ability: i64,
    pub stacks: i64,
    pub icon: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Gear {
    pub id: i64,
    pub quality: i64,
    pub icon: String,
    pub item_level: i64,
    pub permanent_enchant: Option<i64>,

    #[serde(rename = "bonusIDs")]
    pub bonus_ids: Option<Vec<i64>>,

    pub gems: Option<Vec<Gem>>,

    #[serde(rename = "setID")]
    pub set_id: Option<i64>,

    pub on_use_enchant: Option<i64>,
    pub temporary_enchant: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Gem {
    pub id: i64,
    pub item_level: i64,
    pub icon: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PvpTalent {
    pub id: i64,
    pub icon: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TalentTree {
    pub id: i64,
    pub rank: i64,

    #[serde(rename = "nodeID")]
    pub node_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "camelCase")]
pub enum Expansion {
    Dragonflight,
    #[serde(other)]
    Unknown,
}
