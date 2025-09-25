use curve25519_dalek::{RistrettoPoint, Scalar};
use group::Group;
use sha2::{Digest, Sha512};

fn _h_prime(m: &[u8]) -> [u8; 32] {
    let hash = Sha512::digest(m);
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&hash[..32]);
    bytes
}

fn h(m: &[u8]) -> (Scalar, Scalar) {
    let hash = Sha512::digest(m);
    let mut left = [0u8; 32];
    let mut right = [0u8; 32];
    left.copy_from_slice(&hash[..32]);
    right.copy_from_slice(&hash[32..]);
    (
        Scalar::from_bytes_mod_order(left),
        Scalar::from_bytes_mod_order(right),
    )
}

pub fn setup_1(password: &str, idc: &str, ids: &str) -> (Scalar, Scalar) {
    let mut hasher = Sha512::new();
    hasher.update(password.as_bytes());
    hasher.update(idc.as_bytes());
    hasher.update(ids.as_bytes());
    h(&hasher.finalize())
}

pub fn setup_2(phi0: Scalar, phi1: Scalar) -> (Scalar, RistrettoPoint) {
    let c = RistrettoPoint::generator() * phi1;
    (phi0, c)
}

pub fn step_1(_phi0: Scalar) -> RistrettoPoint {
    todo!("implement step 1")
}

pub fn step_2(_phi0: Scalar) -> RistrettoPoint {
    todo!("implement step 2")
}

pub fn step_3(_phi0: Scalar, _v: RistrettoPoint) -> [u8; 32] {
    todo!("implement step 3")
}

pub fn step_4(_phi0: Scalar, _u: RistrettoPoint) -> [u8; 32] {
    todo!("implement step 4")
}
