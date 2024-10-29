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
