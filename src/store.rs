use crate::types::StoreInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Store {
    AppleAppStore,
    GooglePlay,
}

pub const ALL_STORES: [Store; 2] = [Store::AppleAppStore, Store::GooglePlay];

impl Store {
    pub fn id(&self) -> &'static str {
        match self {
            Store::AppleAppStore => "app_store",
            Store::GooglePlay => "google_play",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Store::AppleAppStore => "App Store",
            Store::GooglePlay => "Google Play",
        }
    }

    pub fn platform(&self) -> &'static str {
        match self {
            Store::AppleAppStore => "iOS / macOS",
            Store::GooglePlay => "Android",
        }
    }

    pub fn browse_url(&self, name: &str) -> String {
        match self {
            Store::AppleAppStore => {
                format!("https://apps.apple.com/search?term={name}")
            }
            Store::GooglePlay => {
                format!("https://play.google.com/store/search?q={name}&c=apps")
            }
        }
    }

    pub fn info(&self) -> StoreInfo {
        StoreInfo {
            id: self.id().to_string(),
            name: self.name().to_string(),
            platform: self.platform().to_string(),
        }
    }
}

pub fn all_stores() -> &'static [Store] {
    &ALL_STORES
}

pub fn stores_by_ids(ids: &[String]) -> Vec<Store> {
    ALL_STORES
        .iter()
        .filter(|s| ids.iter().any(|id| id == s.id()))
        .copied()
        .collect()
}
