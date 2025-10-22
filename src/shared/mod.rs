use hex::FromHexError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize)]
pub struct SetupRequest {
    id: String,
    phi0: String,
    c: String,
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("invalid hex encoding: {0}")]
    InvalidHex(#[from] FromHexError),

    #[error("invalid byte length: {0}")]
    InvalidLength(String),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginRequest {
    pub id: String,     // username
    pub u: String,      // hex(CompressedRistretto)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginResponse {
    pub v: String,      // hex(CompressedRistretto)
    pub id_s: String,   // server identifier (plain)
}


impl SetupRequest {
    pub fn new(id: String, phi0: &[u8; 32], c: &[u8; 32]) -> Self {
        Self {
            id,
            phi0: hex::encode(phi0),
            c: hex::encode(c),
        }
    }

    pub fn decode(self) -> Result<(String, [u8; 32], [u8; 32]), DecodeError> {
        let phi0 = hex::decode(&self.phi0)?;
        let c = hex::decode(&self.c)?;
        match (phi0.try_into(), c.try_into()) {
            (Ok(phi0), Ok(c)) => Ok((self.id, phi0, c)),
            (Err(_), _) => Err(DecodeError::InvalidLength("phi0".into())),
            (_, Err(_)) => Err(DecodeError::InvalidLength("c".into())),
        }
    }
}
