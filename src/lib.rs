pub mod client;
pub mod pake;
pub mod server;
pub mod shared;

#[cfg(test)]
mod tests {
    use curve25519_dalek::{RistrettoPoint, Scalar};
    use group::Group;
    use sha2::{Digest, Sha512};

    #[test]
    fn inverse_scalar() {
        let hash = Sha512::digest(b"some point");
        let mut left = [0u8; 32];
        left.copy_from_slice(&hash[..32]);
        let s = Scalar::from_bytes_mod_order(left);
        let s_inv = s.invert();
        let g = RistrettoPoint::generator();
        let p = g * s;
        let q = p * s_inv;
        // g^(s * s^-1) = g^1 = g
        assert_eq!(g, q);
    }

    #[test]
    fn test_group_mul_div() {
        let x = Scalar::hash_from_bytes::<Sha512>(b"test scalar");
        let g = RistrettoPoint::generator();
        let p = RistrettoPoint::hash_from_bytes::<Sha512>(b"test point");
        let a = g + (p * (-x));
        let b = g - (p * x);
        // g/(p^x) = g * p^(-x)
        assert_eq!(a, b);
    }
}
