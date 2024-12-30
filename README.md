Этот документ [переведён на русский язык](README.ru.md).

[![build](https://github.com/char16t/rust-aes256/actions/workflows/build.yml/badge.svg)](https://github.com/char16t/rust-aes256/actions/workflows/build.yml) ![coverage](https://char16t.github.io/rust-aes256/badges/flat.svg)

## Quick start

Two parties can exchange encrypted messages by agreeing to use a password. You can use this library as follows:

```rust
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
```

Over the network you will be able to send data in approximately this format:

```json
{
  "data": "yRN4j3X9ReBocpR6vWrtG96ASY1EXVuptZ/TxrIBvk3zUP3wIMEBOjKnhQfQGVDqn5v5mWjP7x605k82rr5rCg=="
}
```

This library is not published in crates.io. So before you started you should:

1. Clone, run tests and build
```sh
git clone
cargo test
cargo build --release
```

2. Add dependency to `Cargo.toml`
```toml
[dependencies]
rust_aes256 = { path = "path/to/rust_aes256" }
```

## Details

AES is a symmetric block cipher algorithm. It is a method of encrypting data in which the same key is used for both encoding and decrypting information. The AES algorithm is an iterative block cipher with a symmetric key that supports cryptographic keys (secret keys) of 128, 192, and 256 bits to encrypt and decrypt data in 128-bit blocks. If the data to be encrypted does not meet the 128-bit block size requirement, it must be padded. The final block will be padded to 128 bits.

AES variants:

* ECB (Electronic Code Book) – The plaintext is divided into 128-bit blocks. Each block is then encrypted with the same key and algorithm. This means that identical plaintext blocks will result in identical ciphertext blocks.
* CBC (Cipher Block Chaining) – CBC mode solves the problem of identical plaintext blocks in ECB. It introduces an initialization vector (IV) for the first block and XORs the previous ciphertext block with the current plaintext block before encryption. CBC mode provides a higher level of security than ECB and is widely used in secure communication protocols such as SSL/TLS, IPSec, and VPN.

This library uses AES-256 CBC.

Terms:

* **message** is a set of data that we exchange over the network
* **password** is a "secret key" that we exchange once. It is a string of arbitrary length
* **secret key** is a 256-bit (32-byte) AES-256 binary key that is a hash of the password
* **salt** is a random 8 bytes that is added to the end of the password when forming the secret key. It is transmitted openly at the beginning of the message
* **initialization vector** is a random 16 bytes that will be required to implement AES-256-CBC. It is transmitted openly at the beginning of the message
* **encrypted data** is data encrypted with the AES-256-CBC algorithm. One of the parts of the message

## Message

Messages of the following format will be transmitted over the network:

```
                   BASE64
+----------+--------------+-----------------+
|   Salt   | Init. vector |     Payload     |
|  8 byte  |   16 byte    |    Any length   |
+----------+--------------+-----------------+
```

The message is a set of bytes encoded in base64. The message has the following format:

* Salt - the first 8 bytes (salt)
* Initialization vector - the next 16 bytes (iv, Initialization vector)
* AES-256-CBC encrypted data (payload)

## Salt

As stated above, the AES-256 algorithm works with keys of 256 bits, but the password (the secret key in the form of a string that we exchange) can be of any length. In order to make a binary key of a fixed length from a text password of any length, which is needed for the encryption algorithm, a hash function is used.

```
password = "secret key as a string"
aes256key = hash(password)
```

What happens if an attacker somehow gets the key (aes256key)? Although he won't know the password (the hash function is one-way by definition), all files encrypted with this password will be accessible, because they are encrypted with the same key (aes256key), and it doesn't matter from which password this key was obtained. In order to avoid such a situation, a slightly more complex key generation system is used:

```
password = "secret key as a string"
aes256key = hash(password + salt)
```

Here salt is a random string. Salt is not a secret and is usually transmitted openly along with the encrypted message. Let's say we encrypted two files with one password. The keys for them will be generated as follows:

```
password = "secret key as a string"
aes256key1 = hash(password + salt1)
aes256key2 = hash(password + salt2)
```

If an attacker has received aes256key1, he cannot get the password from it (as in the simple scheme). Also, if one key (aes256key1 or aes256key2) is compromised, the password itself (password) and other encrypted files will not be affected in any way.

Using a regular integer written as a string as the salt would give 4 billion variations, and a 64-bit number should be sufficient in any case.

## Initialization vector

A block cipher encrypts exactly one block. In our case (AES-256) 16 bytes. The original stream is padded to a size multiple of the block size. Then the stream is encrypted, block by block. This scheme is called ECB (Electronic Code Book). The problem with it is that the same input data produces the same encrypted blocks. From the analysis of the same blocks, one can draw some conclusions about the contents of the file.

To combat this, a block chaining method is used. In the simplest case, this is simply XOR with the previous block. For simplicity, we can say that chaining methods are based on the fact that the current block is somehow mixed with the previous one. Therefore, the question naturally arises - what to mix the very first block with? With an artificially generated "zero" block, which is called the "initialization vector".

Like the salt, the initialization vector is not a secret. In our case, we will transmit it at the beginning of the encrypted message.

## Encryption process

The general scheme of the encryption process looks like this:

```
+------------+    +--------+
|  password  |    |  salt  |
+------------+    +--------+
          |           |
          ▼           ▼
        +----------------+   +------------------------+
        |   Secret key   |   |                        |
        |     AES-256    |   |  Initialization vector |
        |                |   |                        | 
        |  (secret key)  |   |                        | 
        +----------------+   +------------------------+
                      |         |
                      ▼         ▼
                     +-----------+
     Data      ----- |   Cipher  | --------->   Encrypted
   to encrypt        |           |                 data
                     +-----------+              (payload)
```

In order to encrypt the data, we will need:

* `data` - The actual data to encrypt
* `password` - The password to encrypt (the key that we will exchange once, in the form of a string of arbitrary length)
* `salt` - A randomly generated salt (an array of bytes of size 8)
* `initialization vector` - A randomly generated initialization vector (an array of bytes of size 16)

Encryption algorithm:

1. Generate salt
2. Generate initialization vector
3. Generate AES-256 secret key (binary key of 256 bits) based on password and salt
4. Generate cipher based on secret key and initialization vector
5. Apply cipher to data to be encrypted and get encrypted data (payload)
6. Generate message as byte array

```
+----------+--------------+-----------------+
|   Salt   | Init. vector |     Payload     |
|  8 byte  |   16 byte    |    Any length   |
+----------+--------------+-----------------+
```
7. Convert the message to base64 format.

## Decryption process

The decryption process is similar to the encryption process:

```
+------------+    +--------+
|            |    |        |
| Password   |    | Salt   |
+------------+    +--------+
          |           |
          ▼           ▼
        +----------------+   +------------------------+
        |   Secret key   |   |                        |
        |    AES-256     |   |    Initialization      |
        |                |   |       vector           | 
        |  (secret key)  |   |                        | 
        +----------------+   +------------------------+
                      |         |
                      ▼         ▼
                     +-----------+
  Encrypted    ----- |  Cipher   | --------->   Decrypted
   data              |           |               data
  (payload)          +-----------+              
```

The difference is that there is no need to generate a password and salt anymore. Now we read them from the beginning of the transmitted message. Also, now we use the cipher in the opposite direction -- for decryption.

The full decryption algorithm:

1. Get the message encoded in base64
```
+-------------------------------------------+
|                BASE64                     |
+----------+--------------+-----------------+
|   Salt   | Init. vector |     Payload     |
|  8 byte  |   16 byte    |    Any length   |
+----------+--------------+-----------------+
```
2. Decode the message and get a set of bytes
```
+----------+--------------+-----------------+
|   Salt   | Init. vector |     Payload     |
|  8 byte  |   16 byte    |    Any length   |
+----------+--------------+-----------------+
```
3. Store the salt (Salt), the first 8 bytes of the message, into a byte array
4. Store the initialization vector (Init. vector), the next 16 bytes of the message, into a byte array
5. Store the encrypted data (Payload), the rest of the message, into a byte array
6. Generate a secret AES-256 key (a 256-bit binary key) based on the password and salt
7. Generate a cipher based on the secret key and initialization vector
8. Apply the cipher to the encrypted data (payload) to decrypt and get the data
