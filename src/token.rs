use crate::structs::Token;
use rand::RngExt;
use rand::distr::Alphanumeric;
use time::OffsetDateTime;

pub fn generate_token(expires: Option<OffsetDateTime>) -> Token {
    let value = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>();

    Token { value, expires }
}

pub fn validate_token(token: &str, tokens: &[Token]) -> bool {
    tokens
        .iter()
        .any(|t| t.value == token && t.expires.map_or(true, |e| e > OffsetDateTime::now_utc()))
}
