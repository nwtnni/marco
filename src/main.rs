use std::time;
use anyhow::Context;

mod api;
mod ip;

fn main() -> anyhow::Result<()> {
    let client = client()?;
    let ip = ip::get(&client)?;
    println!("{}", ip);
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
