//! R9.64 transport-neutral commercial-path Site/POP registry BFF.
//!
//! Intentionally UNMOUNTED: the existing R8.8 BrowserSessionVault is
//! in-memory and has NO live public IdP/MFA, trusted Host→tenant membership or
//! HA session runtime. The future HTTPS handler MUST prove those boundaries
//! BEFORE passing its server-derived tenant UUID here. This module rechecks
//! authorization on EVERY SQL call and has ZERO device/host executor access.
use identity_core::browser_session::{BrowserSessionVault, RequestKind};
use tokio_postgres::Client;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct SiteRow {
    pub code: String,
    pub display_name: String,
    pub revision: i64,
    pub assigned_devices: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct SitePage {
    pub sites: Vec<SiteRow>,
    pub next_after: Option<String>,
}

fn valid_code(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= 128
        && code
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}
fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 120
        && name == name.trim()
        && !name.chars().any(char::is_control)
}

pub(super) async fn create_for_session(
    sessions: &mut BrowserSessionVault,
    cookie: &str,
    csrf: &str,
    origin_verified: bool,
    db: &Client,
    verified_host_tenant: Uuid,
    code: &str,
    display_name: &str,
    now: u64,
) -> Option<String> {
    let actor = sessions.authenticate(
        cookie,
        Some(csrf),
        RequestKind::Mutation,
        origin_verified,
        now,
    )?;
    if !valid_code(code) || !valid_name(display_name) {
        return None;
    }
    let row = db
        .query_one(
            "SELECT ipat_platform.create_tenant_site($1,$2,$3::uuid,$4,$5)",
            &[
                &actor.issuer(),
                &actor.subject(),
                &verified_host_tenant,
                &code,
                &display_name,
            ],
        )
        .await
        .ok()?;
    // Database independently verifies CURRENT tenant_admin membership;
    // identical Site code in another tenant cannot grant any authority.
    let saved: Option<String> = row.get(0);
    saved.filter(|value| value == code)
}

pub(super) async fn rename_for_session(
    sessions: &mut BrowserSessionVault,
    cookie: &str,
    csrf: &str,
    origin_verified: bool,
    db: &Client,
    verified_host_tenant: Uuid,
    code: &str,
    display_name: &str,
    expected_revision: i64,
    now: u64,
) -> Option<i64> {
    let actor = sessions.authenticate(
        cookie,
        Some(csrf),
        RequestKind::Mutation,
        origin_verified,
        now,
    )?;
    if !valid_code(code) || !valid_name(display_name) || expected_revision < 1 {
        return None;
    }
    let row = db
        .query_one(
            "SELECT ipat_platform.rename_tenant_site($1,$2,$3::uuid,$4,$5,$6::bigint)",
            &[
                &actor.issuer(),
                &actor.subject(),
                &verified_host_tenant,
                &code,
                &display_name,
                &expected_revision,
            ],
        )
        .await
        .ok()?;
    row.get::<_, Option<i64>>(0)
        .filter(|next| *next > expected_revision)
}

pub(super) async fn delete_for_session(
    sessions: &mut BrowserSessionVault,
    cookie: &str,
    csrf: &str,
    origin_verified: bool,
    db: &Client,
    verified_host_tenant: Uuid,
    code: &str,
    expected_revision: i64,
    now: u64,
) -> bool {
    let Some(actor) = sessions.authenticate(
        cookie,
        Some(csrf),
        RequestKind::Mutation,
        origin_verified,
        now,
    ) else {
        return false;
    };
    if !valid_code(code) || expected_revision < 1 {
        return false;
    }
    db.query_one(
        "SELECT ipat_platform.delete_tenant_site($1,$2,$3::uuid,$4,$5::bigint)",
        &[
            &actor.issuer(),
            &actor.subject(),
            &verified_host_tenant,
            &code,
            &expected_revision,
        ],
    )
    .await
    .ok()
    .is_some_and(|row| row.get::<_, bool>(0))
}

pub(super) async fn list_for_session(
    sessions: &mut BrowserSessionVault,
    cookie: &str,
    origin_verified: bool,
    db: &Client,
    verified_host_tenant: Uuid,
    exact_pop_scope: Option<&str>,
    after: Option<&str>,
    now: u64,
) -> Option<SitePage> {
    let actor = sessions.authenticate(cookie, None, RequestKind::Read, origin_verified, now)?;
    if exact_pop_scope.is_some_and(|pop| !valid_code(pop))
        || after.is_some_and(|cursor| !valid_code(cursor))
    {
        return None;
    }
    let rows = db
        .query(
            "SELECT code,display_name,revision,assigned_devices \
             FROM ipat_platform.list_tenant_sites($1,$2,$3::uuid,$4,$5)",
            &[
                &actor.issuer(),
                &actor.subject(),
                &verified_host_tenant,
                &exact_pop_scope,
                &after,
            ],
        )
        .await
        .ok()?;
    if rows.len() > 101 {
        return None;
    }
    let has_next = rows.len() == 101;
    let mut sites = Vec::with_capacity(rows.len().min(100));
    for row in rows.into_iter().take(100) {
        let site = SiteRow {
            code: row.get(0),
            display_name: row.get(1),
            revision: row.get(2),
            assigned_devices: row.get(3),
        };
        if !valid_code(&site.code)
            || !valid_name(&site.display_name)
            || site.revision < 1
            || site.assigned_devices < 0
        {
            return None;
        }
        sites.push(site);
    }
    let next_after = if has_next {
        sites.last().map(|site| site.code.clone())
    } else {
        None
    };
    Some(SitePage { sites, next_after })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reject_invalid_site_ids_before_database_query() {
        for value in [
            "",
            "../site",
            "other/site",
            "with space",
            "site\n",
            "∑-site",
        ] {
            assert!(!valid_code(value), "unexpected allowed: {value:?}");
        }
        assert!(valid_code("JKT-CORE_01"));
        assert!(valid_code(&"a".repeat(128)));
        assert!(!valid_code(&"a".repeat(129)));
    }
    #[test]
    fn no_control_text_or_whitespace_site_names() {
        for value in [
            "",
            " Insecure",
            "Trailing ",
            "bad\nline",
            "bad\u{0000}control",
        ] {
            assert!(!valid_name(value));
        }
        assert!(valid_name("Core POP Jakarta"));
        assert!(!valid_name(&"x".repeat(121)));
    }
}
