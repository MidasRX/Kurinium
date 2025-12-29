pub fn decrypt_token(encrypted: &[u8], key: u8) -> String {
    let decrypted: Vec<u8> = encrypted
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key.wrapping_add(i as u8))
        .collect();

    String::from_utf8_lossy(&decrypted).to_string()
}

#[macro_export]
macro_rules! decrypt_token {
    ($encrypted:expr, $key:expr) => {
        $crate::utils::token::decrypt_token($encrypted, $key)
    };
}
