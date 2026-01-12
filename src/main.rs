use clap::*;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::error::Error;

const CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4";

#[derive(Deserialize)]
struct DnsRecordListResponse {
    result: Vec<DnsRecord>,
}

#[derive(Deserialize)]
struct DnsRecord {
    id: String,
    content: String,
}

#[derive(Serialize)]
struct UpdateDnsRecordRequest<'a> {
    r#type: &'a str,
    name: &'a str,
    content: &'a str,
    ttl: u32,
    proxied: bool,
}

#[derive(Parser)]
#[command(
    name = "dns-update",
    about = "silly binary to update cloudflare dns records"
)]

struct Cli {
    #[arg(short, long, env = "CF_TOKEN")]
    token: String,

    #[arg(short, long, env = "CF_ZONE")]
    zone: String,

    #[arg(short, long, env = "CF_RECORD")]
    record: String,

    #[arg(short, long)]
    ip: String,

    #[arg(short, long)]
    force: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", cli.token))?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let res: DnsRecordListResponse = client
        .get(&format!(
            "{}/zones/{}/dns_records?type=A&name={}",
            CLOUDFLARE_API_BASE, cli.zone, cli.record
        ))
        .headers(headers.clone())
        .send()?
        .json()?;

    if let Some(record) = res.result.first() {
        let current_ip = &record.content;

        // IP is the same as dns record, nothing to do unless --force is present
        if !cli.force && current_ip == &cli.ip {
            return Ok(());
        }

        let update_req = UpdateDnsRecordRequest {
            r#type: "A",
            name: &cli.record,
            content: &cli.ip,
            ttl: 1,
            proxied: false,
        };

        let update_url = format!(
            "{}/zones/{}/dns_records/{}",
            CLOUDFLARE_API_BASE, cli.zone, record.id
        );

        let update_res = client
            .put(&update_url)
            .headers(headers)
            .json(&update_req)
            .send()?;

        if update_res.status().is_success() {
            println!("{} {} -> {}", cli.record, current_ip, cli.ip);
        } else {
            println!(
                "Failed to update DNS record {} to {}: {:?}",
                cli.record,
                cli.ip,
                update_res.text()?
            );
        }
    } else {
        println!("dns record {} not found.", cli.record);
    }

    Ok(())
}
