use dices::Dice;
use fraction::ToPrimitive;

// cargo run --example build_dices
fn main() {
    // let a: u64 = 221073919720733357899776;
    let d = Dice::build_from_string("min(d6,d6)").unwrap();
    println!("dice < 6 is {}", d.prob_lt(6));
    println!("dice <= 6 is {}", d.prob_lte(6));
    // match d {
    //     Ok(d) => println!("ok, build_time: {}", d.build_time),
    //     Err(e) => println!("err: {:?}", e),
    // }
    // println!("time to construct: {:?}", d.build_time);
    // // let v = d.roll_many(10);
    // // println!("rolled: {:?}", v);
    // for (i, g) in d.distribution {
    //     println!("i: {:?} , g: {:?}", i, g.numer().unwrap().to_i64().unwrap());
    // }
}
