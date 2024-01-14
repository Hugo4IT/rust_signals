use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct Signal<T> {
    inner: T,
    generation: u32,
}

impl<T> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: value,
            generation: 1,
        }
    }

    #[inline(always)]
    pub fn flag_updated(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    #[inline]
    pub fn set(&mut self, value: T) {
        self.flag_updated();

        self.inner = value;
    }

    #[inline]
    pub fn get(&self) -> &T {
        &self.inner
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.flag_updated();

        &mut self.inner
    }

    #[inline]
    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn derive<O: Copy, F: Fn(&T) -> O>(&self, function: F) -> Derived<T, O, F> {
        let output = Cell::new(function(self.get()));
        let output_gen = self.generation;

        Derived {
            input: self as *const Signal<T>,
            output,
            output_gen,
            function,
        }
    }
}

impl<T> Deref for Signal<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for Signal<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

#[derive(Debug)]
pub struct Derived<T, O: Copy, F: Fn(&T) -> O> {
    input: *const Signal<T>,
    output: Cell<O>,
    output_gen: u32,
    function: F,
}

impl<T, O: Copy, F: Fn(&T) -> O> Derived<T, O, F> {
    pub fn get(&self) -> O {
        let signal = unsafe { &*self.input as &Signal<T> };

        if self.output_gen != signal.generation() {
            self.output.set((self.function)(signal.get()));
        }

        self.output.get()
    }
}

#[cfg(test)]
mod tests {
    use crate::Signal;

    #[test]
    fn test_memos() {
        let mut number = Signal::new(1);
        let double = number.derive(|number| number * 2);

        assert_eq!(*number, 1);
        assert_eq!(double.get(), 2);

        *number += 1;

        assert_eq!(*number, 2);
        assert_eq!(double.get(), 4);
    }
}
