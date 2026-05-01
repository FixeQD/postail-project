//! Shared regex patterns for CSS inlining.

use regex::Regex;
use std::sync::LazyLock;

pub static STYLE_BLOCK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)(<style[^>]*>)(.*?)(</style>)").unwrap());

pub static ANIM_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"animation(?:-name)?\s*:\s*([a-zA-Z_][\w-]*)").unwrap());

pub static STYLE_ATTR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"style="([^"]*)"#).unwrap());

pub static ANIM_PROP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"animation[^:]*:\s*[^;]+;?").unwrap());

pub static TO_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s|;|\})(to|100\s*%)\s*\{").unwrap());

pub static CLAMP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"clamp\(\s*([^,]+)\s*,\s*([^,]+)\s*,\s*([^)]+)\s*\)").unwrap());
