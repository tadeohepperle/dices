use dices::Dice;
use fraction::ToPrimitive;

// cargo run --example build_dices
fn main() {
    // let a: u64 = 221073919720733357899776;
    let d = Dice::build_from_string("3x(d4xd5)+65");
    match d {
        Ok(d) => println!("ok, build_time: {}", d.build_time),
        Err(e) => println!("err: {:?}", e),
    }
    // println!("time to construct: {:?}", d.build_time);
    // // let v = d.roll_multiple(10);
    // // println!("rolled: {:?}", v);
    // for (i, g) in d.distribution {
    //     println!("i: {:?} , g: {:?}", i, g.numer().unwrap().to_i64().unwrap());
    // }
}
