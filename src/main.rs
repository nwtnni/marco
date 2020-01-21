use anyhow::anyhow;
use anyhow::Context;

static CLOUDFLARE_API_TOKEN: &str = env!("CLOUDFLARE_API_TOKEN");

fn main() -> anyhow::Result<()> {

    let ip = ureq::get("http://icanhazip.com")
        .timeout_connect(1_000)
        .call()
        .into_string()
        .with_context(|| anyhow!("Could not parse response as UTF-8"))?
        .trim()
        .to_owned();

    println!("{}", ip);

    Ok(())

}
