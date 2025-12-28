use std::time::{SystemTime, UNIX_EPOCH};

use totp_rs::{Algorithm, Secret, TOTP};

pub struct TOTPEntry {
    pub name: String,
    totp: TOTP,
}

impl TOTPEntry {
    pub fn new(name: &str, secret: &str) -> Self {
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.to_string()).to_bytes().unwrap(),
        )
        .expect("Cannot create TOTP instance");

        TOTPEntry {
            name: name.to_string(),
            totp,
        }
    }

    pub fn current_code(&self) -> String {
        self.totp.generate_current().unwrap()
    }

    pub fn remaining_seconds(&self) -> u16 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let next_step = self.totp.next_step_current().unwrap();

        // 4. Считаем остаток
        (next_step - now) as u16
    }

    pub fn parse(s: &str) -> Self {
        let parts: Vec<&str> = s.split("=").collect();
        if parts.len() != 2 {
            panic!("Invalid entry format");
        }
        TOTPEntry::new(parts[0], parts[1])
    }
}
