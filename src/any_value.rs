use std::any::{Any, TypeId};

/// A container for type-erased values that preserves type information
#[derive(Debug)]
pub(crate) struct AnyValue {
    pub(crate) type_id: TypeId,
    pub(crate) value: Box<dyn Any + Send + Sync>,
}

impl AnyValue {
    /// Create a new AnyValue from a value of any type that implements Any, Send, and Sync
    pub(crate) fn new<T: 'static + Any + Send + Sync>(value: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            value: Box::new(value),
        }
    }

    /// Check if the contained value is of type T
    pub(crate) fn is_type<T: 'static>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }

    /// Get a reference to the contained value if it is of type T
    pub(crate) fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.value.downcast_ref::<T>()
    }

    /// Get a mutable reference to the contained value if it is of type T
    pub(crate) fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.value.downcast_mut::<T>()
    }
}
