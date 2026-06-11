use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub fn hash_password(password: &str) -> String {
  Argon2::default().hash_password(password.as_bytes()).unwrap().to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
  let parsed = PasswordHash::new(hash).unwrap();
  Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok()
}
