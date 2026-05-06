use std::collections::HashMap;

/// Trait for resolving external symbol names to addresses.
pub trait SymbolResolver {
    /// Returns the address of `name`, or `None` if unknown.
    fn resolve(&self, name: &str) -> Option<u32>;
}

/// A resolver that never knows any symbol.
#[derive(Default, Debug, Clone, Copy)]
pub struct NoSymbolResolver;

impl SymbolResolver for NoSymbolResolver {
    fn resolve(&self, _name: &str) -> Option<u32> {
        None
    }
}

/// A resolver backed by a `HashMap<String, u32>`.
#[derive(Default, Debug, Clone)]
#[repr(transparent)]
pub struct HashMapSymbolResolver(HashMap<String, u32>);

#[macro_export]
macro_rules! symbols {
    (map $(($symbol:expr, $addr:expr)),+) => {
        {
            let mut res = HashMapSymbolResolver::new();
            $(
                res.insert($symbol, $addr);
            )+
            res
        }
    };

    ($(($symbol:expr, $addr:expr)),+) => {
        { symbols!(map $(($symbol, $addr)),+) }
    };

    () => {
        { HashMapSymbolResolver::new() }
    }
}

impl HashMapSymbolResolver {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, name: impl Into<String>, addr: u32) {
        self.0.insert(name.into(), addr);
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl SymbolResolver for HashMapSymbolResolver {
    fn resolve(&self, name: &str) -> Option<u32> {
        self.0.get(name).copied()
    }
}

/// A resolver built from any closure/function.
#[derive(Debug, Clone)]
pub struct FnSymbolResolver<F> {
    f: F,
}

impl<F> FnSymbolResolver<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> SymbolResolver for FnSymbolResolver<F>
where
    F: Fn(&str) -> Option<u32>,
{
    fn resolve(&self, name: &str) -> Option<u32> {
        (self.f)(name)
    }
}
