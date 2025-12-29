use std::env;
use std::fs;
use unipred_core::domain::MarketSource;
use unipred_core::UnipredCore;

// Helper to setup authenticated core
async fn setup_core() -> UnipredCore {
    // Manually parse .env because rust-dotenv struggles with multiline strings
    let env_paths = vec![".env", "../.env"];
    for path in env_paths {
        let p = std::path::Path::new(path);
        if p.exists() {
            if let Ok(content) = fs::read_to_string(p) {
                // Parse KALSHI_API_KEY_ID
                if env::var("KALSHI_API_KEY_ID").is_err() {
                    for line in content.lines() {
                        if line.trim().starts_with("KALSHI_API_KEY_ID=") {
                            let val = line.trim()
                                .trim_start_matches("KALSHI_API_KEY_ID=")
                                .trim_matches('"');
                            env::set_var("KALSHI_API_KEY_ID", val);
                            break;
                        }
                    }
                }

                // Parse KALSHI_PRIVATE_KEY (multiline support)
                if env::var("KALSHI_PRIVATE_KEY").is_err() {
                    let key_marker = "KALSHI_PRIVATE_KEY=\"";
                    if let Some(start_idx) = content.find(key_marker) {
                        let rest = &content[start_idx + key_marker.len()..];
                        if let Some(end_idx) = rest.find('"') {
                            let val = &rest[..end_idx];
                            env::set_var("KALSHI_PRIVATE_KEY", val);
                        }
                    }
                }
            }
        }
    }

    let key_id = env::var("KALSHI_API_KEY_ID").expect("KALSHI_API_KEY_ID must be set");
    let private_key = env::var("KALSHI_PRIVATE_KEY").expect("KALSHI_PRIVATE_KEY must be set");

    let mut core = UnipredCore::new("".to_string());
    core.kalshi
        .login_apikey(&key_id, &private_key) // Using string content method for simplicity if available, or path
        .await
        .expect("Login failed");

    core
}

#[tokio::test]
async fn test_fetch_kalshi_markets() {
    let core = setup_core().await;

    let result = core
        .fetch_markets(
            Some(MarketSource::Kalshi),
            Some(10),
            None,
            Some("open".to_string()),
        )
        .await
        .expect("Failed to fetch markets");

    assert!(!result.markets.is_empty());
    assert!(result.markets.len() <= 10);

    let market = &result.markets[0];
    assert!(!market.ticker.is_empty());
    assert!(!market.title.is_empty());
    assert_eq!(market.source, "Kalshi");
    assert_eq!(market.status, "active");
}

#[tokio::test]
async fn test_fetch_polymarket_markets() {
    let core = setup_core().await;

    let result = core
        .fetch_markets(Some(MarketSource::Polymarket), Some(10), None, None)
        .await
        .expect("Failed to fetch markets");

    assert!(!result.markets.is_empty());

    let market = &result.markets[0];
    assert!(!market.ticker.is_empty());
    assert!(!market.title.is_empty());
    assert_eq!(market.source, "Polymarket");
    assert!(market.status == "active" || market.status == "closed");
}

#[tokio::test]
async fn test_fetch_markets_pagination() {
    let core = setup_core().await;

    // First page
    let page1 = core
        .fetch_markets(
            Some(MarketSource::Kalshi),
            Some(5),
            None,
            Some("open".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(page1.markets.len(), 5);
    assert!(!page1.cursor.is_empty());

    // Second page
    let page2 = core
        .fetch_markets(
            Some(MarketSource::Kalshi),
            Some(5),
            Some(page1.cursor.clone()),
            Some("open".to_string()),
        )
        .await
        .unwrap();

    assert!(!page2.markets.is_empty());

    // Disjoint check
    let tickers1: std::collections::HashSet<_> =
        page1.markets.iter().map(|m| &m.ticker).collect();
    let tickers2: std::collections::HashSet<_> =
        page2.markets.iter().map(|m| &m.ticker).collect();

    assert!(tickers1.is_disjoint(&tickers2));
}

#[tokio::test]
async fn test_get_quote_kalshi() {
    let core = setup_core().await;
    let ticker = "KXLINKMINY-25-10".to_string(); // Assuming this is valid from Python tests

    let quote = core
        .get_quote(ticker.clone(), None)
        .await
        .expect("Failed to get quote");

    assert_eq!(quote.ticker, ticker);
    assert_eq!(quote.source, "Kalshi");
    assert!(!quote.price.is_empty());
    assert!(!quote.timestamp.is_empty());
}

#[tokio::test]
async fn test_get_quote_polymarket() {
    let core = setup_core().await;
    // Erika Kirk market token ID from Python tests
    let ticker =
        "36725157385158152303355940271421346899386884953712631735038848833359115722560"
            .to_string();

    let quote = core
        .get_quote(ticker.clone(), Some(MarketSource::Polymarket))
        .await;

    match quote {
        Ok(q) => {
            assert_eq!(q.ticker, ticker);
            assert_eq!(q.source, "Polymarket");
            assert!(!q.price.is_empty());
        }
        Err(e) => {
            // Allow "No orderbook exists" error as success for integration plumbing check
            // similar to Python test logic if market is stale
            let err_str = e.to_string();
            if !err_str.contains("No orderbook exists") {
                panic!("Unexpected error: {}", err_str);
            }
        }
    }
}

#[tokio::test]
#[should_panic]
async fn test_get_quote_bad_ticker() {
    let core = setup_core().await;
    core.get_quote("INVALID_TICKER_123".to_string(), None)
        .await
        .unwrap();
}