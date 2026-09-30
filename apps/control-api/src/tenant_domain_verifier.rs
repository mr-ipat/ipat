//! R9.55 automatic DNS TXT ownership verifier.
//!
//! This worker advances ONLY the ownership gate. Routing/TLS/activation remain
//! separate evidence steps. DNS evidence never grants tenant authorization.

use hickory_resolver::Resolver;
use std::{
    fs::OpenOptions,
    io::Read,
    os::unix::{fs::MetadataExt, fs::OpenOptionsExt},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use tokio_postgres::{config::Host, Config, NoTls};

const EXPECTED_VERIFIER_USER: &str = "ipat_domain_verifier_login";
const MAX_CONNFILE_BYTES: u64 = 4096;
const DEFAULT_BATCH_SIZE: i64 = 25;
const MIN_INTERVAL_SECONDS: u64 = 15;
const MAX_INTERVAL_SECONDS: u64 = 3600;

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnershipTarget {
    fqdn: String,
    verification_name: String,
    verification_value: String,
}

pub(crate) struct VerifierStore {
    db: Config,
    interval: Duration,
}

fn read_private_file(path: &Path) -> Result<String, &'static str> {
    let parent = path.parent().ok_or("missing verifier config directory")?;
    if !path.is_absolute() {
        return Err("verifier DB config path must be absolute");
    }
    for (point, mode) in [(parent, 0o700), (path, 0o600)] {
        let metadata =
            std::fs::symlink_metadata(point).map_err(|_| "missing verifier DB config")?;
        if metadata.file_type().is_symlink()
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.mode() & 0o777 != mode
        {
            return Err("unsafe verifier DB config ownership/mode");
        }
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| "invalid verifier DB config")?;
    let metadata = file
        .metadata()
        .map_err(|_| "invalid verifier DB config metadata")?;
    if !metadata.is_file()
        || metadata.nlink() != 1
        || metadata.len() == 0
        || metadata.len() > MAX_CONNFILE_BYTES
    {
        return Err("unsafe verifier DB config file");
    }
    let mut value = String::new();
    file.take(MAX_CONNFILE_BYTES + 1)
        .read_to_string(&mut value)
        .map_err(|_| "invalid verifier DB config encoding")?;
    if value.len() as u64 > MAX_CONNFILE_BYTES {
        return Err("oversized verifier DB config");
    }
    Ok(value)
}

fn valid_db_config(config: &Config) -> bool {
    config.get_user() == Some(EXPECTED_VERIFIER_USER)
        && config.get_dbname().is_some()
        && config.get_hosts().len() == 1
        && config.get_hostaddrs().is_empty()
        && config.get_options().is_none()
        && matches!(config.get_hosts()[0], Host::Unix(ref path) if path.is_absolute())
}

pub(crate) fn from_environment() -> Result<Arc<VerifierStore>, &'static str> {
    if std::env::var("IPAT_TENANT_DOMAIN_VERIFIER").as_deref() != Ok("YES")
        || unsafe { libc::geteuid() } == 0
    {
        return Err("domain verifier requires explicit nonroot opt-in");
    }
    let path = PathBuf::from(
        std::env::var("IPAT_TENANT_DOMAIN_VERIFIER_DB_CONNINFO_FILE")
            .map_err(|_| "missing verifier DB conninfo file")?,
    );
    let value = read_private_file(&path)?;
    let db = Config::from_str(value.trim()).map_err(|_| "invalid verifier DB config")?;
    if !valid_db_config(&db) {
        return Err("verifier must use dedicated local Unix-socket identity");
    }

    let interval_seconds = std::env::var("IPAT_TENANT_DOMAIN_VERIFIER_INTERVAL_SECONDS")
        .ok()
        .map(|value| value.parse::<u64>())
        .transpose()
        .map_err(|_| "invalid verifier interval")?
        .unwrap_or(60);
    if !(MIN_INTERVAL_SECONDS..=MAX_INTERVAL_SECONDS).contains(&interval_seconds) {
        return Err("verifier interval outside allowed range");
    }

    Ok(Arc::new(VerifierStore {
        db,
        interval: Duration::from_secs(interval_seconds),
    }))
}

async fn pending_targets(client: &tokio_postgres::Client) -> Result<Vec<OwnershipTarget>, ()> {
    let rows = client
        .query(
            "SELECT fqdn,verification_name,verification_value
             FROM ipat_platform.list_tenant_domain_ownership_checks($1)",
            &[&DEFAULT_BATCH_SIZE],
        )
        .await
        .map_err(|_| ())?;
    if rows.len() > DEFAULT_BATCH_SIZE as usize {
        return Err(());
    }
    let mut targets = Vec::with_capacity(rows.len());
    for row in rows {
        let fqdn: String = row.get(0);
        let verification_name: String = row.get(1);
        let verification_value: String = row.get(2);
        if crate::tenant_domain::canonical_requested_domain(&fqdn).as_deref() != Some(&fqdn)
            || verification_name != format!("_ipat-verify.{fqdn}")
            || !verification_value.starts_with("ipat-domain=")
            || verification_value.len() > 160
        {
            return Err(());
        }
        targets.push(OwnershipTarget {
            fqdn,
            verification_name,
            verification_value,
        });
    }
    Ok(targets)
}

fn txt_record_matches(
    txt: &hickory_resolver::proto::rr::rdata::TXT,
    expected: &str,
) -> bool {
    let mut joined = Vec::new();
    for part in txt.txt_data() {
        joined.extend_from_slice(part);
    }
    joined == expected.as_bytes()
}

async fn txt_matches(resolver: &Resolver, target: &OwnershipTarget) -> Result<bool, ()> {
    let query = format!("{}.", target.verification_name);
    let lookup = resolver.txt_lookup(query).await.map_err(|_| ())?;
    Ok(lookup
        .iter()
        .any(|record| txt_record_matches(record, &target.verification_value)))
}

async fn advance_ownership(client: &tokio_postgres::Client, fqdn: &str) -> bool {
    client
        .query_opt(
            "SELECT ipat_platform.advance_tenant_domain_lifecycle(
               $1,'ownership_verified','R955.DNS.TXT.MATCH'
             )",
            &[&fqdn],
        )
        .await
        .ok()
        .flatten()
        .is_some_and(|row| row.get::<_, bool>(0))
}

async fn record_error(client: &tokio_postgres::Client, fqdn: &str, code: &str) {
    let _ = client
        .query_opt(
            "SELECT ipat_platform.record_tenant_domain_lifecycle_error($1,$2)",
            &[&fqdn, &code],
        )
        .await;
}

pub(crate) async fn run_once(store: &VerifierStore) -> Result<usize, &'static str> {
    let (client, connection) = store
        .db
        .connect(NoTls)
        .await
        .map_err(|_| "verifier database unavailable")?;
    let connection_task = tokio::spawn(async move {
        let _ = connection.await;
    });

    let targets = pending_targets(&client)
        .await
        .map_err(|_| "invalid verifier target set")?;
    let resolver = Resolver::builder_tokio()
        .map_err(|_| "resolver system configuration unavailable")?
        .build();

    let mut advanced = 0usize;
    for target in targets {
        match txt_matches(&resolver, &target).await {
            Ok(true) => {
                if advance_ownership(&client, &target.fqdn).await {
                    advanced += 1;
                }
            }
            Ok(false) => {
                record_error(&client, &target.fqdn, "DNS.TXT.MISMATCH").await;
            }
            Err(_) => {
                record_error(&client, &target.fqdn, "DNS.TXT.LOOKUP_FAILED").await;
            }
        }
    }
    drop(client);
    connection_task.abort();
    Ok(advanced)
}

pub(crate) async fn run_loop(store: Arc<VerifierStore>) {
    loop {
        if run_once(&store).await.is_err() {
            eprintln!("IPAT_DOMAIN_VERIFIER_RUN_FAILED");
        }
        tokio::time::sleep(store.interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hickory_resolver::proto::rr::rdata::TXT;

    #[test]
    fn r955_txt_chunks_are_joined_before_exact_comparison() {
        let txt = TXT::new(vec![
            b"ipat-domain=95555555-2222-4333-".to_vec(),
            b"8444-555555555555".to_vec(),
        ]);
        assert!(txt_record_matches(
            &txt,
            "ipat-domain=95555555-2222-4333-8444-555555555555"
        ));
        assert!(!txt_record_matches(&txt, "ipat-domain=other"));
    }

    #[test]
    fn r955_interval_is_operationally_bounded() {
        assert!(MIN_INTERVAL_SECONDS >= 15);
        assert!(MAX_INTERVAL_SECONDS <= 3600);
        assert!(MIN_INTERVAL_SECONDS < MAX_INTERVAL_SECONDS);
    }

    #[test]
    fn r955_expected_runtime_role_is_dedicated() {
        assert_eq!(EXPECTED_VERIFIER_USER, "ipat_domain_verifier_login");
    }
}
