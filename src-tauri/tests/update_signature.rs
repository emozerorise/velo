use base64::{engine::general_purpose::STANDARD, Engine};
use minisign_verify::{PublicKey, Signature};

fn decode(encoded: &str) -> String {
    String::from_utf8(STANDARD.decode(encoded.trim()).expect("base64"))
        .expect("UTF-8 minisign document")
}

#[test]
fn updater_public_key_is_valid() {
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).expect("Tauri config");
    let encoded = config["plugins"]["updater"]["pubkey"]
        .as_str()
        .expect("compiled-in updater key");
    PublicKey::decode(&decode(encoded)).expect("valid updater public key");
}

// Release CI supplies the final archive, after libmpv repair and signing.
// Ordinary development tests need neither a release bundle nor a private key.
#[test]
fn final_archive_signature_matches_compiled_key_and_rejects_tampering() {
    let Ok(archive) = std::env::var("VELO_UPDATE_ARCHIVE") else {
        eprintln!("skipped: set VELO_UPDATE_ARCHIVE to verify a signed release archive");
        return;
    };
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).expect("Tauri config");
    let key = PublicKey::decode(&decode(
        config["plugins"]["updater"]["pubkey"].as_str().unwrap(),
    ))
    .expect("public key");
    let signature = Signature::decode(&decode(
        &std::fs::read_to_string(format!("{archive}.sig")).expect("archive signature"),
    ))
    .expect("signature");
    let mut bytes = std::fs::read(&archive).expect("archive bytes");
    assert!(!bytes.is_empty(), "archive must not be empty");
    key.verify(&bytes, &signature, true)
        .expect("final archive signature must match the key in the app");
    bytes[0] ^= 1;
    assert!(
        key.verify(&bytes, &signature, true).is_err(),
        "changed archive bytes must be rejected"
    );
}
