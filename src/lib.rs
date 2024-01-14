use std::{
    cell::Cell,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

pub trait SignalDeriveBase {
    type Generation: Debug + Clone + Copy;
    type Value: Copy;

    fn compare_generation(&self, other: Self::Generation) -> bool;
    fn current_generation(&self) -> Self::Generation;
    fn get(&self) -> Self::Value;
}

pub trait AsSignalDeriveBase {
    type DeriveBase: SignalDeriveBase;

    fn as_derive_base(&self) -> Self::DeriveBase;
}

pub trait Derive<Output: Copy>: AsSignalDeriveBase {
    fn derive<F>(&self, function: F) -> Derived<Self::DeriveBase, Output>
    where
        F: Fn(<Self::DeriveBase as SignalDeriveBase>::Value) -> Output + 'static,
    {
        Derived::new(self.as_derive_base(), function)
    }
}

impl<Output: Copy, T: AsSignalDeriveBase> Derive<Output> for T {}

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
}

impl<T: Copy> AsSignalDeriveBase for Signal<T> {
    type DeriveBase = SignalRef<T>;

    fn as_derive_base(&self) -> Self::DeriveBase {
        Self::DeriveBase {
            inner: self as *const Signal<T>,
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

pub struct SignalRef<T> {
    inner: *const Signal<T>,
}

impl<T: Copy> SignalDeriveBase for SignalRef<T> {
    type Generation = u32;
    type Value = T;

    fn compare_generation(&self, other: Self::Generation) -> bool {
        self.current_generation() == other
    }

    fn current_generation(&self) -> Self::Generation {
        unsafe { &*self.inner as &Signal<T> }.generation()
    }

    fn get(&self) -> Self::Value {
        *unsafe { &*self.inner as &Signal<T> }.get()
    }
}

pub struct Derived<Base: SignalDeriveBase, Output: Copy> {
    base: Base,
    output: Signal<Cell<Output>>,
    output_gen: Base::Generation,
    function: Box<dyn Fn(Base::Value) -> Output>,
}

impl<Base: SignalDeriveBase, Output: Copy + Debug> Debug for Derived<Base, Output> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Derived")
            .field("output", &self.output)
            .field("output_gen", &self.output_gen)
            .finish()
    }
}

impl<Base: SignalDeriveBase, Output: Copy> Derived<Base, Output> {
    pub(crate) fn new<F: Fn(Base::Value) -> Output + 'static>(base: Base, function: F) -> Self {
        let output = Signal::new(Cell::new(function(base.get())));
        let output_gen = base.current_generation();
        let function = Box::new(function);

        Derived {
            base,
            output,
            output_gen,
            function,
        }
    }

    pub fn output_signal(&self) -> &Signal<Cell<Output>> {
        &self.output
    }

    pub fn get(&self) -> Output {
        if self.input_changed() {
            (*self.output).set((self.function)(self.base.get()));
        }

        (*self.output).get()
    }

    pub fn input_changed(&self) -> bool {
        !self.base.compare_generation(self.output_gen)
    }
}

impl<Base: SignalDeriveBase, Output: Copy> AsSignalDeriveBase for Derived<Base, Output> {
    type DeriveBase = DerivedSignalRef<Output>;

    fn as_derive_base(&self) -> Self::DeriveBase {
        Self::DeriveBase {
            inner: &self.output as *const Signal<Cell<Output>>,
        }
    }
}

pub struct DerivedSignalRef<T: Copy> {
    inner: *const Signal<Cell<T>>,
}

impl<T: Copy> SignalDeriveBase for DerivedSignalRef<T> {
    type Generation = u32;
    type Value = T;

    fn compare_generation(&self, other: Self::Generation) -> bool {
        self.current_generation() == other
    }

    fn current_generation(&self) -> Self::Generation {
        unsafe { &*self.inner as &Signal<Cell<T>> }.generation()
    }

    fn get(&self) -> Self::Value {
        unsafe { &*self.inner as &Signal<Cell<T>> }.get().get()
    }
}

impl<A: AsSignalDeriveBase, B: AsSignalDeriveBase> AsSignalDeriveBase for (&A, &B) {
    type DeriveBase = (A::DeriveBase, B::DeriveBase);

    fn as_derive_base(&self) -> Self::DeriveBase {
        (self.0.as_derive_base(), self.1.as_derive_base())
    }
}

impl<A: SignalDeriveBase, B: SignalDeriveBase> SignalDeriveBase for (A, B) {
    type Generation = (A::Generation, B::Generation);
    type Value = (A::Value, B::Value);

    fn compare_generation(&self, other: Self::Generation) -> bool {
        self.0.compare_generation(other.0) && self.1.compare_generation(other.1)
    }

    fn current_generation(&self) -> Self::Generation {
        (self.0.current_generation(), self.1.current_generation())
    }

    fn get(&self) -> Self::Value {
        (self.0.get(), self.1.get())
    }
}

#[cfg(test)]
mod tests {
    use crate::Signal;

    #[test]
    fn test_memos() {
        use crate::Derive;

        let mut number = Signal::new(1);
        let double = number.derive(|number| number * 2);

        assert_eq!(*number, 1);
        assert_eq!(double.get(), 2);

        *number += 1;

        assert_eq!(*number, 2);
        assert_eq!(double.get(), 4);
    }
}
