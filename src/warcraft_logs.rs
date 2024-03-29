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

pub struct WarcraftLogs {}

impl WarcraftLogs {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_report(&self, report_id: &str, fight_id: u32) -> anyhow::Result<CombatLog> {
        let mut headers = header::HeaderMap::new();
        let mut auth_value =
            header::HeaderValue::from_str(&format!("Bearer {}", &env::var("WCL_SECRET")?))?;
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        let mut start_time = 0.0;

        let mut events = Vec::new();

        loop {
            println!("Start time: {start_time}");

            let variables = combat_log_query::Variables {
                report_id: report_id.to_owned(),
                fight_id: fight_id as i64,
                start_time: Some(start_time),
            };
            let body = <CombatLogQuery>::build_query(variables);
            let response = client.post(WCL_URL).json(&body).send().await?;

            let response: serde_json::Value = response.json().await?;

            let mut paginator = combat_log::ReportEventPaginator::deserialize(
                response
                    .get("data")
                    .and_then(|v| v.get("reportData"))
                    .and_then(|v| v.get("report"))
                    .and_then(|v| v.get("events"))
                    .context("Unknown response format")?,
            )?;

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
