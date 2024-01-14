# `rust_signals`

A tiny library to add signals to Rust, inspired by SolidJS.

## Usage

```rs
// Create a signal to track changes to the contained value
let mut number = Signal::new(1);

// Create a derived signal that executes a function with
// the signal as its input. The result will be cached for
// following calls to `.get()`, until the value contained
// in the signal changes
let double = number.derive(|number| number * 2);

assert_eq!(*number, 1);
assert_eq!(double.get(), 2);

// You can mutate the contained value as if it were any
// normal &mut reference
*number += 1;

assert_eq!(*number, 2);

// The value in `number` was changed, the function gets rerun
assert_eq!(double.get(), 4);
```