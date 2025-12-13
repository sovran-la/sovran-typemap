// src/traits.rs
use crate::MapError;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

pub(crate) struct TypeMapValue {
    concrete_type_id: TypeId,
    trait_type_id: TypeId,
    concrete_value: Box<dyn Any + Send + Sync>,
    trait_object: Box<dyn Any + Send + Sync>,
}

impl TypeMapValue {
    pub fn new<T: ?Sized + Any + Send + Sync, U: 'static + Send + Sync>(value: U) -> Self
    where
        U: Into<Box<T>> + Clone,
    {
        let concrete = Box::new(value.clone());
        let trait_obj = Box::new(value.into());

        Self {
            concrete_type_id: TypeId::of::<U>(),
            trait_type_id: TypeId::of::<T>(),
            concrete_value: concrete,
            trait_object: trait_obj,
        }
    }
}

pub struct TraitTypeMap<K> {
    items: Arc<Mutex<HashMap<K, TypeMapValue>>>,
}

impl<K> TraitTypeMap<K>
where
    K: Clone + Eq + Hash + Debug,
{
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_trait<T, U>(&self, key: K, value: U) -> Result<(), MapError>
    where
        T: ?Sized + Any + Send + Sync + 'static,
        U: 'static + Into<Box<T>> + Send + Sync + Clone,
    {
        let type_map_value = TypeMapValue {
            concrete_type_id: TypeId::of::<U>(),
            trait_type_id: TypeId::of::<T>(),
            concrete_value: Box::new(value.clone()),
            trait_object: Box::new(value.into()),
        };

        let mut store = self.items.lock().map_err(|_| MapError::LockError)?;
        store.insert(key, type_map_value);
        Ok(())
    }

    pub fn with<V: 'static, F, R>(&self, key: &K, f: F) -> Result<R, MapError>
    where
        F: FnOnce(&V) -> R,
    {
        let guard = self.items.lock().map_err(|_| MapError::LockError)?;
        let value = guard
            .get(key)
            .ok_or_else(|| MapError::KeyNotFound(format!("{:?}", key)))?;

        if value.concrete_type_id == TypeId::of::<V>() {
            if let Some(concrete) = value.concrete_value.downcast_ref::<V>() {
                return Ok(f(concrete));
            }
        }

        Err(MapError::TypeMismatch)
    }

    pub fn with_mut<V: 'static, F, R>(&self, key: &K, f: F) -> Result<R, MapError>
    where
        F: FnOnce(&mut V) -> R,
    {
        let mut guard = self.items.lock().map_err(|_| MapError::LockError)?;
        let value = guard
            .get_mut(key)
            .ok_or_else(|| MapError::KeyNotFound(format!("{:?}", key)))?;

        if value.concrete_type_id == TypeId::of::<V>() {
            if let Some(concrete) = value.concrete_value.downcast_mut::<V>() {
                return Ok(f(concrete));
            }
        }

        Err(MapError::TypeMismatch)
    }

    pub fn with_trait<T, F, R>(&self, key: &K, f: F) -> Result<R, MapError>
    where
        T: ?Sized + Any + Send + Sync + 'static,
        F: FnOnce(&T) -> R,
    {
        let guard = self.items.lock().map_err(|_| MapError::LockError)?;
        let value = guard
            .get(key)
            .ok_or_else(|| MapError::KeyNotFound(format!("{:?}", key)))?;

        if value.trait_type_id == TypeId::of::<T>() {
            if let Some(boxed_trait) = value.trait_object.downcast_ref::<Box<T>>() {
                return Ok(f(&**boxed_trait));
            }
        }

        Err(MapError::TypeMismatch)
    }
}

impl<K> Default for TraitTypeMap<K>
where
    K: Clone + Eq + Hash + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}
