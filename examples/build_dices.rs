use dices::Dice;
use fraction::ToPrimitive;

// cargo run --example build_dices
fn main() {
    // let a: u64 = 221073919720733357899776;
    let d = Dice::build_from_string("8d120").unwrap();
    println!("time to construct: {:?}", d.build_time);
    // let v = d.roll_multiple(10);
    // println!("rolled: {:?}", v);
    for (i, g) in d.distribution {
        println!("i: {:?} , g: {:?}", i, g.numer().unwrap().to_i64().unwrap());
    }
}
