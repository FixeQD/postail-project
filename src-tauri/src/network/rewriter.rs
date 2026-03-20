use crate::network::cache::RESOURCE_CACHE;
use kuchiki::traits::TendrilSink;
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

/// Collect all external URLs that need to be fetched from the HTML string.
/// Returns a deduplicated list
fn collect_external_urls(html: &str) -> Vec<String> {
    let mut urls: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let document = kuchiki::parse_html().one(html);

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
                    if is_external(val) && seen.insert(val.to_string()) {
                        urls.push(val.to_string());
                    }
                }
            }
        }
    }

    // Collect url() from style attributes and <style> elements via text scan
    collect_css_urls(html, &mut urls, &mut seen);

    urls
}

/// Scan raw HTML text for url(http...) occurrences
fn collect_css_urls(
    html: &str,
    urls: &mut Vec<String>,
    seen: &mut std::collections::HashSet<String>,
) {
    let mut rest = html;
    while let Some(pos) = rest.find("url(") {
        rest = &rest[pos + 4..];
        let Some(end) = rest.find(')') else { break };
        let inner = rest[..end]
            .trim()
            .trim_matches(|c| c == '\'' || c == '"')
            .trim();
        if is_external(inner) && seen.insert(inner.to_string()) {
            urls.push(inner.to_string());
        }
        rest = &rest[end + 1..];
    }
}

/// Replace all occurrences of external URLs in raw HTML with their data: equivalents.
fn apply_replacements(html: &str, replacements: &HashMap<String, String>) -> String {
    if replacements.is_empty() {
        return html.to_string();
    }

    let mut out = html.to_string();
    for (url, data_url) in replacements {
        // Replace decoded form (e.g. from CSS url() or already-decoded attrs)
        out = out.replace(url.as_str(), data_url.as_str());
        // Replace HTML-entity-encoded form (& → &amp;) as found in raw HTML attributes
        let encoded = url.replace('&', "&amp;");
        if encoded != *url {
            out = out.replace(encoded.as_str(), data_url.as_str());
        }
    }
    out
}

fn detect_external(html: &str) -> bool {
    let document = kuchiki::parse_html().one(html);

    for &(tag, attr) in SRC_ATTRS {
        let selector = if attr == "href" {
            "link[rel='stylesheet'][href]".to_string()
        } else {
            format!("{}[{}]", tag, attr)
        };

        if document.select(&selector).into_iter().flatten().any(|n| {
            n.as_node()
                .as_element()
                .and_then(|e| e.attributes.borrow().get(attr).map(|v| is_external(v)))
                .unwrap_or(false)
        }) {
            return true;
        }
    }

    html.contains("url(http://")
        || html.contains("url(https://")
        || html.contains("url('http")
        || html.contains("url('https")
        || html.contains("url(\"http")
        || html.contains("url(\"https")
}

pub async fn rewrite_external_resources(html: &str, allow_external: bool) -> RewriteResult {
    let has_external = detect_external(html);

    if !allow_external || !has_external {
        return RewriteResult {
            html: html.to_string(),
            has_external,
            failed: vec![],
        };
    }

    let cache = match RESOURCE_CACHE.get() {
        Some(c) => c,
        None => {
            warn!("rewriter: resource cache not initialized, skipping rewrite");
            return RewriteResult {
                html: html.to_string(),
                has_external,
                failed: vec![],
            };
        }
    };

    // --- Sync phase: collect all URLs, drop all DOM references ---
    let urls = collect_external_urls(html);

    // --- Async phase: fetch everything, no DOM refs in scope ---
    let mut replacements: HashMap<String, String> = HashMap::new();
    let mut failed: Vec<String> = Vec::new();

    for url in urls {
        match cache.get_or_fetch(&url).await {
            Ok((data, mime)) => {
                use base64::{engine::general_purpose::STANDARD, Engine as _};
                let data_url = format!("data:{};base64,{}", mime, STANDARD.encode(data.as_ref()));
                replacements.insert(url, data_url);
            }
            Err(e) => {
                warn!("rewriter: failed to fetch {url}: {e}");
                failed.push(url);
            }
        }
    }

    // --- Sync phase: apply replacements to raw HTML string ---
    let rewritten = apply_replacements(html, &replacements);

    RewriteResult {
        html: rewritten,
        has_external,
        failed,
    }
}
