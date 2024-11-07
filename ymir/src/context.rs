use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
};

use crate::config::{Config, Environment};

#[derive(Default, Clone)]
pub struct Context {
    /// The environment in which the application is running.
    pub environment: Option<Environment>,
    /// Settings for the application.
    pub configs: Option<Config>,
    /// Extend Context
    pub extend: Option<Box<AnyMap>>,
}

impl Context {
    pub fn new() -> Context {
        Context::default()
    }

    pub fn set<T: Clone + Send + Sync + 'static>(&mut self, val: T) -> Option<Box<AnyMap>> {
        self.extend
            .get_or_insert_with(Box::default)
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(|boxed| boxed.into_any().downcast().ok().map(|boxed| *boxed))
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.extend
            .as_ref()
            .and_then(|map| map.get(&TypeId::of::<T>()))
            .and_then(|boxed| (**boxed).as_any().downcast_ref())
    }

    #[inline]
    pub fn clear(&mut self) {
        if let Some(ref mut m) = self.extend {
            m.clear();
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.extend.as_ref().map_or(true, |m| m.is_empty())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.extend.as_ref().map_or(0, |m| m.len())
    }
}

type AnyMap = HashMap<TypeId, Box<dyn AnyClone + Send + Sync>, BuildHasherDefault<HasherId>>;

#[derive(Default)]
pub struct HasherId(u64);

impl Hasher for HasherId {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

pub trait AnyClone: Any {
    fn clone_box(&self) -> Box<dyn AnyClone + Send + Sync>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Clone + Send + Sync + 'static> AnyClone for T {
    fn clone_box(&self) -> Box<dyn AnyClone + Send + Sync> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl Clone for Box<dyn AnyClone + Send + Sync> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper structs for testing
    #[derive(Clone, Debug, PartialEq)]
    struct TestString(String);

    #[derive(Clone, Debug, PartialEq)]
    struct TestNumber(i32);

    #[test]
    fn test_context_new() {
        let ctx = Context::new();
        assert!(ctx.environment.is_none());
        assert!(ctx.configs.is_none());
        assert!(ctx.extend.is_none());
        assert!(ctx.is_empty());
        assert_eq!(ctx.len(), 0);
    }

    #[test]
    fn test_context_set_and_get() {
        let mut ctx = Context::new();

        // Test setting and getting a string
        let test_string = TestString("Hello".to_string());
        ctx.set(test_string.clone());

        let retrieved_string = ctx.get::<TestString>().unwrap();
        assert_eq!(retrieved_string, &test_string);

        // Test setting and getting a number
        let test_number = TestNumber(42);
        ctx.set(test_number.clone());

        let retrieved_number = ctx.get::<TestNumber>().unwrap();
        assert_eq!(retrieved_number, &test_number);
    }

    #[test]
    fn test_context_overwrite() {
        let mut ctx = Context::new();

        ctx.set(TestNumber(42));
        ctx.set(TestNumber(24));

        let retrieved = ctx.get::<TestNumber>().unwrap();
        assert_eq!(retrieved, &TestNumber(24));
    }

    #[test]
    fn test_context_clear() {
        let mut ctx = Context::new();

        ctx.set(TestString("Hello".to_string()));
        ctx.set(TestNumber(42));

        assert_eq!(ctx.len(), 2);

        ctx.clear();

        assert!(ctx.is_empty());
        assert_eq!(ctx.len(), 0);
        assert!(ctx.get::<TestString>().is_none());
        assert!(ctx.get::<TestNumber>().is_none());
    }

    #[test]
    fn test_context_multiple_types() {
        let mut ctx = Context::new();

        ctx.set(TestString("Hello".to_string()));
        ctx.set(TestNumber(42));
        ctx.set(true);

        assert_eq!(ctx.len(), 3);
        assert_eq!(
            ctx.get::<TestString>().unwrap(),
            &TestString("Hello".to_string())
        );
        assert_eq!(ctx.get::<TestNumber>().unwrap(), &TestNumber(42));
        assert_eq!(ctx.get::<bool>().unwrap(), &true);
    }

    #[test]
    fn test_any_clone_implementation() {
        let original = TestString("Test".to_string());
        let boxed: Box<dyn AnyClone + Send + Sync> = Box::new(original.clone());

        // Test clone_box
        let cloned = boxed.clone();
        let downcast_original = boxed.as_any().downcast_ref::<TestString>();
        let downcast_cloned = cloned.as_any().downcast_ref::<TestString>();

        assert_eq!(downcast_original, downcast_cloned);
    }

    #[test]
    fn test_hasher_id() {
        let mut hasher = HasherId::default();
        let test_id = 12345u64;

        hasher.write_u64(test_id);
        assert_eq!(hasher.finish(), test_id);
    }

    #[test]
    fn test_context_get_nonexistent() {
        let ctx = Context::new();
        assert!(ctx.get::<TestString>().is_none());
    }

    #[test]
    fn test_context_size_tracking() {
        let mut ctx = Context::new();
        assert_eq!(ctx.len(), 0);

        ctx.set(TestString("Hello".to_string()));
        assert_eq!(ctx.len(), 1);

        ctx.set(TestNumber(42));
        assert_eq!(ctx.len(), 2);

        ctx.clear();
        assert_eq!(ctx.len(), 0);
    }

    #[test]
    #[should_panic(expected = "TypeId calls write_u64")]
    fn test_hasher_id_write_panic() {
        let mut hasher = HasherId::default();
        hasher.write(&[1, 2, 3]);
    }

    #[test]
    fn test_any_clone_mutations() {
        let mut ctx = Context::new();

        // Test that mutations don't affect the original
        let original = TestString("Original".to_string());
        ctx.set(original.clone());

        let new_value = TestString("New".to_string());
        ctx.set(new_value.clone());

        assert_eq!(ctx.get::<TestString>().unwrap(), &new_value);
        assert_ne!(ctx.get::<TestString>().unwrap(), &original);
    }
}
