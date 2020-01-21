use anyhow::anyhow;
use anyhow::Context;
use serde_json::Value;
use serde::Deserialize;

static CLOUDFLARE_API_TOKEN: &str = env!("CLOUDFLARE_API_TOKEN");
static DNS_RECORD_NAME: &str = env!("DNS_RECORD_NAME");

macro_rules! cloudflare {
    ($route:expr) => {
        concat!("https://api.cloudflare.com/client/v4/", $route)
    }
}

#[derive(Debug, Deserialize)]
struct Zones {
    result: Vec<Zone>,
}

#[derive(Debug, Deserialize)]
struct Zone {
    id: String,
}

fn main() -> anyhow::Result<()> {

    let ip_url = "http://icanhazip.com";
    let ip = ureq::get(ip_url)
        .timeout_connect(1_000)
        .call()
        .into_string()
        .with_context(|| anyhow!("Invalid UTF-8 response from '{}'", ip_url))?;

    let client = ureq::agent()
        .auth_kind("Bearer", CLOUDFLARE_API_TOKEN)
        .set("Content-Type", "application/json")
        .build();

    let zone_url = cloudflare!("zones");
    let zone_id = client.get(zone_url)
        .timeout_connect(1_000)
        .call()
        .into_string()
        .with_context(|| anyhow!("Invalid UTF-8 response from '{}'", zone_url))
        .map(|json| serde_json::from_str::<Zones>(&json))?
        .with_context(|| anyhow!("Invalid JSON response from '{}'", zone_url))?
        .result
        .remove(0)
        .id;

    println!("{:#?}", zone_id);

    Ok(())

}
