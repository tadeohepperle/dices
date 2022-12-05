use dices::Dice;

// cargo run --example build_dices
fn main() {
    let d = Dice::build_from_string("abs(d6-3)").unwrap();
    println!("{:?}", d.distribution);
}
