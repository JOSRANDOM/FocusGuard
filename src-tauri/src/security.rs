use rand::RngCore;
use sha2::{Digest, Sha256};

/// Genera una sal aleatoria de 16 bytes, codificada en hex.
pub fn generate_salt() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Hash de la contraseña con sal (SHA-256). No es para credenciales de alto
/// valor: es solo la "fricción" de un candado local de autocontrol.
pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

pub fn verify(password: &str, hash: &str, salt: &str) -> bool {
    hash_password(password, salt) == hash
}
