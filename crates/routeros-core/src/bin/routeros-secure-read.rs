//! Strict owner-approved RouterOS REST HTTPS GET, no arbitrary CLI arguments.
use routeros_core::secure_read::{read_routeros_identity, ApprovedTarget, LocalTrust};
use std::{
    env,
    fs::{self, File},
    io::Read,
    net::Ipv4Addr,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};
fn private_file(path: &Path, limit: u64) -> Result<Vec<u8>, ()> {
    if !path.is_absolute() {
        return Err(());
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| ())?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.permissions().mode() & 0o777 != 0o600
        || metadata.len() > limit
    {
        return Err(());
    }
    let parent = path.parent().ok_or(())?;
    let pm = fs::metadata(parent).map_err(|_| ())?;
    if !pm.is_dir() || pm.permissions().mode() & 0o077 != 0 {
        return Err(());
    }
    let mut file = File::open(path).map_err(|_| ())?;
    let now = file.metadata().map_err(|_| ())?;
    if metadata.dev() != now.dev() || metadata.ino() != now.ino() {
        return Err(());
    }
    let mut data = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut data)
        .map_err(|_| ())?;
    if data.is_empty() || data.len() as u64 > limit {
        return Err(());
    }
    Ok(data)
}
fn hex_sha(value: &str) -> Result<[u8; 32], ()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        return Err(());
    }
    let mut out = [0u8; 32];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&value[i * 2..i * 2 + 2], 16).map_err(|_| ())?;
    }
    Ok(out)
}
fn cred(data: &[u8]) -> Result<(&str, &str), ()> {
    let text = std::str::from_utf8(data).map_err(|_| ())?;
    let data = text.strip_suffix('\n').unwrap_or(text);
    let (u, p) = data.split_once('\n').ok_or(())?;
    if p.contains('\n') || u.is_empty() || p.is_empty() {
        return Err(());
    }
    Ok((u, p))
}
#[tokio::main]
async fn main() {
    if unsafe { libc::geteuid() } == 0
        || std::env::args_os().len() != 1
        || env::var("IPAT_R1012_READ_ONLY_APPROVED").as_deref() != Ok("YES")
    {
        eprintln!("ROUTEROS_READ_DENIED");
        std::process::exit(4);
    }
    let outcome=async {
        let ip=env::var("IPAT_R1012_PRIVATE_TARGET").map_err(|_|())?
            .parse::<Ipv4Addr>().map_err(|_|())?;
        let target=ApprovedTarget::private_ipv4(ip,443).map_err(|_|())?;
        let cred_path=PathBuf::from(env::var("IPAT_R1012_CREDENTIAL_FILE").map_err(|_|())?);
        let ca_path=PathBuf::from(env::var("IPAT_R1012_APPROVED_CA_DER_FILE").map_err(|_|())?);
        let cred_bytes=private_file(&cred_path,350)?;
        let ca_der=private_file(&ca_path,8192)?;
        let (user,password)=cred(&cred_bytes)?;
        let pin=hex_sha(&env::var("IPAT_R1012_APPROVED_LEAF_SHA256").map_err(|_|())?)?;
        let proof=read_routeros_identity(target,LocalTrust{
            ca_der:&ca_der,expected_leaf_sha256:pin,username:user,password,
            approved_read_only:true,
        }).await.map_err(|_|())?;
        println!("ROUTEROS_READ_ONLY_UNREVIEWED target={} board={} architecture={} routeros={} physical_interop_qualified=false write_allowed=false",
            proof.target_id(),proof.board(),proof.architecture(),proof.observed_routeros());
        Ok::<(),()>(())
    }.await;
    if outcome.is_err() {
        eprintln!("ROUTEROS_READ_DENIED_OR_UNQUALIFIED");
        std::process::exit(4);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn r1012_nonsecret_config_parser_rejects_duplicates_and_invalid() {
        assert!(hex_sha(&"a".repeat(64)).is_ok());
        for bad in ["a".repeat(63), "Z".repeat(64), "G".repeat(64)] {
            assert!(hex_sha(&bad).is_err());
        }
        assert_eq!(
            cred(b"reader\nsynthetic-pass\n"),
            Ok(("reader", "synthetic-pass"))
        );
        assert!(cred(b"reader\npass\nanother\n").is_err());
        assert!(cred(b"reader-only").is_err());
        assert!(cred(b"reader\n").is_err());
    }
}
