use anyhow::{bail, Context};
use graphql_client::{GraphQLQuery, QueryBody};
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};

use self::combat_log::{CombatLog, CombatLogEvent};

pub mod combat_log;
pub mod report;

#[allow(clippy::upper_case_acronyms)]
type JSON = serde_json::Value;

const WCL_URL: &str = "https://www.warcraftlogs.com/api/v2/client";

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "generated/schema.json",
    query_path = "src/queries/CombatLogQuery.graphql"
)]
pub struct CombatLogQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "generated/schema.json",
    query_path = "src/queries/ReportQuery.graphql"
)]
pub struct ReportQuery;

struct ApiContext {
    access_token: String,
}

impl ApiContext {
    async fn perform_query<V: serde::Serialize>(
        &self,
        body: QueryBody<V>,
    ) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::new();

        let response = client
            .post(WCL_URL)
            .bearer_auth(self.access_token.clone())
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            bail!("Request failed: {:?}", response.text().await?);
        }

        let json: serde_json::Value = response.json().await?;

        Ok(json)
    }

    pub async fn get_combat_log(
        &self,
        report_id: &str,
        fight_id: u32,
    ) -> anyhow::Result<Vec<CombatLogEvent>> {
        let mut start_time = 0.0;

        let mut events = Vec::new();

        loop {
            log::debug!("Downloading combat data starting from time: {start_time}");

            let variables = combat_log_query::Variables {
                report_id: report_id.to_owned(),
                fight_id: fight_id as i64,
                start_time: Some(start_time),
            };

            let response = self
                .perform_query(CombatLogQuery::build_query(variables))
                .await?;

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

        Ok(events)
    }
}

pub struct WarcraftLogs {
    api_context: Arc<ApiContext>,
}

#[allow(clippy::new_without_default)]
impl WarcraftLogs {
    pub fn new() -> Self {
        Self {
            api_context: Arc::new(ApiContext {
                access_token: env::var("WCL_ACCESS_TOKEN").expect("WCL_ACCESS_TOKEN not set"),
            }),
        }
    }

    pub async fn get_report(&self, report_id: &str) -> anyhow::Result<report::Report> {
        let variables = report_query::Variables {
            report_id: report_id.to_owned(),
        };

        let response = self
            .api_context
            .perform_query(ReportQuery::build_query(variables))
            .await?;

        let raw_report_data = response
            .get("data")
            .and_then(|v| v.get("reportData"))
            .and_then(|v| v.get("report"))
            .context("Unknown response format")?;

        let report: report::ReportData = serde_path_to_error::deserialize(raw_report_data)
            .context("ReportData deserialization failed")?;

        Ok(report::Report::new(report, self.api_context.clone()))
    }
}
