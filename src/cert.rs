use anyhow::Result;
use rcgen::{generate_simple_self_signed, CertifiedKey};

pub fn generate_cert() -> Result<()> {
  let subject_alt_names = vec!["localhost".to_string()];
  let CertifiedKey { cert, signing_key } =
    generate_simple_self_signed(subject_alt_names)?;

  fs_err::create_dir_all("certs")?;
  fs_err::write("certs/cert.pem", cert.pem())?;
  fs_err::write("certs/key.pem", signing_key.serialize_pem())?;

  Ok(())
}