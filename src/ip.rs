//! Retrieval of our public IP address.
//!
//! For robustness, this module includes backup providers
//! so that we only fail if all of them fail. All of these
//! providers return our public IP address in plaintext form.

use std::fmt;
use std::net;
use std::error;

use anyhow::anyhow;
use anyhow::Context;

static PROVIDERS: [&str; 3] = [
    "https://api.ipify.org",
    "https://icanhazip.com",
    "https://bot.whatismyipaddress.com",
];

#[derive(Debug)]
struct Errors(Vec<anyhow::Error>);

impl fmt::Display for Errors {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let total = self.0.len();
        for (index, error) in self.0.iter().enumerate() {
            write!(fmt, "\n{}/{}: {}", index + 1, total, error)?;
        }
        Ok(())
    }
}

impl error::Error for Errors {}

pub fn get(client: &reqwest::blocking::Client) -> anyhow::Result<net::IpAddr> {
    let mut errors = Vec::new();
    for provider in &PROVIDERS {
        match get_from(client, provider) {
        | Ok(ip) => return Ok(ip),
        | Err(error) => errors.push(error),
        }
    }
    Err(anyhow!(Errors(errors)))
}

fn get_from(client: &reqwest::blocking::Client, provider: &str) -> anyhow::Result<net::IpAddr> {
    client.get(provider)
        .send()
        .with_context(|| format!("Failed to send GET request to {}", provider))?
        .error_for_status()
        .with_context(|| format!("Received error code from {}", provider))?
        .text()
        .with_context(|| format!("Could not parse response from {} as UTF-8", provider))?
        .trim()
        .parse::<net::IpAddr>()
        .with_context(|| format!("Could not parse response from {} as IP address", provider))
}
