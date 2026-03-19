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
                    similar_count: 0,
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
                        similar_count: 0,
                        browse_url: Some(store.browse_url(name)),
                        elapsed_ms: start.elapsed().as_millis() as u64,
                        error: Some(e.to_string()),
                    };
                }
            };

            let (available, similar_count) = match serde_json::from_str::<serde_json::Value>(&body)
            {
                Ok(json) => {
                    let results = json.get("results").and_then(|v| v.as_array());
                    match results {
                        Some(arr) => {
                            let has_exact_match = arr.iter().any(|item| {
                                item.get("trackName")
                                    .and_then(|v| v.as_str())
                                    .is_some_and(|track_name| track_name.eq_ignore_ascii_case(name))
                            });
                            let availability = if has_exact_match {
                                Availability::Taken
                            } else {
                                Availability::Available
                            };
                            (availability, arr.len())
                        }
                        None => (Availability::Unknown, 0),
                    }
                }
                Err(_) => (Availability::Unknown, 0),
            };

            StoreResult {
                store_id: store.id().to_string(),
                store_name: store.name().to_string(),
                available,
                similar_count,
                browse_url: Some(store.browse_url(name)),
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: None,
            }
        }
        Err(e) => StoreResult {
            store_id: store.id().to_string(),
            store_name: store.name().to_string(),
            available: Availability::Unknown,
            similar_count: 0,
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
                    similar_count: 0,
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
                        similar_count: 0,
                        browse_url: Some(store.browse_url(name)),
                        elapsed_ms: start.elapsed().as_millis() as u64,
                        error: Some(e.to_string()),
                    };
                }
            };

            let (available, similar_count) = check_google_play_html(&body, name);

            StoreResult {
                store_id: store.id().to_string(),
                store_name: store.name().to_string(),
                available,
                similar_count,
                browse_url: Some(store.browse_url(name)),
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: None,
            }
        }
        Err(e) => StoreResult {
            store_id: store.id().to_string(),
            store_name: store.name().to_string(),
            available: Availability::Unknown,
            similar_count: 0,
            browse_url: Some(store.browse_url(name)),
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some(e.to_string()),
        },
    }
}

/// Check Google Play HTML for an exact app name match.
///
/// Returns (availability, similar_count) where similar_count is the
/// number of app results returned by the search.
///
/// Google Play server-renders search results. App titles appear in
/// elements with class "DdYX5". This is a best-effort heuristic —
/// the HTML structure may change, in which case this returns Unknown.
fn check_google_play_html(body: &str, name: &str) -> (Availability, usize) {
    if body.len() < 1000 {
        return (Availability::Unknown, 0);
    }

    // Count app detail links to get the number of search results.
    let similar_count = body.matches("/store/apps/details").count();

    if similar_count == 0 {
        return (Availability::Available, 0);
    }

    // Extract app titles from elements with class "DdYX5" (app name elements).
    let name_lower = name.to_lowercase();
    let class_marker = "DdYX5";
    let mut search_from = 0;
    let mut exact_match = false;

    while let Some(class_pos) = body[search_from..].find(class_marker) {
        let abs_pos = search_from + class_pos + class_marker.len();
        // Find the next '>' after the class attribute
        if let Some(gt_offset) = body[abs_pos..].find('>') {
            let text_start = abs_pos + gt_offset + 1;
            // Find the closing '<' for the text content
            if let Some(lt_offset) = body[text_start..].find('<') {
                let title = &body[text_start..text_start + lt_offset];
                let title = title.trim();
                if !title.is_empty() && title.to_lowercase() == name_lower {
                    exact_match = true;
                }
            }
        }
        search_from = abs_pos;
    }

    let availability = if exact_match {
        Availability::Taken
    } else {
        Availability::Available
    };
    (availability, similar_count)
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
