use rust_aes256::AES256;

fn main() {
    let password: &str = "secret";
    let original: &str = "Original string";

    let aes256: AES256 = AES256::new(password.as_bytes());
    let encrypted: String = aes256.encrypt(&original).expect("Unable to encrypt original message");
    println!("Encrypted: {}", encrypted);
    
    let decrypted: String = aes256.decrypt(&encrypted).expect("Unable to decrypt encrypted message");
    println!("Decrypted: {}", decrypted);
}