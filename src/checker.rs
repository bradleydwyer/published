use crate::store::Store;
use crate::types::*;
use reqwest::Client;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

fn build_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .expect("Failed to build HTTP client")
}

async fn check_store(
    client: &Client,
    store: Store,
    name: &str,
    semaphore: &Semaphore,
) -> StoreResult {
    let _permit = semaphore.acquire().await.unwrap();
    match store {
        Store::AppleAppStore => check_apple(client, store, name).await,
        Store::GooglePlay => check_google_play(client, store, name).await,
    }
}

async fn check_apple(client: &Client, store: Store, name: &str) -> StoreResult {
    let start = Instant::now();
    let url = format!(
        "https://itunes.apple.com/search?term={}&entity=software&limit=50",
        name
    );

    match client.get(&url).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                return StoreResult {
                    store_id: store.id().to_string(),
                    store_name: store.name().to_string(),
                    available: Availability::Unknown,
                    browse_url: Some(store.browse_url(name)),
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("HTTP {}", response.status())),
                };
            }

            let body = match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    return StoreResult {
                        store_id: store.id().to_string(),
                        store_name: store.name().to_string(),
                        available: Availability::Unknown,
                        browse_url: Some(store.browse_url(name)),
                        elapsed_ms: start.elapsed().as_millis() as u64,
                        error: Some(e.to_string()),
                    };
                }
            };

            let available = match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    let results = json.get("results").and_then(|v| v.as_array());
                    match results {
                        Some(arr) => {
                            let has_exact_match = arr.iter().any(|item| {
                                item.get("trackName")
                                    .and_then(|v| v.as_str())
                                    .is_some_and(|track_name| {
                                        track_name.eq_ignore_ascii_case(name)
                                    })
                            });
                            if has_exact_match {
                                Availability::Taken
                            } else {
                                Availability::Available
                            }
                        }
                        None => Availability::Unknown,
                    }
                }
                Err(_) => Availability::Unknown,
            };

            StoreResult {
                store_id: store.id().to_string(),
                store_name: store.name().to_string(),
                available,
                browse_url: Some(store.browse_url(name)),
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: None,
            }
        }
        Err(e) => StoreResult {
            store_id: store.id().to_string(),
            store_name: store.name().to_string(),
            available: Availability::Unknown,
            browse_url: Some(store.browse_url(name)),
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some(e.to_string()),
        },
    }
}

async fn check_google_play(client: &Client, store: Store, name: &str) -> StoreResult {
    let start = Instant::now();
    let url = format!(
        "https://play.google.com/store/search?q={}&c=apps&hl=en",
        name
    );

    match client.get(&url).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                return StoreResult {
                    store_id: store.id().to_string(),
                    store_name: store.name().to_string(),
                    available: Availability::Unknown,
                    browse_url: Some(store.browse_url(name)),
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("HTTP {}", response.status())),
                };
            }

            let body = match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    return StoreResult {
                        store_id: store.id().to_string(),
                        store_name: store.name().to_string(),
                        available: Availability::Unknown,
                        browse_url: Some(store.browse_url(name)),
                        elapsed_ms: start.elapsed().as_millis() as u64,
                        error: Some(e.to_string()),
                    };
                }
            };

            // Google Play server-renders search results for SEO.
            // Check for the exact app name in HTML element text content.
            // Look for patterns like >Name< which indicate text within HTML elements.
            let available = check_google_play_html(&body, name);

            StoreResult {
                store_id: store.id().to_string(),
                store_name: store.name().to_string(),
                available,
                browse_url: Some(store.browse_url(name)),
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: None,
            }
        }
        Err(e) => StoreResult {
            store_id: store.id().to_string(),
            store_name: store.name().to_string(),
            available: Availability::Unknown,
            browse_url: Some(store.browse_url(name)),
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some(e.to_string()),
        },
    }
}

/// Check Google Play HTML for an exact app name match.
///
/// Google Play server-renders search results with app data embedded in
/// script tags. This is a best-effort heuristic — the HTML structure
/// may change, in which case this returns Unknown.
fn check_google_play_html(body: &str, name: &str) -> Availability {
    if body.len() < 1000 {
        return Availability::Unknown;
    }

    // Check if search returned any app listings.
    // App detail links indicate results exist.
    let has_results = body.contains("/store/apps/details");

    if !has_results {
        // No app listings on the page — name is available
        return Availability::Available;
    }

    // Results exist. Check if any app has an exact name match.
    //
    // Google Play embeds app data in AF_initDataCallback script blocks.
    // The search query appears once in the request data (e.g., "Slack").
    // If an app with that exact name exists, it appears again in the
    // result data — so >= 2 case-insensitive occurrences of "name"
    // indicates an exact match.
    let name_lower = name.to_lowercase();
    let body_lower = body.to_lowercase();
    let quoted = format!("\"{name_lower}\"");
    let occurrences = body_lower.matches(&quoted).count();

    if occurrences >= 2 {
        Availability::Taken
    } else {
        // Results exist but none match the exact name
        Availability::Available
    }
}

async fn check_app_inner(
    name: &str,
    stores: &[Store],
    client: &Client,
    semaphore: &Arc<Semaphore>,
) -> CheckResult {
    let start = Instant::now();
    let name = name.trim().to_string();

    let mut handles = Vec::new();
    for &store in stores {
        let client = client.clone();
        let sem = Arc::clone(semaphore);
        let name = name.clone();
        handles.push(tokio::spawn(async move {
            check_store(&client, store, &name, &sem).await
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    let summary = Summary {
        available: results
            .iter()
            .filter(|r| r.available == Availability::Available)
            .count(),
        taken: results
            .iter()
            .filter(|r| r.available == Availability::Taken)
            .count(),
        unknown: results
            .iter()
            .filter(|r| r.available == Availability::Unknown)
            .count(),
        total: results.len(),
    };

    CheckResult {
        name,
        summary,
        results,
        elapsed_ms: start.elapsed().as_millis() as u64,
    }
}

pub async fn check_app(name: &str, stores: &[Store]) -> CheckResult {
    let client = build_client();
    let semaphore = Arc::new(Semaphore::new(5));
    check_app_inner(name, stores, &client, &semaphore).await
}

pub async fn check_apps(names: &[String], stores: &[Store]) -> Vec<CheckResult> {
    let client = build_client();
    let semaphore = Arc::new(Semaphore::new(5));

    let mut handles = Vec::new();
    for name in names {
        let name = name.clone();
        let stores = stores.to_vec();
        let client = client.clone();
        let sem = Arc::clone(&semaphore);
        handles.push(tokio::spawn(async move {
            check_app_inner(&name, &stores, &client, &sem).await
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }
    results
}
