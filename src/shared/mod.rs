use curve25519_dalek::{RistrettoPoint, Scalar, ristretto::CompressedRistretto};
use hex::FromHexError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("invalid hex encoding: {0}")]
    InvalidHex(#[from] FromHexError),

    #[error("invalid byte length: {0}")]
    InvalidLength(String),

    #[error("invalid Ristretto point")]
    InvalidPoint,
}

#[derive(Serialize, Deserialize)]
pub struct SetupRequestEncoded {
    pub id: String,
    pub phi0: String,
    pub c: String,
}

pub struct SetupRequest {
    pub id: String,
    pub phi0: Scalar,
    pub c: RistrettoPoint,
}

impl SetupRequestEncoded {
    pub fn decode(self) -> Result<SetupRequest, DecodeError> {
        let phi0_bytes = hex::decode(&self.phi0)?;
        let c_bytes = hex::decode(&self.c)?;

        let phi0_bytes: [u8; 32] = phi0_bytes
            .try_into()
            .map_err(|_| DecodeError::InvalidLength("phi0".into()))?;

        let phi0 = Scalar::from_bytes_mod_order(phi0_bytes);
        let c = match CompressedRistretto::from_slice(&c_bytes) {
            Ok(compressed) => match compressed.decompress() {
                Some(c) => c,
                None => return Err(DecodeError::InvalidPoint),
            },
            _ => return Err(DecodeError::InvalidLength("c".into())),
        };

        Ok(SetupRequest {
            id: self.id,
            phi0,
            c,
        })
    }
}

impl SetupRequest {
    pub fn new(id: String, phi0: Scalar, c: RistrettoPoint) -> Self {
        Self { id, phi0, c }
    }

    pub fn encode(self) -> SetupRequestEncoded {
        SetupRequestEncoded {
            id: self.id,
            phi0: hex::encode(self.phi0.to_bytes()),
            c: hex::encode(self.c.compress().to_bytes()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequestEncoded {
    pub id: String,
    pub u: String,
}

pub struct LoginRequest {
    pub id: String,
    pub u: RistrettoPoint,
}

impl LoginRequestEncoded {
    pub fn decode(self) -> Result<LoginRequest, DecodeError> {
        let u_bytes = hex::decode(&self.u)?;

        let u = match CompressedRistretto::from_slice(&u_bytes) {
            Ok(c) => match c.decompress() {
                Some(u) => u,
                None => return Err(DecodeError::InvalidPoint),
            },
            Err(_) => return Err(DecodeError::InvalidLength("u".into())),
        };

        Ok(LoginRequest { id: self.id, u })
    }
}

impl LoginRequest {
    pub fn new(id: String, u: RistrettoPoint) -> Self {
        Self { id, u }
    }

    pub fn encode(self) -> LoginRequestEncoded {
        LoginRequestEncoded {
            id: self.id,
            u: hex::encode(self.u.compress().to_bytes()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponseEncoded {
    pub v: String,
}

pub struct LoginResponse {
    pub v: RistrettoPoint,
}

impl LoginResponseEncoded {
    pub fn decode(self) -> Result<LoginResponse, DecodeError> {
        let v_bytes = hex::decode(&self.v)?;

        let v = match CompressedRistretto::from_slice(&v_bytes) {
            Ok(c) => match c.decompress() {
                Some(u) => u,
                None => return Err(DecodeError::InvalidPoint),
            },
            Err(_) => return Err(DecodeError::InvalidLength("u".into())),
        };

        Ok(LoginResponse { v })
    }
}

impl LoginResponse {
    pub fn new(v: RistrettoPoint) -> Self {
        Self { v }
    }

    pub fn encode(self) -> LoginResponseEncoded {
        LoginResponseEncoded {
            v: hex::encode(self.v.compress().to_bytes()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct VerifyRequestEncoded {
    pub idc: String,
    pub key: String,
}

pub struct VerifyRequest {
    pub idc: String,
    pub key: [u8; 32],
}

impl VerifyRequestEncoded {
    // Assumes key is a valid hex string representing [u8; 32]
    pub fn new(idc: String, key: String) -> Self {
        Self { idc, key }
    }

    pub fn decode(self) -> Result<VerifyRequest, DecodeError> {
        let key = hex::decode(self.key)?;
        let key = key
            .try_into()
            .map_err(|_| DecodeError::InvalidLength("key".into()))?;
        Ok(VerifyRequest { idc: self.idc, key })
    }
}
