use crate::network::cache::RESOURCE_CACHE;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use futures::future::join_all;
use kuchikiki::traits::TendrilSink;
use std::collections::HashMap;
use tracing::warn;

const SRC_ATTRS: &[(&str, &str)] = &[
    ("img", "src"),
    ("script", "src"),
    ("source", "src"),
    ("audio", "src"),
    ("video", "src"),
    ("input", "src"),
    ("link", "href"),
];

pub struct RewriteResult {
    pub html: String,
    pub has_external: bool,
    pub failed: Vec<String>,
}

fn is_external(value: &str) -> bool {
    let v = value.trim();
    v.starts_with("http://") || v.starts_with("https://")
}

struct AnalysisResult {
    pub has_external: bool,
    pub urls: Vec<String>,
}

/// CPU-bound: parse HTML once, both detect external resources and collect URLs.
fn analyze_html(html: &str) -> AnalysisResult {
    let mut urls: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut has_external = false;

    let document = kuchikiki::parse_html().one(html).document_node;

    for &(tag, attr) in SRC_ATTRS {
        let selector = if attr == "href" {
            "link[rel='stylesheet'][href]".to_string()
        } else {
            format!("{}[{}]", tag, attr)
        };

        for node_data in document.select(&selector).into_iter().flatten() {
            if let Some(element) = node_data.as_node().as_element() {
                let attrs = element.attributes.borrow();
                if let Some(val) = attrs.get(attr) {
                    if is_external(val) {
                        has_external = true;
                        if seen.insert(val.to_string()) {
                            urls.push(val.to_string());
                        }
                    }
                }
            }
        }
    }

    collect_css_urls(html, &mut urls, &mut seen, &mut has_external);
    AnalysisResult { has_external, urls }
}

fn collect_css_urls(
    html: &str,
    urls: &mut Vec<String>,
    seen: &mut std::collections::HashSet<String>,
    has_external: &mut bool,
) {
    let mut rest = html;
    while let Some(pos) = rest.find("url(") {
        rest = &rest[pos + 4..];
        let Some(end) = rest.find(')') else { break };
        let inner = rest[..end]
            .trim()
            .trim_matches(|c| c == '\'' || c == '"')
            .trim();
        if is_external(inner) {
            *has_external = true;
            if seen.insert(inner.to_string()) {
                urls.push(inner.to_string());
            }
        }
        rest = &rest[end + 1..];
    }
}

/// CPU-bound: string replacement pass. Runs inside spawn_blocking.
fn apply_replacements(html: String, replacements: HashMap<String, String>) -> String {
    if replacements.is_empty() {
        return html;
    }

    let mut out = html;
    for (url, data_url) in &replacements {
        out = out.replace(url.as_str(), data_url.as_str());
        let encoded = url.replace('&', "&amp;");
        if encoded != *url {
            out = out.replace(encoded.as_str(), data_url.as_str());
        }
    }
    out
}

pub async fn rewrite_external_resources(html: &str, allow_external: bool) -> RewriteResult {
    // Single analysis pass - parse HTML once to both detect and collect external URLs
    let html_owned = html.to_string();
    let analysis = tokio::task::spawn_blocking(move || analyze_html(&html_owned))
        .await
        .unwrap_or(AnalysisResult {
            has_external: false,
            urls: Vec::new(),
        });

    if !allow_external || !analysis.has_external {
        return RewriteResult {
            html: html.to_string(),
            has_external: analysis.has_external,
            failed: vec![],
        };
    }

    let cache = match RESOURCE_CACHE.get() {
        Some(c) => c,
        None => {
            warn!("rewriter: resource cache not initialized, skipping rewrite");
            return RewriteResult {
                html: html.to_string(),
                has_external: analysis.has_external,
                failed: vec![],
            };
        }
    };

    let urls = analysis.urls;

    // Fetch all URLs in parallel — no more sequential waterfall
    let fetch_futures = urls.iter().map(|url| {
        let url = url.clone();
        async move {
            let result = cache.get_or_fetch(&url).await;
            (url, result)
        }
    });

    let results = join_all(fetch_futures).await;

    let mut replacements: HashMap<String, String> = HashMap::new();
    let mut failed: Vec<String> = Vec::new();

    for (url, result) in results {
        match result {
            Ok((data, mime)) => {
                let data_url = format!("data:{};base64,{}", mime, STANDARD.encode(data.as_ref()));
                replacements.insert(url, data_url);
            }
            Err(e) => {
                warn!("rewriter: failed to fetch {url}: {e}");
                failed.push(url);
            }
        }
    }

    // Offload string replacement (potentially large HTML) to blocking thread pool
    let html_owned = html.to_string();
    let rewritten =
        tokio::task::spawn_blocking(move || apply_replacements(html_owned, replacements))
            .await
            .unwrap_or_else(|_| html.to_string());

    RewriteResult {
        html: rewritten,
        has_external: analysis.has_external,
        failed,
    }
}
