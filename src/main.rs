use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use group::Group;
use sha2::{Digest, Sha512};

fn main() {
    let msg = "Secret message";

    let hash = Sha512::digest(msg.as_bytes());
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&hash[..32]);
    let scalar = Scalar::from_bytes_mod_order(bytes);

    let generator = RistrettoPoint::generator();
    println!("Generator: {:?}", generator.compress());

    let point = RistrettoPoint::generator() * scalar;
    println!("New point: {:?}", point.compress());
}
