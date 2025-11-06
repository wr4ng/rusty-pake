use curve25519_dalek::{RistrettoPoint, Scalar};
use group::Group;
use rand::rngs::OsRng;
use sha2::{Digest, Sha512};

fn a_point() -> RistrettoPoint {
    RistrettoPoint::hash_from_bytes::<Sha512>(b"A")
}

fn b_point() -> RistrettoPoint {
    RistrettoPoint::hash_from_bytes::<Sha512>(b"B")
}

fn h_prime(
    phi0: Scalar,
    idc: &str,
    ids: &str,
    u: RistrettoPoint,
    v: RistrettoPoint,
    w: RistrettoPoint,
    d: RistrettoPoint,
) -> [u8; 32] {
    let mut hasher = Sha512::new();
    hasher.update(phi0.as_bytes());
    hasher.update(idc.as_bytes());
    hasher.update(ids.as_bytes());
    hasher.update(u.compress().as_bytes());
    hasher.update(v.compress().as_bytes());
    hasher.update(w.compress().as_bytes());
    hasher.update(d.compress().as_bytes());
    let hash = hasher.finalize();
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

pub fn client_secret(password: &str, idc: &str, ids: &str) -> (Scalar, Scalar) {
    let mut hasher = Sha512::new();
    hasher.update(password.as_bytes());
    hasher.update(idc.as_bytes());
    hasher.update(ids.as_bytes());
    h(&hasher.finalize())
}

pub fn client_cipher(phi1: Scalar) -> RistrettoPoint {
    RistrettoPoint::generator() * phi1
}

pub fn client_initial(phi0: Scalar) -> (RistrettoPoint, Scalar) {
    let alpha = Scalar::random(&mut OsRng);
    if alpha == Scalar::ZERO {
        panic!("alpha should not be zero!");
    }
    let a = a_point();
    let g = RistrettoPoint::generator();
    let u = g * alpha + a * phi0;
    (u, alpha)
}

pub fn server_initial(phi0: Scalar) -> (RistrettoPoint, Scalar) {
    let beta = Scalar::random(&mut OsRng);
    if beta == Scalar::ZERO {
        panic!("beta should not be zero!");
    }
    let b = b_point();
    let g = RistrettoPoint::generator();
    let v = g * beta + b * phi0;
    (v, beta)
}

pub fn client_compute_key(
    idc: &str,
    ids: &str,
    phi0: Scalar,
    phi1: Scalar,
    alpha: Scalar,
    u: RistrettoPoint,
    v: RistrettoPoint,
) -> [u8; 32] {
    let b = b_point();
    let w = (v - b * phi0) * alpha;
    let d = (v - b * phi0) * phi1;
    h_prime(phi0, idc, ids, u, v, w, d)
}

pub fn server_compute_key(
    idc: &str,
    ids: &str,
    phi0: Scalar,
    c: RistrettoPoint,
    beta: Scalar,
    u: RistrettoPoint,
    v: RistrettoPoint,
) -> [u8; 32] {
    let a = a_point();
    let w = (u - a * phi0) * beta;
    let d = c * beta;
    h_prime(phi0, idc, ids, u, v, w, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distinct_a_b_g() {
        assert!(!bool::from(a_point().is_identity()));
        assert!(!bool::from(b_point().is_identity()));
        assert_ne!(a_point(), b_point());
        assert_ne!(a_point(), RistrettoPoint::generator());
        assert_ne!(b_point(), RistrettoPoint::generator());
    }

    #[test]
    fn correct_password_same_key() {
        let idc = "client";
        let ids = "server";
        let password = "password123";

        // Initial setup
        let (phi0, phi1) = client_secret(password, idc, ids);
        let c = client_cipher(phi1);

        // Step 1: Client computes u
        let (u, alpha) = client_initial(phi0);

        // Step 2: Server computes v
        let (v, beta) = server_initial(phi0);

        // Step 3: Client computes session key
        // Uses v from server
        let k_client = client_compute_key(idc, ids, phi0, phi1, alpha, u, v);

        // Step 4: Server computes session key
        // Uses u from client and c from setup_2
        let k_server = server_compute_key(idc, ids, phi0, c, beta, u, v);

        assert_eq!(k_client, k_server);
    }

    #[test]
    fn wrong_password_different_key() {
        let idc = "client";
        let ids = "server";
        let password = "password123";
        let wrong_password = "wrongpassword";

        // Initial setup
        let (phi0, phi1) = client_secret(password, idc, ids);
        let c = client_cipher(phi1);

        // Wrang setup
        let (phi0_wrong, phi1_wrong) = client_secret(wrong_password, idc, ids);

        // Step 1: Client computes u with wrong phi0
        let (u_wrong, alpha) = client_initial(phi0_wrong);

        // Step 2: Server computes v with correct saved password
        let (v, beta) = server_initial(phi0);

        // Step 3: Client computes session key with wrong phi0 and phi1
        let k_client = client_compute_key(idc, ids, phi0_wrong, phi1_wrong, alpha, u_wrong, v);

        // Step 4: Server computes session key with correct saved phi0 and c
        let k_server = server_compute_key(idc, ids, phi0, c, beta, u_wrong, v);

        assert_ne!(k_client, k_server);
    }
}
