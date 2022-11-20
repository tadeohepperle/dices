use dices::Dice;

// cargo run --example build_dices
fn main() {
    let d = Dice::build_from_string("d5*d5d6*d3*d4").unwrap();
    println!("time to construct: {:?}", d.build_time);
    let v = d.roll_multiple(10);
    println!("rolled: {:?}", v);
}
