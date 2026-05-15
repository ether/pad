use std::time::Duration;

pub struct CommunityInstance {
    pub url: String,
    pub label: String,
}

const FALLBACK_INSTANCES: &[(&str, &str)] = &[
    ("https://yopad.eu", "yopad.eu"),
    ("https://pad.disroot.org", "pad.disroot.org"),
    ("https://etherpad.wikimedia.org", "etherpad.wikimedia.org"),
    ("https://pad.riseup.net", "pad.riseup.net"),
];

pub async fn fetch_or_fallback() -> Vec<CommunityInstance> {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => return fallback(),
    };
    let resp = match client
        .get("https://scanner.etherpad.org/instances.json")
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return fallback(),
    };
    let body: serde_json::Value = match resp.json().await {
        Ok(b) => b,
        Err(_) => return fallback(),
    };
    let Some(arr) = body.as_array() else {
        return fallback();
    };
    let mut out = Vec::new();
    for item in arr {
        let Some(url) = item["url"].as_str() else {
            continue;
        };
        let label = item["name"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| url.to_string());
        out.push(CommunityInstance {
            url: url.to_string(),
            label,
        });
    }
    if out.is_empty() { fallback() } else { out }
}

fn fallback() -> Vec<CommunityInstance> {
    FALLBACK_INSTANCES
        .iter()
        .map(|(url, label)| CommunityInstance {
            url: (*url).into(),
            label: (*label).into(),
        })
        .collect()
}
