//! Compile-time build metadata injected by build.rs.

pub const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");
pub const GIT_HASH: &str = env!("GIT_HASH");
pub const GIT_BRANCH: &str = env!("GIT_BRANCH");
pub const BUILD_PROFILE: &str = env!("BUILD_PROFILE");
pub const RUSTC_VERSION: &str = env!("RUSTC_VERSION");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(serde::Serialize)]
pub struct BuildInfo {
    pub version: &'static str,
    pub build_timestamp: &'static str,
    pub git_hash: &'static str,
    pub git_branch: &'static str,
    pub profile: &'static str,
    pub rustc: &'static str,
}

pub fn get() -> BuildInfo {
    BuildInfo {
        version: APP_VERSION,
        build_timestamp: BUILD_TIMESTAMP,
        git_hash: GIT_HASH,
        git_branch: GIT_BRANCH,
        profile: BUILD_PROFILE,
        rustc: RUSTC_VERSION,
    }
}
