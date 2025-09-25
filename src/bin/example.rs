use curve25519_dalek::ristretto::RistrettoPoint;
use group::Group;
use rusty_pake::pake::setup_1;

fn main() {
    let (scalar, _) = setup_1("password", "idc", "ids");

    let generator = RistrettoPoint::generator();
    println!("Generator: {:?}", generator.compress());

    let point = RistrettoPoint::generator() * scalar;
    println!("New point: {:?}", point.compress());
}
