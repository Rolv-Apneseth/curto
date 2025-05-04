use axum_prometheus::metrics::counter;
use block_id::{Alphabet, BlockId};
use chrono::{NaiveDateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use strum::IntoEnumIterator;
use utoipa::ToSchema;

use crate::{error::{Error, Result}, routes::Route, utils::get_default_db_timeout};

const CHARS: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    /// ID of the shortened link.
    pub id: String,
    /// URL that the shortened link will redirect to.
    pub target_url: String,
    /// Count of successful redirects to [`Self::target_url`].
    pub count_redirects: i64,
    /// Shortened link creation time.
    pub created_at: NaiveDateTime,
    /// Shortened link last modification time.
    pub updated_at: NaiveDateTime,
}

impl Link {
    pub fn new(id: Option<String>, target_url: String) -> Self {
        let id = id.unwrap_or_else(Link::generate_id);
        let now = Utc::now().naive_utc();

        Link {
            id,
            target_url,
            count_redirects: 0,
            created_at: now,
            updated_at: now,
        }
    }

    fn generate_id() -> String {
        // TODO: investigate whether this is safe to do - is there any issue with being
        // able to work out the random number from the link ID?
        const SEED: u128 = 1234;
        const LENGTH: u8 = 5;

        let random_number = u64::from(rand::rng().random_range(0..u32::MAX));
        let id = BlockId::new(Alphabet::alphanumeric(), SEED, LENGTH)
            .encode_string(random_number)
            .expect("could not encode random number as the short link ID");

        for _ in 0..5 {
            if Link::validate_id(&id) {
                return id;
            }
        }
        unreachable!("something wrong with ID generation");
    }

    fn validate_id(id: &str) -> bool {
        !id.is_empty()
        // Does not contain non-alphanumeric characters
        && !id.chars().any(|c| !CHARS.chars().any(|cc| c == cc))
        // Does not match any existing routes
        && !Route::iter().any(|r| {
                let mut r = r.as_str().trim_start_matches("/");
                if r.contains("/") {
                    r = r.split_once("/").expect("should contain /").0;
                };

                id.to_lowercase() == r.to_lowercase()
        })
    }
}

pub async fn create_link(
    db: &Pool<Postgres>,
    link_id: Option<String>,
    link_target: String,
) -> Result<Link> {
    // User provided invalid link ID
    if let Some(id) = link_id.as_ref() {
        if !Link::validate_id(id) {
            return Err(Error::LinkIdNotValid(id.clone()));
        }
    };

    tokio::time::timeout(
        get_default_db_timeout(),
        sqlx::query_as!(
            Link,
            r#"
                insert into links(id, target_url)
                values ($1, $2)
                returning *
            "#,
            link_id.clone().unwrap_or_else(Link::generate_id),
            link_target,
        )
        .fetch_one(db),
    )
    .await
    .inspect_err(|_| counter!("db.connection_timeout").increment(1))?
    .map_err(|e| {
        // Provided custom ID already exists in the database
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.kind() == sqlx::error::ErrorKind::UniqueViolation && link_id.is_some() {
                counter!("db.user_provided_taken_id").increment(1);
                return Error::LinkIdNotUnique(link_id.unwrap());
            }
        }

        counter!("db.saving_link_impossible").increment(1);
        e.into()
    })
}

/// Find an existing [`Link`] in the database with the given ID.
pub async fn get_link(db: &Pool<Postgres>, link_id: impl AsRef<str>) -> Result<Option<Link>> {
    tokio::time::timeout(
        get_default_db_timeout(),
        sqlx::query_as!(
            Link,
            r#"select * from links where id = $1"#,
            link_id.as_ref()
        )
        .fetch_optional(db),
    )
    .await
    .inspect_err(|_| counter!("db.connection_timeout").increment(1))?
    .inspect_err(|_| counter!("db.failed_to_lookup_link").increment(1))
    .map_err(Error::from)
}

/// Get all existing [`Link`]s in the database.
pub async fn get_links(db: &Pool<Postgres>) -> Result<Vec<Link>> {
    tokio::time::timeout(
        get_default_db_timeout(),
        sqlx::query_as!(Link, r#"select * from links"#,).fetch_all(db),
    )
    .await
    .inspect_err(|_| counter!("db.connection_timeout").increment(1))?
    .inspect_err(|_| counter!("db.failed_to_lookup_link").increment(1))
    .map_err(Error::from)
}

/// Increment [`Link::count_redirects`], returning [`None`] if no link with the
/// given ID was found.
pub async fn increment_link_redirect_count(
    db: &Pool<Postgres>,
    link_id: impl AsRef<str>,
) -> Result<Option<Link>> {
    let link = tokio::time::timeout(
        get_default_db_timeout(),
        sqlx::query_as!(
            Link,
            r#"
                update links set count_redirects = count_redirects + 1
                where id = $1
                returning *
            "#,
            link_id.as_ref()
        )
        .fetch_optional(db),
    )
    .await
    .inspect_err(|e| {
        counter!("db.failed_to_increment_link").increment(1);
        tracing::error!(
            "Incrementing link redirect count resulted in a timeout: {}",
            e
        );
    })?
    .inspect_err(|e| {
        counter!("db.failed_to_increment_link").increment(1);
        tracing::error!(
            "Incrementing link redirect count resulted in the following error: {}",
            e
        );
    })?;

    if link.is_some() {
        tracing::debug!(
            "Incremented redirect count for link with ID {}",
            link_id.as_ref(),
        );
    }

    Ok(link)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_validate_link_id() {
        assert!(!Link::validate_id(""));
        assert!(!Link::validate_id("/"));
        assert!(!Link::validate_id("/abc"));
        assert!(!Link::validate_id("abc-xyz"));
        assert!(!Link::validate_id("😥"));
        assert!(!Link::validate_id(Route::Docs.as_str()));
        assert!(!Link::validate_id(Route::Health.as_str()));
        assert!(!Link::validate_id("health"));
        assert!(!Link::validate_id("Health"));
        assert!(!Link::validate_id("links"));
        assert!(!Link::validate_id("lInKs"));

        assert!(Link::validate_id("abc"));
        assert!(Link::validate_id("alkw13"));
        assert!(Link::validate_id("BAD"));
    }

    #[test]
    fn test_generate_link_id() {
        for _ in 0..1000 {
            let id = Link::generate_id();
            assert!(!id.is_empty());
            assert!(!id.len() >= 4);
            assert!(Link::validate_id(&id));
        }
    }
}
