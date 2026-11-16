//! Sign a release's `SHA256SUMS.txt`, or create the update signing key.
//!
//! Standalone on purpose: the release workflow runs this in the publish job,
//! which has Rust but no GTK/WebKit development packages, so it must not pull in
//! the application crate.
//!
//! ```text
//! sign-manifest generate <key-path>
//! UPDATE_SIGNING_KEY=... sign-manifest sign <manifest> <out.sig> --expect <public-key-file>
//! ```

use std::path::Path;

use ed25519_compact::{KeyPair, Noise, Seed};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    let text = text.trim().as_bytes();
    if text.len() % 2 != 0 {
        return None;
    }
    text.chunks(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(pair, 16).ok()
        })
        .collect()
}

fn verify(manifest: &[u8], signature_hex: &str, public_hex: &str) -> Result<(), String> {
    use ed25519_compact::{PublicKey, Signature};

    let key = unhex(public_hex).ok_or("the public key is not hex")?;
    let signature = unhex(signature_hex).ok_or("the signature is not hex")?;
    let key = PublicKey::from_slice(&key).map_err(|_| "the public key is malformed")?;
    let signature = Signature::from_slice(&signature).map_err(|_| "the signature is malformed")?;
    key.verify(manifest, &signature)
        .map_err(|_| "the signature does not verify under that key".to_string())
}

fn public_hex(pair: &KeyPair) -> String {
    hex(pair.pk.as_ref())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command, key_path] if command == "generate" => {
            let seed = Seed::generate();
            let pair = KeyPair::from_seed(seed);
            let path = Path::new(key_path);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            // create_new: an existing key is never overwritten. Losing it ends
            // signing for every installed copy.
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(path).unwrap_or_else(|error| {
                eprintln!("error: {key_path} cannot be created: {error}");
                std::process::exit(1);
            });
            use std::io::Write;
            file.write_all(hex(seed.as_ref()).as_bytes())
                .expect("the key file could not be written");
            // Only the public half goes to stdout, so it can be committed.
            println!("{}", public_hex(&pair));
        }
        [command, input, output, flag, expected_path]
            if command == "sign" && flag == "--expect" =>
        {
            let seed_hex =
                std::env::var("UPDATE_SIGNING_KEY").expect("UPDATE_SIGNING_KEY is not set");
            let seed = Seed::from_slice(&unhex(&seed_hex).expect("the key is not hex"))
                .expect("the key is not a 32-byte Ed25519 seed");
            let pair = KeyPair::from_seed(seed);

            // The secret must be the key the app trusts, or the release would
            // ship an update every installed copy refuses.
            let expected = std::fs::read_to_string(expected_path)
                .expect("the public key file could not be read");
            if public_hex(&pair) != expected.trim().to_lowercase() {
                eprintln!(
                    "error: UPDATE_SIGNING_KEY does not match {expected_path}; \
                     the release would be signed by a key the app does not trust"
                );
                std::process::exit(1);
            }

            let message = std::fs::read(input).expect("the manifest could not be read");
            let signature = pair.sk.sign(&message, Some(Noise::generate()));
            std::fs::write(output, hex(signature.as_ref()))
                .expect("the signature file could not be written");
            println!("signed {input}");
        }
        [command, manifest, signature, flag, expected_path]
            if command == "verify" && flag == "--expect" =>
        {
            let public = std::fs::read_to_string(expected_path)
                .expect("the public key file could not be read");
            let signature = std::fs::read_to_string(signature)
                .expect("the signature file could not be read");
            let message = std::fs::read(manifest).expect("the manifest could not be read");
            match verify(&message, &signature, public.trim()) {
                Ok(()) => println!("signature verifies under {expected_path}"),
                Err(error) => {
                    eprintln!("error: {error}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!(
                "usage: sign-manifest generate <key-path>\n       \
                 sign-manifest sign <manifest> <out.sig> --expect <public-key-file>\n       \
                 sign-manifest verify <manifest> <sig> --expect <public-key-file>"
            );
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_signature_verifies_under_its_key_and_not_another() {
        let pair = KeyPair::from_seed(Seed::generate());
        let public = hex(pair.pk.as_ref());
        let signature = hex(pair.sk.sign(b"manifest", Some(Noise::generate())).as_ref());
        assert!(verify(b"manifest", &signature, &public).is_ok());
        assert!(verify(b"changed", &signature, &public).is_err());
        let other = hex(KeyPair::from_seed(Seed::generate()).pk.as_ref());
        assert!(verify(b"manifest", &signature, &other).is_err());
    }

    #[test]
    fn hex_decoding_rejects_garbage_without_panicking() {
        assert!(unhex("0\u{4e00}").is_none());
        assert!(unhex("zz").is_none());
        assert!(unhex("abc").is_none());
        assert_eq!(unhex("00ff"), Some(vec![0, 255]));
    }
}
