//! Ported from `net.sourceforge.plantuml.tim.builtin.RandomFunction`.

use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%random", 2));

/// `%random([max])` or `%random(min, max)` — returns a random integer.
///
/// Uses a simple xorshift PRNG seeded from the system clock.
pub struct RandomFunction {
    state: AtomicU64,
}

impl RandomFunction {
    /// Creates a new `RandomFunction` with a time-seeded PRNG.
    pub fn new() -> Self {
        let seed = crate::wasm_time::now_nanos();
        Self {
            state: AtomicU64::new(seed | 1),
        }
    }

    fn next(&self) -> u64 {
        // xorshift64
        let mut x = self.state.load(Ordering::Relaxed);
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state.store(x, Ordering::Relaxed);
        x
    }
}

impl Default for RandomFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleReturnFunction for RandomFunction {}

crate::impl_simple_return_function!(
    RandomFunction,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0 || nb_arg == 1 || nb_arg == 2,
    execute = |self_, _context, _memory, location, values, _named| {
        match values.len() {
            0 => Ok(TValue::from_int((self_.next() % 2) as i32)),
            1 => {
                let mx = values[0].to_int();
                if mx <= 0 {
                    return Err(EaterException::new(
                        "Error on Random: bound must be positive".to_string(),
                        location,
                    ));
                }
                Ok(TValue::from_int((self_.next() % mx as u64) as i32))
            }
            2 => {
                let min = values[0].to_int();
                let max = values[1].to_int();
                let range = max - min;
                Ok(TValue::from_int((self_.next() % range as u64) as i32 + min))
            }
            _ => Err(EaterException::new(
                "Error on Random: Too many arguments".to_string(),
                location,
            )),
        }
    },
);
