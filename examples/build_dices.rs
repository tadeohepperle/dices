use dices::Dice;

// cargo run --example build_dices
fn main() {
    let d = Dice::build_from_string("2d6").unwrap();
    let v = d.roll_multiple(10);
    println!("rolled: {:?}", v);
}
