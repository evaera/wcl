use anyhow::Context;
use clap::{Parser, Subcommand};
use graphql_client::GraphQLQuery;
use reqwest::header;
use serde::Deserialize;
use std::env;

use crate::combat_log::{self, CombatLog};

#[allow(clippy::upper_case_acronyms)]
type JSON = serde_json::Value;

const WCL_URL: &str = "https://www.warcraftlogs.com/api/v2/client";

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "generated/schema.json",
    query_path = "src/queries/CombatLogQuery.graphql",
    response_derives = "Debug"
)]
pub struct CombatLogQuery;

pub struct WarcraftLogs {
    access_token: String,
}

#[allow(clippy::new_without_default)]
impl WarcraftLogs {
    pub fn new() -> Self {
        Self {
            access_token: env::var("WCL_SECRET").unwrap(),
        }
    }

    pub async fn get_report(&self, report_id: &str, fight_id: u32) -> anyhow::Result<CombatLog> {
        let client = reqwest::Client::new();

        let mut start_time = 0.0;

        let mut events = Vec::new();

        loop {
            log::debug!("Downloading combat data starting from time: {start_time}");

            let variables = combat_log_query::Variables {
                report_id: report_id.to_owned(),
                fight_id: fight_id as i64,
                start_time: Some(start_time),
            };
            let body = <CombatLogQuery>::build_query(variables);
            let response = client
                .post(WCL_URL)
                .bearer_auth(self.access_token.clone())
                .json(&body)
                .send()
                .await?;

            let response: serde_json::Value = response.json().await?;

            let mut paginator = combat_log::ReportEventPaginator::deserialize(
                response
                    .get("data")
                    .and_then(|v| v.get("reportData"))
                    .and_then(|v| v.get("report"))
                    .and_then(|v| v.get("events"))
                    .context("Unknown response format")?,
            )?;

            log::debug!("Received {} events", paginator.data.len());

            events.append(&mut paginator.data);

            if let Some(next_timestamp) = paginator.next_page_timestamp {
                if next_timestamp <= start_time {
                    break;
                }

                start_time = next_timestamp;
            } else {
                break;
            }
        }

        Ok(CombatLog {
            report_id: report_id.to_owned(),
            fight_id: fight_id as i64,
            events,
        })
    }
}
