//! Type-erased scene node properties: [`PropValue`], [`AnyReactive`], [`PropMap`].
//!
//! Props can be static values or reactive bindings (Signal/Memo). The
//! [`AnyReactive`] trait enables type-erased reads of reactive values so the
//! runtime can subscribe per-node Effects without knowing `T` at the call site.
//!
//! # Send/Sync
//!
//! `PropValue` does **not** require `Send + Sync` on the inner `Box<dyn Any>`.
//! `Signal<T>` and `Memo<T>` are `!Send + !Sync`; the Rust builder path holds
//! actual signal references. The JSON/Python path (Phase 3.2) will use
//! `BindIr::Signal(SignalRef)` which IS Send -- a different representation for
//! the same concept.

use std::any::Any;
use std::collections::HashMap;
use std::fmt;

use happyterminals_core::{Memo, Signal};

/// Trait for type-erased reactive reads.
///
/// When `get_any` is called inside an [`Effect`](happyterminals_core::Effect),
/// it registers a dependency -- enabling fine-grained per-node invalidation.
pub trait AnyReactive {
    /// Read the current value as a type-erased `Box<dyn Any>`.
    /// When called inside an Effect, registers a dependency.
    fn get_any(&self) -> Box<dyn Any>;

    /// Read the current value without subscribing.
    fn get_any_untracked(&self) -> Box<dyn Any>;
}

impl<T: Clone + 'static> AnyReactive for Signal<T> {
    fn get_any(&self) -> Box<dyn Any> {
        Box::new(self.get())
    }

    fn get_any_untracked(&self) -> Box<dyn Any> {
        Box::new(self.untracked())
    }
}

impl<T: Clone + PartialEq + 'static> AnyReactive for Memo<T> {
    fn get_any(&self) -> Box<dyn Any> {
        Box::new(self.get())
    }

    fn get_any_untracked(&self) -> Box<dyn Any> {
        Box::new(self.untracked())
    }
}

/// A scene node property value -- either a static value or a reactive binding.
pub enum PropValue {
    /// A constant value, set once at build time.
    Static(Box<dyn Any>),
    /// A signal or memo binding -- read inside an Effect for fine-grained updates.
    Reactive(Box<dyn AnyReactive>),
}

impl PropValue {
    /// Downcast a static prop to `&T`. Returns `None` if the prop is reactive
    /// or if the type does not match.
    #[must_use]
    pub fn get<T: 'static>(&self) -> Option<&T> {
        match self {
            Self::Static(boxed) => boxed.downcast_ref::<T>(),
            Self::Reactive(_) => None,
        }
    }

    /// Read the current value as an owned `T`. Works for both `Static` (downcast
    /// + clone) and `Reactive` (`get_any` + downcast).
    #[must_use]
    pub fn read<T: Clone + 'static>(&self) -> Option<T> {
        match self {
            Self::Static(boxed) => boxed.downcast_ref::<T>().cloned(),
            Self::Reactive(reactive) => {
                let any = reactive.get_any();
                any.downcast_ref::<T>().cloned()
            }
        }
    }
}

impl fmt::Debug for PropValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(_) => f.write_str("PropValue::Static(...)"),
            Self::Reactive(_) => f.write_str("PropValue::Reactive(...)"),
        }
    }
}

/// A type alias for the node property map.
pub type PropMap = HashMap<String, PropValue>;
