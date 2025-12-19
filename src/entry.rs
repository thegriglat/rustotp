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

    pub fn dump(&self) -> String {
        format!("{}={}", self.name, self.totp.get_secret_base32())
    }

    pub fn parse(s: &str) -> Self {
        let parts: Vec<&str> = s.split("=").collect();
        if parts.len() != 2 {
            panic!("Invalid entry format");
        }
        TOTPEntry::new(parts[0], parts[1])
    }
}
