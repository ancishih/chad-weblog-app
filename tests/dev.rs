// to run this test:
// first run `cargo watch -q -c -w src/ -x run` to run up main.rs
// second run `cargo watch -q -c -w tests/ -x "test -q dev -- --nocapture"` for testing.
use anyhow::Result;
use serde_json::json;
#[tokio::test]
async fn dev() -> Result<()> {
    let client = httpc_test::new_client("http://127.0.0.1:9598")?;

    // region: --- stock routes
    // list_sectors
    let req_sectors = client.do_get("/api/stock/sectors");
    req_sectors.await?.print().await?;
    // list_symbols
    let req_symbols = client.do_get("/api/stock/symbol");
    req_symbols.await?.print().await?;
    // search_symbol
    let req_search_symbol = client.do_post("/api/stock/symbol", json!({"search_string":"Tsla"}));
    req_search_symbol.await?.print().await?;

    let req_symbol_price = client.do_get("/api/stock/price/daily/a");
    req_symbol_price.await?.print().await?;
    // profile
    let req_profile = client.do_get("/api/stock/profile/AAPL");
    req_profile.await?.print().await?;

    // endregion: --- stock routes
    Ok(())
}
