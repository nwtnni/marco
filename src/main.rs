use std::time;

use anyhow::Context;
use structopt::StructOpt;

mod api;
mod ip;

/// Utility for basic dynamic DNS.
///
/// Retrieves the current public IP address and updates
/// the corresponding DNS record entry in Cloudflare.
/// Intended to be scheduled as a recurring job.
#[derive(Debug, StructOpt)]
#[structopt(name = "marco", about = "Utility for basic dynamic DNS")]
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

    /// Log debug information to `stdout`.
    #[structopt(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {

    let opt = Opt::from_args();

    macro_rules! log {
        ($($arg:expr),* $(,)?) => {
            if opt.verbose {
                println!($($arg),*);
            }
        }
    }

    log!("Starting...");

    let client = client()?;

    log!("Initialized HTTP client");

    let cloudflare = api::Client::new(&client, opt.token);

    let zone_id = cloudflare.get_zone_id(&opt.zone)?;

    log!("Retrieved Cloudflare zone ID for {}: {}", opt.zone, zone_id);

    let mut record = cloudflare.get_dns_record(&opt.zone, &zone_id, &opt.record)?;

    log!("Retrieved Cloudflare record for {}: {:#?}", opt.record, record);

    let ip = ip::get(&client)?;

    log!("Retrieved public IP address: {}", ip);

    // No further work to be done
    if record.ip() == ip {
        log!("No mismatch detected, exiting successfully");
        return Ok(());
    }

    // Otherwise update Cloudflare DNS to match new public IP
    record.set_ip(ip);

    log!("Mismatch detected, updating Cloudflare DNS record");

    cloudflare.put_dns_record(&zone_id, &opt.zone, record.id(), &opt.record, &record)?;

    log!("Successfully updated Cloudflare DNS record");

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
