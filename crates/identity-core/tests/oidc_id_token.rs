//! Independent synthetic RSA JWT fixtures: NEVER a real human IdP session.
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use identity_core::PinnedIssuer;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
const ISS: &str = "https://id.example.invalid/realms/ipat";
const API: &str = "ipat-control-api";
const CLIENT: &str = "ipat-private-browser";
const KID: &str = "pinned-test-key";
const NONCE: &str = "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn";
struct Fixture {
    _tmp: tempfile::TempDir,
    private: Vec<u8>,
    pinned: PinnedIssuer,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let privf = dir.path().join("synthetic-private.pem");
        let pubf = dir.path().join("synthetic-public.pem");
        assert!(Command::new("openssl")
            .args([
                "genpkey",
                "-algorithm",
                "RSA",
                "-pkeyopt",
                "rsa_keygen_bits:2048",
                "-out"
            ])
            .arg(&privf)
            .output()
            .unwrap()
            .status
            .success());
        assert!(Command::new("openssl")
            .args(["pkey", "-pubout", "-in"])
            .arg(&privf)
            .arg("-out")
            .arg(&pubf)
            .output()
            .unwrap()
            .status
            .success());
        let public = fs::read(&pubf).unwrap();
        Self {
            _tmp: dir,
            private: fs::read(&privf).unwrap(),
            pinned: PinnedIssuer::new(ISS, API, KID, &public).unwrap(),
        }
    }
    fn sign(&self, value: &Value, kid: &str, typ: &str) -> String {
        let mut h = Header::new(Algorithm::RS256);
        h.kid = Some(kid.into());
        h.typ = Some(typ.into());
        encode(
            &h,
            value,
            &EncodingKey::from_rsa_pem(&self.private).unwrap(),
        )
        .unwrap()
    }
    fn access(&self) -> String {
        self.sign(&access_claims(), KID, "at+jwt")
    }
    fn id(&self, access: &str) -> Value {
        let now = clock();
        let hash = Sha256::digest(access.as_bytes());
        json!({"iss":ISS,"aud":CLIENT,"sub":"human-synthetic-only",
            "iat":now,"nbf":now,"exp":now+180,"auth_time":now,
            "nonce":NONCE,"at_hash":URL_SAFE_NO_PAD.encode(&hash[..16]),
            "amr":["pwd","mfa"],"azp":CLIENT})
    }
    fn verify(&self, c: &Value, access: &str) -> bool {
        self.pinned
            .verify_offline_browser_pair(CLIENT, NONCE, &self.sign(c, KID, "JWT"), access)
            .is_ok()
    }
}
fn clock() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
fn access_claims() -> Value {
    let n = clock();
    json!({"iss":ISS,"aud":API,"sub":"human-synthetic-only",
        "iat":n,"nbf":n,"exp":n+180,"amr":["pwd","mfa"],
        "tenant_id":"fake-escalation","realm_access":{"roles":["super_admin"]}})
}
#[test]
fn genuinely_signed_matching_pair_accepts_only_identity_not_tenant() {
    let fixture = Fixture::new();
    let access = fixture.access();
    let id = fixture.id(&access);
    let verified = fixture
        .pinned
        .verify_offline_browser_pair(CLIENT, NONCE, &fixture.sign(&id, KID, "JWT"), &access)
        .unwrap();
    assert_eq!(verified.subject(), "human-synthetic-only");
    assert_eq!(verified.issuer(), ISS);
    assert!(verified.expires_at() > clock());
    // The returned identity has no tenant, POP, role, approver or session field.
}
#[test]
fn nonce_access_binding_and_wrong_client_fail() {
    let fixture = Fixture::new();
    let access = fixture.access();
    let mut id = fixture.id(&access);
    assert!(!fixture
        .pinned
        .verify_offline_browser_pair(
            CLIENT,
            &"q".repeat(43),
            &fixture.sign(&id, KID, "JWT"),
            &access
        )
        .is_ok());
    id["nonce"] = json!("q".repeat(43));
    assert!(!fixture.verify(&id, &access));
    id = fixture.id(&access);
    id["at_hash"] = json!("AAAAAAAAAAAAAAAAAAAAAA");
    assert!(!fixture.verify(&id, &access));
    id = fixture.id(&access);
    id["aud"] = json!(API);
    assert!(!fixture.verify(&id, &access));
    id = fixture.id(&access);
    id["azp"] = json!("another-client");
    assert!(!fixture.verify(&id, &access));
}
#[test]
fn missing_or_bogus_mfa_and_stale_auth_time_fail() {
    let fixture = Fixture::new();
    let access = fixture.access();
    for amr in [
        json!(["pwd"]),
        json!([]),
        json!(["not_mfa"]),
        json!(["mfa".repeat(33)]),
    ] {
        let mut id = fixture.id(&access);
        id["amr"] = amr;
        assert!(!fixture.verify(&id, &access));
    }
    let mut stale = fixture.id(&access);
    stale["auth_time"] = json!(clock() - 1000);
    assert!(!fixture.verify(&stale, &access));
    let mut future = fixture.id(&access);
    future["auth_time"] = json!(clock() + 1000);
    assert!(!fixture.verify(&future, &access));
    let mut no_amr = fixture.id(&access);
    no_amr.as_object_mut().unwrap().remove("amr");
    assert!(!fixture.verify(&no_amr, &access));
}
#[test]
fn signature_header_subject_and_access_token_confusion_fail() {
    let fixture = Fixture::new();
    let other = Fixture::new();
    let access = fixture.access();
    let id = fixture.id(&access);
    let other_token = other.sign(&id, KID, "JWT");
    assert!(fixture
        .pinned
        .verify_offline_browser_pair(CLIENT, NONCE, &other_token, &access)
        .is_err());
    assert!(fixture
        .pinned
        .verify_offline_browser_pair(
            CLIENT,
            NONCE,
            &fixture.sign(&id, "wrong-kid", "JWT"),
            &access
        )
        .is_err());
    assert!(fixture
        .pinned
        .verify_offline_browser_pair(CLIENT, NONCE, &fixture.sign(&id, KID, "at+jwt"), &access)
        .is_err());
    let mut wrong = id;
    wrong["sub"] = json!("other-subject");
    assert!(!fixture.verify(&wrong, &access));
    let mut only_password = access_claims();
    only_password["amr"] = json!(["pwd"]);
    let weak_access = fixture.sign(&only_password, KID, "at+jwt");
    assert!(fixture
        .pinned
        .verify_offline_browser_pair(
            CLIENT,
            NONCE,
            &fixture.sign(&fixture.id(&weak_access), KID, "JWT"),
            &weak_access
        )
        .is_err());
}
#[test]
fn malformed_and_future_id_tokens_rejected() {
    let fixture = Fixture::new();
    let access = fixture.access();
    let mut id = fixture.id(&access);
    id["iat"] = json!(clock() + 3600);
    id["nbf"] = json!(clock() + 3600);
    id["exp"] = json!(clock() + 4000);
    assert!(!fixture.verify(&id, &access));
    let mut long = fixture.id(&access);
    long["exp"] = json!(clock() + 3600);
    assert!(!fixture.verify(&long, &access));
    let id = fixture.id(&access);
    assert!(fixture
        .pinned
        .verify_offline_browser_pair(CLIENT, NONCE, "not-a-jwt", &access)
        .is_err());
    assert!(fixture
        .pinned
        .verify_offline_browser_pair(
            CLIENT,
            NONCE,
            &fixture.sign(&id, KID, "JWT"),
            "bogus-access"
        )
        .is_err());
}
