#[cfg(not(feature = "wasm"))]
use rand::Rng;

#[cfg(not(feature = "wasm"))]
pub type WasmSafeInstant = std::time::Instant;

#[cfg(not(feature = "wasm"))]
pub fn elapsed_millis(instant: &WasmSafeInstant) -> u64 {
    instant.elapsed().as_millis() as u64
}

#[cfg(feature = "wasm")]
pub struct WasmSafeInstant {
    start: f64,
}

#[cfg(feature = "wasm")]
impl WasmSafeInstant {
    pub fn now() -> WasmSafeInstant {
        WasmSafeInstant {
            start: time_now_in_ms(),
        }
    }
}

#[cfg(feature = "wasm")]
pub fn elapsed_millis(instant: &WasmSafeInstant) -> u64 {
    let end = time_now_in_ms();
    let start = instant.start;
    (end - start) as u64
}

#[cfg(feature = "wasm")]
fn time_now_in_ms() -> f64 {
    js_sys::Date::new_0().value_of()
}

#[cfg(feature = "wasm")]
pub fn random_number_between_0_and_1() -> f64 {
    js_sys::Math::random()
}

#[cfg(not(feature = "wasm"))]
pub fn random_number_between_0_and_1() -> f64 {
    let mut rng = rand::thread_rng();
    let f: f64 = rng.gen();
    f
}
