A crate for calculating discrete probability distributions of dice.

## Build for WASM

#### For bundler (default):

```
wasm-pack build --release --no-default-features --features wasm
```

#### With better error messages:

```
wasm-pack build --release --no-default-features --features wasm --features console_error_panic_hook
```

needs to have this in Cargo.toml:

```
[profile.release]
debug = true
```

#### If you don't use a web bundler:

```
wasm-pack build --target web --release --features wasm --no-default-features
```

## Create and use Dice

To create a [`Dice`], build it from a [`DiceBuilder`] or directly from a string:

```

let dice: Dice = DiceBuilder::from_string("2d6").unwrap().build()
let dice: Dice = Dice::build_from_string("2d6").unwrap()

```

---

Properties of these dice are calculated in the `build()` function:

```

min: 2
max: 12
mode: vec![7],
mean: 7,
median: 7,
distribution: vec![(2, 1/36), (3, 1/18), (4, 1/12), (5, 1/9), (5, 1/9), (6, 5/36), (7, 1/6), ...]
cumulative_distribution: vec![(2, 1/36), (3, 1/12), (4, 1/6), ...]

```

A DiceBuildingError could be returned, if the `input` string could not be parsed into a proper syntax tree for the [`DiceBuilder`].

---

To roll a [`Dice`] call the `roll()` function:

```

let num = dice.roll();
// num will be some i64 between 2 and 12, sampled according to the dices distribution

```

For rolling multiple times call the `roll_many()` function:

```

let nums = dice.roll_many(10);
// nums could be vec![7,3,9,11,7,8,5,6,3,6]

```

---

# Syntax Examples:

Some exaple strings that can be passed into the `DiceBuilder::from_string(input)` function

3 six-sided dice:

```txt
"3d6", "3w6" or "3xw6"
```

one six-sided die multiplied by 3:

```txt
"3*d6" or "d6*3"
```

rolling one or two six sided dice and summing them up

```txt
"d2xd6"
```

the maximum of two six-sided-dice minus the minimum of two six sided dice

```txt
"max(d6,d6)-min(d6,d6)""
```

rolling a die but any value below 2 becomes 2 and above 5 becomes 5

```txt
"min(max(2,d6),5)"
```

multiplying 3 20-sided-dice

```txt
"d20*d20*d20"
```

# Background Information

This [`crate`] uses the [`BigFraction`](fraction::BigFraction) data type from the [`fraction`](fraction) crate to represent probabilities
This is quite nice because it allows for precise probabilities with infinite precision.
The drawback is that it is less efficient than using floats.

While `"d100*d100"` takes about 100ms for me, something like "d10xd100" took 9.000 ms to finish calculating the probability distribution.
There is room for optimization.
