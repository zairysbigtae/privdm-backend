use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claim {
    pub iat: usize,
    pub eat: usize,
    pub name: String,
}

pub fn generate_token<S: AsRef<str>>(name: S, secret: S) -> String {
    let now = Utc::now();
    let expire = Duration::weeks(26); // 26 weeks for 6 months

    let claim = Claim {
        iat: now.timestamp() as usize,
        eat: (now + expire).timestamp() as usize,
        name: name.as_ref().to_owned(),
    };

    encode(&Header::default(), &claim, &EncodingKey::from_secret(secret.as_ref().as_bytes()))
        .expect("Token generation failed")
}

#[cfg(test)]
mod tests {
    use super::generate_token;

    #[test]
    fn generate_token_test() {
        let token = generate_token("Zai", "zairysbigtae222");
        println!("{token}");
    }
}
