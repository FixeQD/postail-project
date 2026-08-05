use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Gmail,
    Outlook,
}

impl ProviderKind {
    pub fn all() -> &'static [ProviderKind] {
        &[ProviderKind::Gmail, ProviderKind::Outlook]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::Gmail => "gmail",
            ProviderKind::Outlook => "outlook",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "gmail" => Some(ProviderKind::Gmail),
            "outlook" => Some(ProviderKind::Outlook),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderKind::Gmail => "Gmail",
            ProviderKind::Outlook => "Outlook",
        }
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl ProviderKind {
    pub fn from_imap_host(host: &str) -> Option<Self> {
        match host {
            "imap.gmail.com" => Some(ProviderKind::Gmail),
            "outlook.office365.com" => Some(ProviderKind::Outlook),
            _ => None,
        }
    }

    /// Extract provider kind from OAuth state string
    /// State format: "{provider}:{random}"
    pub fn from_state(state: &str) -> Option<Self> {
        state.split(':').next().and_then(Self::parse)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInfo {
    pub kind: ProviderKind,
    pub name: &'static str,
    pub auth_url: &'static str,
    pub token_url: &'static str,
    pub scopes: &'static str,
    pub imap_host: &'static str,
    pub smtp_host: &'static str,
    pub canonical_prefix: Option<&'static str>,
    // IMAP Connection Pool configuration
    pub max_idle_connections: usize,
    pub idle_timeout_seconds: u64,
    pub poll_interval_seconds: u64,
    pub rebalance_interval_seconds: u64,
    pub stale_threshold_seconds: u64,
    pub hot_threshold_seconds: u64,
}

impl ProviderInfo {
    pub fn get(kind: ProviderKind) -> Self {
        match kind {
            ProviderKind::Gmail => ProviderInfo {
                kind,
                name: "Gmail",
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth",
                token_url: "https://oauth2.googleapis.com/token",
                scopes: "https://mail.google.com/ https://www.googleapis.com/auth/gmail.send https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/gmail.readonly",
                imap_host: "imap.gmail.com",
                smtp_host: "smtp.gmail.com",
                canonical_prefix: Some("[Gmail]/"),
                // Gmail limits: 15 connections per account, leave buffer
                max_idle_connections: 5,
                idle_timeout_seconds: 29 * 60, // RFC 2177
                poll_interval_seconds: 60,
                rebalance_interval_seconds: 5 * 60,
                stale_threshold_seconds: 29 * 60,
                hot_threshold_seconds: 5 * 60,
            },
            ProviderKind::Outlook => ProviderInfo {
                kind,
                name: "Outlook",
                auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
                token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token",
                scopes: "https://outlook.office.com/IMAP.AccessAsUser.All https://outlook.office.com/SMTP.Send",
                imap_host: "outlook.office365.com",
                smtp_host: "smtp-mail.outlook.com",
                canonical_prefix: None,
                // Outlook limits: 4 concurrent connections per account
                max_idle_connections: 3,
                idle_timeout_seconds: 29 * 60, // RFC 2177
                poll_interval_seconds: 60,
                rebalance_interval_seconds: 5 * 60,
                stale_threshold_seconds: 29 * 60,
                hot_threshold_seconds: 5 * 60,
            },
        }
    }

    pub fn client_id(&self) -> Option<String> {
        match self.kind {
            ProviderKind::Gmail => option_env!("GMAIL_PKCE_ID").map(|s| s.to_string()),
            ProviderKind::Outlook => option_env!("OUTLOOK_PKCE_ID").map(|s| s.to_string()),
        }
    }

    pub fn client_secret(&self) -> Option<String> {
        match self.kind {
            ProviderKind::Gmail => option_env!("GMAIL_PKCE_SECRET").map(|s| s.to_string()),
            ProviderKind::Outlook => option_env!("OUTLOOK_PKCE_SECRET").map(|s| s.to_string()),
        }
    }

    pub fn user_info_url(&self) -> &'static str {
        match self.kind {
            ProviderKind::Gmail => "https://www.googleapis.com/oauth2/v2/userinfo",
            ProviderKind::Outlook => "https://graph.microsoft.com/v1.0/me",
        }
    }

    pub fn extract_email(&self, user_info: &serde_json::Value) -> Option<String> {
        match self.kind {
            ProviderKind::Gmail => user_info["email"].as_str().map(|s| s.to_string()),
            ProviderKind::Outlook => user_info["mail"]
                .as_str()
                .or_else(|| user_info["userPrincipalName"].as_str())
                .map(|s| s.to_string()),
        }
    }

    pub fn strip_display_name_prefix(&self, name: &str) -> String {
        match self.canonical_prefix {
            Some(prefix) => name.strip_prefix(prefix).unwrap_or(name).to_string(),
            None => name.to_string(),
        }
    }
}
