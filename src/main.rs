use std::time;

use anyhow::Context;
use structopt::StructOpt;

mod api;
mod ip;

/// Utility for basic dynamic DNS.
#[derive(Debug, StructOpt)]
struct Opt {
    /// Cloudflare API token. 
    ///
    /// Requires the following permissions:
    /// - `#zone:read`
    /// - `#dns_records:edit`
    ///
    /// See ["Managing API Tokens and Keys"][cf] for more information.
    ///
    /// [cf]: https://support.cloudflare.com/hc/en-us/articles/200167836-Managing-API-Tokens-and-Keys
    #[structopt(verbatim_doc_comment)]
    #[structopt(short, long, env = "CLOUDFLARE_API_TOKEN", hide_env_values = true)]
    token: String, 

    /// Cloudflare DNS record name, e.g. `foo.bar.com` or `bar.io`.
    #[structopt(short, long, env = "CLOUDFLARE_DNS_RECORD")]
    record: String,

    /// Cloudflare zone name, e.g. `foo.com` or `bar.io`.
    #[structopt(short, long, env = "CLOUDFLARE_ZONE")]
    zone: String,
}

fn main() -> anyhow::Result<()> {

    let opt = Opt::from_args();
    let client = client()?;
    let ip = ip::get(&client)?;
    let cf = api::Client::new(&client, opt.token);

    let zone_id = cf.get_zone_id(&opt.zone)?;
    let mut record = cf.get_dns_record(&opt.zone, &zone_id, &opt.record)?;

    println!("{:?}", record);


    Ok(())
}

fn client() -> anyhow::Result<reqwest::blocking::Client> {
    static TIME: time::Duration = time::Duration::from_secs(15);
    static USER: &str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
        " (",
        env!("CARGO_PKG_REPOSITORY"),
        ")",
    );
    reqwest::blocking::Client::builder()
        .timeout(TIME)
        .user_agent(USER)
        .build()
        .context("Could not construct HTTP client")
}
