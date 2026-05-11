//! `ship openapi export` — prints the daemon's utoipa-derived OpenAPI spec to stdout.

use anyhow::Result;

pub fn run(pretty: bool) -> Result<()> {
    let json: String = shipd::routes::openapi::to_json(pretty)?;
    println!("{json}");
    Ok(())
}
