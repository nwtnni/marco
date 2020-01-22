use std::any;
use std::error;
use std::fmt;
use std::net;

use anyhow::anyhow;
use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;

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
    #[serde(flatten)]
    meta: Meta,
    result: Vec<Zone>,
}

impl From<Zones> for anyhow::Result<Vec<Zone>> {
    fn from(zones: Zones) -> anyhow::Result<Vec<Zone>> {
        anyhow::Result::<()>::from(zones.meta)?;
        Ok(zones.result)
    }
}

/// Represents a single zone from `GET /zones`.
#[derive(Debug, Deserialize)]
struct Zone {
    id: ZoneID,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZoneID(String);

impl fmt::Display for ZoneID {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[derive(Debug, Deserialize)]
struct Records {
    #[serde(flatten)]
    meta: Meta,
    result: Vec<Record<String>>, 
}

impl From<Records> for anyhow::Result<Vec<Record<String>>> {
    fn from(records: Records) -> anyhow::Result<Vec<Record<String>>> {
        anyhow::Result::<()>::from(records.meta)?;
        Ok(records.result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Record<C> {
    id: RecordID,
    r#type: String,
    name: String,
    content: C,
    ttl: serde_json::Value,
    proxied: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RecordID(String);

impl fmt::Display for RecordID {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[derive(Debug, Deserialize)]
struct Meta {
    success: bool,
    errors: serde_json::Value,
    messages: serde_json::Value,
}

impl fmt::Display for Meta {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "{:#}", self.errors)?;
        write!(fmt, "{:#}", self.messages)
    }
}

impl error::Error for Meta {}

impl From<Meta> for anyhow::Result<()> {
    fn from(meta: Meta) -> anyhow::Result<()> {
        if meta.success {
            Ok(())
        } else {
            Err(anyhow!(meta))
        }
    }
}

impl<'c> Client<'c> {
    pub fn new(inner: &'c reqwest::blocking::Client, token: String) -> Self {
        Client {
            inner,
            token,
        }
    }

    /// Retrieve the ID of a single zone.
    pub fn get_zone_id(&self, zone_name: &str) -> anyhow::Result<ZoneID> {
        let mut zones = self
            .get::<Zones>("zones")
            .with_context(|| format!("Could not get ID of zone {}", zone_name))
            .and_then(anyhow::Result::<Vec<Zone>>::from)?;

        zones.retain(|zone| &zone.name == zone_name);

        match zones.len() {
        | 0 => Err(anyhow!("No matching zones found")),
        | 1 => Ok(zones.remove(0).id),
        | _ => unreachable!(),
        }
    }

    pub fn get_dns_record(
        &self,
        zone_name: &str,
        zone_id: &ZoneID,
        record_name: &str,
    ) -> anyhow::Result<Record<net::IpAddr>> {
        let mut records = self
            .get::<Records>(&format!("zones/{}/dns_records", zone_id))
            .with_context(|| format!("Could not get DNS record for {} in zone {}", record_name, zone_name))
            .and_then(anyhow::Result::<Vec<Record<String>>>::from)?;

        records.retain(|record| &record.name == record_name);

        match records.len() {
        | 0 => Err(anyhow!("No matching DNS records found")),
        | 1 => {
            let record = records.remove(0);
            let ip = record.content
                .parse::<net::IpAddr>()
                .with_context(|| format!("Could not parse content of DNS record for {}", record_name))?;
            Ok(Record {
                id: record.id,
                r#type: record.r#type,
                name: record.name,
                content: ip,
                ttl: record.ttl,
                proxied: record.proxied,
            })
        }
        | _ => unreachable!(),
        }
    }

    pub fn put_dns_record(
        &self,
        zone_id: &ZoneID,
        zone_name: &str,
        record_id: &RecordID,
        record_name: &str,
        record: &Record<net::IpAddr>,
    ) -> anyhow::Result<()> {
        self.put(&format!("zones/{}/dns_records/{}", zone_id, record_id), record)
            .with_context(|| format!("Could not PUT DNS record for {} in zone {}", record_name, zone_name))
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
