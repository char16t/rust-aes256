use rand::Rng;
use rand::prelude::ThreadRng;
use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use base64::{Engine as _, engine::general_purpose};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use anyhow::Result;

const SALT_SIZE: usize = 8;
const INITIALIZATION_VECTOR_SIZE: usize = 16;  
const KEY_LENGTH: usize = 32; // 32 bytes = 256 bit
const ITERATIONS: u32 = 4096;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

pub struct AES256<'a> {
    password: &'a [u8],
}
impl<'a> AES256<'a> {
    pub fn new(password: &'a [u8]) -> Self {
        Self { password }
    }
    pub fn encrypt(&self, text: &str) -> Result<String> {
        let salt: [u8; SALT_SIZE] = self.generate_random_salt();
        let iv: [u8; INITIALIZATION_VECTOR_SIZE] = self.generate_random_initialization_vector();        
        let key: [u8; KEY_LENGTH] = self.generate_key(&salt);

        let cipher: Aes256Cbc = Aes256Cbc::new_from_slices(&key, &iv)?;
        let encrypted: Vec<u8> = cipher.encrypt_vec(text.as_bytes());

        let mut result: Vec<u8> = Vec::new();
        result.extend(salt);
        result.extend(iv);
        result.extend(&encrypted);

        let b64: String = general_purpose::STANDARD.encode(&result);
        Ok(b64)
    }
    pub fn decrypt(&self, encrypted_text: &str) -> Result<String> {
        let encrypted: &[u8] = &general_purpose::STANDARD.decode(encrypted_text)?[..];
        let salt: &[u8] = &encrypted[0..SALT_SIZE];
        let iv: &[u8] = &encrypted[SALT_SIZE..SALT_SIZE+INITIALIZATION_VECTOR_SIZE];
        let content: &[u8] = &encrypted[SALT_SIZE+INITIALIZATION_VECTOR_SIZE..];

        let key: [u8; KEY_LENGTH] = self.generate_key(salt);
        let cipher: Aes256Cbc = Aes256Cbc::new_from_slices(&key, iv)?;
        let decrypted: Vec<u8> = cipher.decrypt_vec(content)?;
        Ok(String::from_utf8(decrypted)?)
    }
    fn generate_random_salt(&self) -> [u8; SALT_SIZE] {
        let mut rng: ThreadRng = rand::thread_rng();
        let mut salt: [u8; SALT_SIZE] = [0u8; SALT_SIZE];
        rng.fill(&mut salt);
        salt
    }
    fn generate_random_initialization_vector(&self) -> [u8; INITIALIZATION_VECTOR_SIZE] {
        let mut rng: ThreadRng = rand::thread_rng();
        let mut iv: [u8; INITIALIZATION_VECTOR_SIZE] = [0u8; INITIALIZATION_VECTOR_SIZE];
        rng.fill(&mut iv);
        iv
    }
    fn generate_key(&self, salt: &[u8]) -> [u8; KEY_LENGTH] {
        let mut key: [u8; KEY_LENGTH] = [0u8; KEY_LENGTH];
        pbkdf2_hmac::<Sha256>(self.password, salt, ITERATIONS, &mut key);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success() {
        let password: &str = "secret";
        let original: &str = "Original string";

        let aes256: AES256 = AES256::new(password.as_bytes());
        let encrypted: String = aes256.encrypt(&original).expect("Unable to encrypt original message");
        let decrypted: String = aes256.decrypt(&encrypted).expect("Unable to decrypt encrypted message");

        assert_eq!(original, decrypted);
    }

    #[test]
    #[should_panic(expected="Unable to decrypt encrypted message")]
    fn test_failure() {
        let password: &str = "secret";
        let original: &str = "Original string";

        let aes256: AES256 = AES256::new(password.as_bytes());
        let encrypted: String = aes256.encrypt(&original).expect("Unable to encrypt original message");
        let broken: String = encrypted.to_lowercase();
        let _: String = aes256.decrypt(&broken).expect("Unable to decrypt encrypted message");
    }
}