use minisign::{sign, verify, PublicKey, PublicKeyBox, SecretKey, SecretKeyBox};
use std::{
    env,
    fs::{self, File},
    path::PathBuf,
};

fn sign_assets(
    secret_key: &SecretKey,
    public_key: &PublicKey,
    paths: &[PathBuf],
) -> Result<(), Box<dyn std::error::Error>> {
    if paths.is_empty() || paths.len() % 2 != 0 {
        return Err("Supply one or more asset and signature path pairs.".into());
    }
    for pair in paths.chunks_exact(2) {
        let signature = sign(
            None,
            secret_key,
            File::open(&pair[0])?,
            None,
            Some("Wavelength desktop update"),
        )?;
        verify(
            public_key,
            &signature,
            File::open(&pair[0])?,
            true,
            false,
            false,
        )?;
        fs::write(&pair[1], signature.to_string())?;
    }
    Ok(())
}

fn validate_secret_key_contents(contents: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut lines = contents.lines().filter(|line| !line.trim().is_empty());
    if lines.next().is_none() || lines.next().is_none() {
        return Err(
            "UPDATE_SIGNING_PRIVATE_KEY must contain the complete Minisign key file, including its comment and encoded-key lines."
                .into(),
        );
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = env::args_os().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if paths.is_empty() || paths.len() % 2 != 0 {
        return Err("Supply one or more asset and signature path pairs.".into());
    }
    let key = env::var("UPDATE_SIGNING_PRIVATE_KEY")?;
    validate_secret_key_contents(&key)?;
    let public_key = env::var("UPDATE_SIGNING_PUBLIC_KEY")?;
    let password = env::var("UPDATE_SIGNING_PRIVATE_KEY_PASSWORD")
        .ok()
        .filter(|password| !password.is_empty());
    let secret_key = if let Some(password) = password {
        SecretKey::from_box(SecretKeyBox::from(key), Some(password))?
    } else {
        SecretKey::from_unencrypted_box(SecretKeyBox::from(key))?
    };
    let public_key = PublicKey::from_box(PublicKeyBox::from(public_key))?;
    sign_assets(&secret_key, &public_key, &paths)
}

#[cfg(test)]
mod tests {
    use super::{sign_assets, validate_secret_key_contents};
    use minisign::{KeyPair, PublicKey, SecretKey, SignatureBox};
    use std::{
        fs::{self, File},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn rejects_a_private_key_secret_without_its_encoded_key_line() {
        assert!(validate_secret_key_contents("untrusted comment: minisign key\n").is_err());
        assert!(validate_secret_key_contents(
            "untrusted comment: minisign key\nencoded-key-line\n"
        )
        .is_ok());
    }

    #[test]
    fn signs_each_asset_with_the_configured_key() {
        let keypair = KeyPair::generate_unencrypted_keypair().unwrap();
        let secret_key = SecretKey::from_unencrypted_box(keypair.sk.to_box(None).unwrap()).unwrap();
        let base = std::env::temp_dir().join(format!(
            "wavelength-update-signer-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let asset = base.with_extension("exe");
        let signature = PathBuf::from(format!("{}.minisig", asset.display()));
        fs::write(&asset, b"test installer").unwrap();

        let unrelated_key = KeyPair::generate_unencrypted_keypair().unwrap();
        let unrelated_public_key =
            PublicKey::from_box(unrelated_key.pk.to_box().unwrap()).unwrap();
        assert!(sign_assets(
            &secret_key,
            &unrelated_public_key,
            &[asset.clone(), signature.clone()]
        )
        .is_err());
        assert!(!signature.exists());

        let public_key = PublicKey::from_box(keypair.pk.to_box().unwrap()).unwrap();
        sign_assets(
            &secret_key,
            &public_key,
            &[asset.clone(), signature.clone()],
        )
        .unwrap();
        let signature_box = SignatureBox::from_file(&signature).unwrap();
        minisign::verify(
            &keypair.pk,
            &signature_box,
            File::open(&asset).unwrap(),
            true,
            false,
            false,
        )
        .unwrap();

        fs::remove_file(asset).unwrap();
        fs::remove_file(signature).unwrap();
    }
}
