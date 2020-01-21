use std::any;

use anyhow::anyhow;
use anyhow::Context;
use serde::Deserialize;

static CLOUDFLARE_API_URL: &str = "https://api.cloudflare.com/client/v4/";

pub struct Client<'c> {
    token: String,
    inner: &'c reqwest::blocking::Client,
}

/// Represents the response from `GET /zones`:
///
/// https://api.cloudflare.com/#zone-list-zones
#[derive(Debug, Deserialize)]
struct Zones {
    result: Vec<Zone>,
}

/// Represents a single zone from `GET /zones`.
#[derive(Debug, Deserialize)]
struct Zone {
    id: String,
    name: String,
}

impl<'c> Client<'c> {
    pub fn new(inner: &'c reqwest::blocking::Client, token: String) -> Self {
        Client {
            inner,
            token,
        }
    }

    /// Retrieve the ID of a single zone.
    pub fn get_zone_id(&self, name: &str) -> anyhow::Result<String> {
        let mut zones = self
            .get::<Zones>("zones")
            .with_context(|| format!("Could not get ID of zone {}", name))?
            .result;

        zones.retain(|zone| &zone.name == name);

        match zones.len() {
        | 0 => Err(anyhow!("No matching zones found")),
        | 1 => Ok(zones.remove(0).id),
        | n => Err(anyhow!("{} matching zones found, could not select one", n)),
        }
    }

    fn get<T: serde::de::DeserializeOwned>(&self, route: &str) -> anyhow::Result<T> {
        let url = format!("{}/{}", CLOUDFLARE_API_URL, route);
        self.inner
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .with_context(|| format!("Failed to send GET request to {}", url))?
            .json()
            .with_context(|| format!("Failed to parse JSON as {} from {}", any::type_name::<T>(), url))
    }

    fn put<T: serde::Serialize>(&self, route: &str, data: &T) -> anyhow::Result<()> {
        let url = format!("{}/{}", CLOUDFLARE_API_URL, route);
        self.inner
            .put(&url)
            .bearer_auth(&self.token)
            .json(data)
            .send()
            .with_context(|| format!("Failed to send PUT request to {}", url))?
            .error_for_status()
            .with_context(|| format!("Received error response for PUT request to {}", url))
            .map(drop)
    }
}
