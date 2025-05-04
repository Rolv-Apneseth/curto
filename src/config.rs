use serde::{Deserialize, Deserializer, de::Error};
use url::Url;

/// Configuration options for the application.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub application: AppConfig,
    pub database: DbConfig,
}

/// Configuration options specific to the main application.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AppConfig {
    /// The network host that the application should run on.
    ///
    /// The default is `[0,0,0,0]`
    #[serde(deserialize_with = "deserialize_host")]
    pub host: [u8; 4],
    /// The network port that the application should run on.
    ///
    /// The default is 7229.
    pub port: u16,
    /// Whether the application should enable rate-limiting.
    ///
    /// Rate-limiting should only be disabled for testing.
    pub shouldratelimit: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: [0, 0, 0, 0],
            port: 7229,
            shouldratelimit: true,
        }
    }
}

/// Configuration options specific to the database.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct DbConfig {
    /// The fully qualified URL used to connect to the PostgreSQL database.
    pub url: Url,
    /// Whether to required SSL mode for the database.
    pub requiressl: bool,
}

impl Config {
    // Build configuration from env vars
    pub fn get_config() -> Result<Self, config::ConfigError> {
        // Try to load env vars from a `.env` file, without overriding existing
        // variables
        let _ = dotenvy::dotenv();

        config::Config::builder()
            // Will match env vars like this: `CURTO__APPLICATION_URL`
            .add_source(
                config::Environment::with_prefix("")
                    .prefix_separator("")
                    .separator("_"),
            )
            .build()?
            .try_deserialize::<Config>()
    }
}

/// Custom de-serialiser for the host, converting a string value to `[u8; 4]`
fn deserialize_host<'de, D>(deserializer: D) -> Result<[u8; 4], D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let split: Vec<&str> = s.split('.').collect();

    if split.len() != 4 {
        return Err(D::Error::custom(
            "Invalid host -> value needs to be provided in the format '0.0.0.0', with 4 period-separated numbers between 0 and 255.",
        ));
    }

    let mut res: [u8; 4] = [0, 0, 0, 0];
    for (i, s) in split.into_iter().enumerate() {
        res[i] = match s.parse::<u8>() {
            Ok(n) => n,
            Err(e) => {
                return Err(D::Error::custom(format!(
                    "Invalid host -> error parsing one of the period-separated numbers for the host - ensure all values are within 0-255.\n\
                    Error encountered: {e}"
                )));
            }
        };
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    // UNIT TESTS - HELPERS
    #[test]
    fn test_get_config() {
        assert!(Config::get_config().is_ok());
    }
}
