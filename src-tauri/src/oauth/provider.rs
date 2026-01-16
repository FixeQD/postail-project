use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Gmail,
    Outlook,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::Gmail => "gmail",
            ProviderKind::Outlook => "outlook",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInfo {
    pub kind: ProviderKind,
    pub name: &'static str,
    pub auth_url: &'static str,
    pub token_url: &'static str,
    pub scopes: &'static str,
    pub imap_host: &'static str,
    pub smtp_host: &'static str,
    pub sent_folder: &'static str,
}

impl ProviderInfo {
    pub fn get(kind: ProviderKind) -> Self {
        match kind {
            ProviderKind::Gmail => ProviderInfo {
                kind,
                name: "Gmail",
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth",
                token_url: "https://oauth2.googleapis.com/token",
                scopes: "https://mail.google.com/ https://www.googleapis.com/auth/userinfo.email",
                imap_host: "imap.gmail.com",
                smtp_host: "smtp.gmail.com",
                sent_folder: "[Gmail]/Sent Mail",
            },
            ProviderKind::Outlook => ProviderInfo {
                kind,
                name: "Outlook",
                auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
                token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token",
                scopes: "https://outlook.office.com/IMAP.AccessAsUser.All https://outlook.office.com/SMTP.Send",
                imap_host: "outlook.office365.com",
                smtp_host: "smtp-mail.outlook.com",
                sent_folder: "Sent Items",
            },
        }
    }

    pub fn client_id(&self) -> Option<String> {
        match self.kind {
            ProviderKind::Gmail => option_env!("GMAIL_CLIENT_ID").map(|s| s.to_string()),
            ProviderKind::Outlook => option_env!("OUTLOOK_CLIENT_ID").map(|s| s.to_string()),
        }
    }

    pub fn client_secret(&self) -> Option<String> {
        match self.kind {
            ProviderKind::Gmail => option_env!("GMAIL_CLIENT_SECRET").map(|s| s.to_string()),
            ProviderKind::Outlook => option_env!("OUTLOOK_CLIENT_SECRET").map(|s| s.to_string()),
        }
    }
}
