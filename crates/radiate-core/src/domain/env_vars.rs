use std::sync::LazyLock;

pub const COMPACT_LOGGING: &str = "compact";
pub const JSON_LOGGING: &str = "json";
pub const DEFAULT_LOG_LEVEL: &str = "info";

static ENV_CONFIG: LazyLock<EnvConfig> = LazyLock::new(EnvConfig::from_env);

#[derive(Debug, Clone)]
pub struct EnvConfig {
    log_format: Option<String>,
    seed: Option<u64>,
}

impl EnvConfig {
    fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            log_format: std::env::var("RADIATE_LOG_FORMAT").ok(),
            seed: std::env::var("RADIATE_SEED").ok().and_then(|s| {
                let parsed = s.parse().ok();
                if parsed.is_none() {
                    eprintln!("RADIATE_SEED={s:?} is not a valid u64, ignoring");
                }
                parsed
            }),
        }
    }
}

pub fn config() -> &'static EnvConfig {
    &ENV_CONFIG
}

pub fn log_format() -> Option<String> {
    ENV_CONFIG.log_format.clone()
}

pub fn seed() -> Option<u64> {
    ENV_CONFIG.seed
}
