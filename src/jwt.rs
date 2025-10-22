use chrono::{Duration, TimeDelta, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claim {
    pub iat: usize,
    pub exp: usize,
    pub name: String,
}

pub fn generate_token<S: AsRef<str>>(name: S, secret: S, exp: TimeDelta) -> String {
    let now = Utc::now();

    let claim = Claim {
        iat: now.timestamp() as usize,
        exp: (now + exp).timestamp() as usize,
        name: name.as_ref().to_owned(),
    };

    encode(&Header::default(), &claim, &EncodingKey::from_secret(secret.as_ref().as_bytes()))
        .expect("Token generation failed")
}
