use anyhow::Context;

mod api;

static IP0: &str = "https://icanhazip.com";

fn main() -> anyhow::Result<()> {

    let ip = reqwest::blocking::get(IP0)
        .with_context(|| format!("Failed to send GET request to {}", IP0))?
        .text()
        .with_context(|| format!("Failed to parse GET respnonse from {} as UTF-8", IP0))?;

    println!("{}", ip);

    Ok(())

}
