use anyhow::Context;
use graphql_client::GraphQLQuery;
use reqwest::header;
use serde::Deserialize;
use std::env;

pub mod combat_log;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().expect(".env file not found");

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

    loop {
        println!("Start time: {start_time}");

        let variables = combat_log_query::Variables {
            report_id: "BTGmLz3pPbn4xhcJ".to_owned(),
            fight_id: 3,
            start_time: Some(start_time),
        };
        let body = <CombatLogQuery>::build_query(variables);
        let response = client.post(WCL_URL).json(&body).send().await?;

        let response: serde_json::Value = response.json().await?;

        let data = combat_log::ReportEventPaginator::deserialize(
            response
                .get("data")
                .and_then(|v| v.get("reportData"))
                .and_then(|v| v.get("report"))
                .and_then(|v| v.get("events"))
                .context("Unknown response format")?,
        )?;

        if let Some(next_timestamp) = data.next_page_timestamp {
            if next_timestamp <= start_time {
                break;
            }

            start_time = next_timestamp;
        } else {
            break;
        }
    }

    Ok(())
}
