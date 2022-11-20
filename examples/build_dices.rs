use dices::DiceBuilder;
use fraction::ToPrimitive;
use rand::Rng;

// cargo run --example build_dices
fn main() {
    let mut rng = rand::thread_rng();
    let f: f64 = rng.gen();
    println!("{f}")
}
