// ── CSS class constants (shared) ────────────────────────────────────────────
pub const NO_UNDERLINE: &str = "no-underline";
pub const HOVER_UNDERLINE: &str = "hover:underline";
pub const BADGE_BLUE_NO_UL: &str = "badge-blue no-underline";
pub const BADGE_GRAY: &str = "badge-gray";
pub const BADGE_GRAY_NO_UL: &str = "badge-gray no-underline";
#[cfg(feature = "oth")]
pub const BADGE_GREEN: &str = "badge-green";
#[cfg(feature = "oth")]
pub const BADGE_RED: &str = "badge-red";

// ── Shared utility class combinations ────────────────────────────────────
pub const TEXT_MUTED: &str = "text-gray-600 dark:text-gray-300";
pub const TEXT_SUBTLE: &str = "text-gray-500 dark:text-gray-400";
pub const TEXT_XS_MUTED: &str = "text-xs text-gray-400";
pub const TEXT_WARN: &str = "text-xs text-red-400 dark:text-red-500";
pub const FLEX_BETWEEN: &str = "flex items-center justify-between";
pub const BG_CARD: &str = "bg-white dark:bg-gray-800";
pub const MAIN: &str = "max-w-4xl mx-auto px-4 py-8";
pub const WIDE: &str = "max-w-7xl mx-auto px-4 py-8";
pub const GRID_2: &str = "grid grid-cols-1 md:grid-cols-2 gap-4";
pub const GRID_3: &str = "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4";
pub const HOVER_SHADOW: &str = "hover:shadow-md transition-shadow";
pub const EMPTY: &str = "text-center py-16";
pub const CARD_SECTION: &str = "card p-6 mb-6";
pub const H1: &str = "text-xl font-bold text-gray-800 dark:text-gray-100 mb-6";
pub const NO_DATA: &str = "text-xl text-gray-800 dark:text-gray-100 mb-4";
pub const SECTION_H2: &str = "text-base font-semibold text-gray-700 dark:text-gray-200 mb-4";
pub const FLEX_WRAP_GAP: &str = "flex flex-wrap gap-2";
pub const CAT_BTN: &str = "text-sm text-gray-600 dark:text-gray-200 px-2 py-0.5 border border-gray-300 dark:border-gray-600 rounded bg-transparent hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors";
pub const CAT_BTN_MORE: &str = "text-sm text-blue-600 dark:text-blue-400 border border-blue-300 dark:border-blue-600 rounded bg-transparent px-2 py-0.5 hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-colors";

// ── Slide panel classes ───────────────────────────────────────────────────
pub const SLIDE_OVERLAY: &str = "slide-overlay";
pub const SLIDE_PANEL: &str = "slide-panel";
pub const SLIDE_OPEN: &str = "open";
pub const SLIDE_CLOSE: &str = "slide-close";
pub const SLIDE_BODY: &str = "slide-body";
pub const SLIDE_SIZED_SM: &str = "slide-sized-sm";
pub const SLIDE_SIZED_MD: &str = "slide-sized-md";
pub const SLIDE_SIZED_LG: &str = "slide-sized-lg";

// ── Config (SSR only) ──────────────────────────────────────────────────────
#[cfg(feature = "ssr")]
use std::sync::LazyLock;

#[cfg(feature = "ssr")]
pub struct Config {
    pub domain: String,
    pub site_key: String,
    pub claim_exp: usize,
    pub page_size: i64,
    pub db_url: String,
    pub db_ns: String,
    pub db_name: String,
    pub db_user: String,
    pub db_pass: String,
    pub email_smtp: String,
    pub email_from: String,
    pub email_username: String,
    pub email_password: String,
}

#[cfg(feature = "ssr")]
static CFG: LazyLock<Config> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    Config {
        domain: env("DOMAIN"),
        site_key: env("SITE_KEY"),
        claim_exp: now() + parse::<usize>("CLAIM_EXP"),
        page_size: parse::<i64>("PAGE_SIZE"),
        db_url: env("DB_URL"),
        db_ns: env("DB_NS"),
        db_name: env("DB_NAME"),
        db_user: env("DB_USER"),
        db_pass: env("DB_PASS"),
        email_smtp: env("EMAIL_SMTP"),
        email_from: env("EMAIL_FROM"),
        email_username: env("EMAIL_USERNAME"),
        email_password: env("EMAIL_PASSWORD"),
    }
});

#[cfg(feature = "ssr")]
pub fn config() -> &'static Config {
    &CFG
}

// ── helpers (SSR only) ──────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{key} must be set"))
}

#[cfg(feature = "ssr")]
fn parse<T: std::str::FromStr>(key: &str) -> T
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    std::env::var(key)
        .unwrap_or_else(|_| panic!("{key} must be set"))
        .parse()
        .unwrap_or_else(|e| panic!("{key} must be a valid integer: {e}"))
}

#[cfg(feature = "ssr")]
fn now() -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}
