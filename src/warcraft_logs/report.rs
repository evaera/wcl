use anyhow::{bail, Context};
use graphql_client::{GraphQLQuery, QueryBody};
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::serde_as;
use std::env;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use serde::de::{self, MapAccess, Visitor};

use super::combat_log::CombatLog;
use super::ApiContext;

pub struct Report {
    pub data: ReportData,
    api_context: Arc<ApiContext>,
    pub fights: Vec<Fight>,
}

impl Report {
    pub(super) fn new(mut data: ReportData, api_context: Arc<ApiContext>) -> Self {
        let mut fights = Vec::new();

        for fight in data.fights.drain(..) {
            let mut friendly_players = Vec::new();
            let mut enemy_players = Vec::new();

            for idx in fight.friendly_players.iter() {
                friendly_players.push(data.master_data.actors[*idx as usize].clone())
            }

            for idx in fight.enemy_players.iter() {
                enemy_players.push(data.master_data.actors[*idx as usize].clone())
            }

            fights.push(Fight {
                api_context: api_context.clone(),
                data: fight,
                report_id: data.report_id.clone(),
                report_start_time: data.start_time,
                friendly_players,
                enemy_players,
                actors: data.master_data.actors.clone(),
            });
        }

        Self {
            data,
            api_context,
            fights,
        }
    }
}

pub struct Fight {
    pub data: FightData,
    pub report_id: String,
    api_context: Arc<ApiContext>,
    report_start_time: f64,
    pub friendly_players: Vec<Actor>,
    pub enemy_players: Vec<Actor>,
    actors: Vec<Actor>,
}

impl Fight {
    pub async fn download_combat_log(&self) -> anyhow::Result<CombatLog> {
        let events = self
            .api_context
            .get_combat_log(&self.report_id, self.data.id)
            .await?;

        let first_event_timestamp = events.first().map(|e| e.timestamp).unwrap_or(0);

        Ok(CombatLog {
            report_id: self.report_id.clone(),
            fight_id: self.data.id as i64,
            events,
            actors: self.actors.clone(),
            first_event_timestamp,
        })
    }

    pub fn start_time(&self) -> f64 {
        self.report_start_time + self.data.start_time
    }

    pub fn duration(&self) -> Duration {
        Duration::from_millis((self.data.end_time - self.data.start_time) as u64)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Server {
    name: String,
}

impl From<Server> for String {
    fn from(value: Server) -> Self {
        value.name
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Guild {
    pub id: u64,
    pub name: String,
    #[serde(deserialize_with = "string_or_struct::<Server, _>")]
    pub server: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ReportVisibility {
    Public,
    Private,
    Unlisted,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Region {
    compact_name: String,
}

impl From<Region> for String {
    fn from(value: Region) -> Self {
        value.compact_name
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReportData {
    #[serde(rename = "code")]
    pub report_id: String,
    pub end_time: f64,
    pub start_time: f64,
    pub guild: Option<Guild>,
    pub owner: User,

    #[serde(deserialize_with = "string_or_struct::<Region, _>")]
    pub region: String,
    pub title: String,
    pub visibility: ReportVisibility,
    fights: Vec<FightData>,
    pub master_data: MasterData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MasterData {
    pub actors: Vec<Actor>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    pub name: String,
    pub server: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FightData {
    pub id: u32,
    pub average_item_level: f64,
    pub boss_percentage: Option<f64>,
    pub difficulty: Option<i64>,
    #[serde(rename = "encounterID", deserialize_with = "none_when_default")]
    pub encounter_id: Option<i64>,
    end_time: f64,
    start_time: f64,
    pub in_progress: bool,

    #[serde(deserialize_with = "true_or_null")]
    pub kill: bool,

    pub last_phase: Option<u32>,
    pub name: String,
    pub wipe_called_time: Option<f64>,

    friendly_players: Vec<u64>,
    enemy_players: Vec<u64>,
}

pub enum PlayerClass {
    Warrior,
    Paladin,
    Hunter,
    Rogue,
    Priest,
    DeathKnight,
    Shaman,
    Mage,
    Warlock,
    Monk,
    Druid,
    DemonHunter,
    Evoker,
}

fn string_or_struct<'de, T, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
    T: Into<String> + Deserialize<'de>,
{
    struct StringOrStruct<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Into<String> + Deserialize<'de>,
    {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or Server object")
        }

        fn visit_str<E>(self, value: &str) -> Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_owned())
        }

        fn visit_map<M>(self, map: M) -> Result<String, M::Error>
        where
            M: MapAccess<'de>,
        {
            let guild = T::deserialize(de::value::MapAccessDeserializer::new(map))?;
            Ok(guild.into())
        }
    }

    deserializer.deserialize_any(StringOrStruct::<T>(PhantomData))
}

fn true_or_null<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<bool> = Option::deserialize(deserializer)?;
    Ok(value.unwrap_or(false))
}

fn none_when_default<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de> + Default + PartialEq,
{
    let value = T::deserialize(deserializer)?;
    Ok((value != T::default()).then_some(value))
}
