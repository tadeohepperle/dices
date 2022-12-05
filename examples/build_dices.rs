use dices::Dice;

// cargo run --example build_dices
fn main() {
    let d = Dice::build_from_string("d6/20").unwrap();
    println!("{:?}", d.distribution);
}
