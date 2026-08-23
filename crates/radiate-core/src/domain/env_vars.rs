use std::sync::LazyLock;

static ENV_CONFIG: LazyLock<EnvConfig> = LazyLock::new(EnvConfig::from_env);

#[derive(Debug, Clone)]
pub struct EnvConfig {
    log_enabled: Option<bool>,
    log_level: Option<String>,
    log_format: Option<String>,
    seed: Option<u64>,
}

impl EnvConfig {
    fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            log_enabled: std::env::var("RADIATE_LOG_ENABLED").ok().and_then(|s| {
                let parsed = s.parse().ok();
                if parsed.is_none() {
                    eprintln!("RADIATE_LOG_ENABLED={s:?} is not a valid bool, ignoring");
                }
                parsed
            }),
            log_level: std::env::var("RADIATE_LOG_LEVEL")
                .or_else(|_| std::env::var("RUST_LOG"))
                .ok(),
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

pub fn log_enabled() -> Option<bool> {
    ENV_CONFIG.log_enabled
}

pub fn log_level() -> Option<String> {
    ENV_CONFIG.log_level.clone()
}

pub fn log_format() -> Option<String> {
    ENV_CONFIG.log_format.clone()
}

pub fn seed() -> Option<u64> {
    ENV_CONFIG.seed
}
