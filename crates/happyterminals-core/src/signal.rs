//! `Signal<T>` and `SignalSetter<T>` — the read/write data cells of the
//! reactive core.
//!
//! # Threading
//!
//! [`Signal<T>`] is **single-threaded**: `!Send + !Sync`. It wraps
//! `reactive_graph::signal::RwSignal::new_local`, which pins the value to the
//! thread that constructs it. Accessing the underlying arena from another
//! thread panics inside `reactive_graph`; our `PhantomData<*const ()>`
//! marker catches the same class of mistake at compile time.
//!
//! Cross-thread writes go through [`SignalSetter<T>`]. A setter is `Send`
//! (but not `Sync`) and serializes writes into an `mpsc` queue. The owning
//! thread drains the queue via [`Signal::drain_setter_queue`] — typically
//! once per frame tick, from the backend event loop (Phase 1.1).
//!
//! # Keep-alive pattern
//!
//! Each [`Signal<T>`] holds its own `mpsc::Sender` as a keep-alive so that
//! dropping every [`SignalSetter<T>`] does not disconnect the queue. See
//! RESEARCH §"Pitfall 5: `SignalSetter` dropped before drain".
//!
//! # Name collision warning
//!
//! [`SignalSetter<T>`] in this crate is **not** the same type as
//! `reactive_graph::wrappers::write::SignalSetter`. The latter is
//! `reactive_graph`'s synchronous signal-mutator handle; ours is an mpsc-queued
//! cross-thread write handle. We never re-export `reactive_graph`'s type.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::mpsc;

use reactive_graph::owner::LocalStorage;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set, Update};

/// A boxed closure that applies a pending cross-thread write when run on the
/// signal's owning thread. Must be `Send + 'static` so it can cross thread
/// boundaries through the `mpsc` queue.
type PendingOp<T> = Box<dyn FnOnce(&RwSignal<T, LocalStorage>) + Send + 'static>;

/// A fine-grained reactive cell.
///
/// Reads via [`Signal::get`] subscribe the current observer (Effect or Memo).
/// Writes via [`Signal::set`] / [`Signal::update`] trigger synchronous
/// propagation (or coalesced propagation, if wrapped in `batch`).
///
/// **Single-threaded.** `Signal<T>` is `!Send + !Sync`. Use
/// [`Signal::setter`] to obtain a `Send` handle for cross-thread writes.
///
/// `Signal<T>` is cheap to [`Clone`]: clones share the same underlying
/// reactive cell and mpsc queue.
pub struct Signal<T: 'static> {
    // Fields are implementation details; public methods carry the docs.
    inner: RwSignal<T, LocalStorage>,
    /// Render-thread-owned receiver; wrapped in `Rc<RefCell<_>>` so clones of
    /// the same `Signal` share one queue.
    rx: Rc<RefCell<mpsc::Receiver<PendingOp<T>>>>,
    /// Keep-alive sender on the owning thread. Holding this prevents mpsc
    /// `Disconnected` errors when every user-side `SignalSetter` is dropped.
    /// Deliberately unused at runtime; its lifetime is the keep-alive.
    _keepalive_tx: mpsc::Sender<PendingOp<T>>,
    /// Cloneable sender used to mint new [`SignalSetter`]s.
    tx_factory: mpsc::Sender<PendingOp<T>>,
    _not_send: PhantomData<*const ()>,
}

impl<T: 'static> Clone for Signal<T> {
    #[allow(
        clippy::used_underscore_binding,
        reason = "the `_keepalive_tx` underscore signals `never read, only held`; \
                  cloning it is still required to preserve the keep-alive invariant \
                  on the clone — see Pitfall 5"
    )]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner,
            rx: Rc::clone(&self.rx),
            _keepalive_tx: self._keepalive_tx.clone(),
            tx_factory: self.tx_factory.clone(),
            _not_send: PhantomData,
        }
    }
}

impl<T: 'static> Signal<T> {
    /// Constructs a new signal with the given initial value, pinned to the
    /// current thread.
    #[track_caller]
    pub fn new(value: T) -> Self {
        let inner = RwSignal::new_local(value);
        let (tx, rx) = mpsc::channel::<PendingOp<T>>();
        let tx_factory = tx.clone();
        Self {
            inner,
            rx: Rc::new(RefCell::new(rx)),
            _keepalive_tx: tx,
            tx_factory,
            _not_send: PhantomData,
        }
    }

    /// Returns the current value, subscribing the active observer.
    #[must_use]
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.inner.get()
    }

    /// Writes a new value, triggering dependents.
    pub fn set(&self, value: T) {
        self.inner.set(value);
    }

    /// Mutates the value in place, triggering dependents.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        self.inner.update(f);
    }

    /// Returns the current value **without** subscribing the active observer.
    #[must_use]
    pub fn untracked(&self) -> T
    where
        T: Clone,
    {
        self.inner.get_untracked()
    }

    /// Returns a [`SignalSetter<T>`]: a `Send` handle that queues writes onto
    /// this signal's mpsc channel. The writes become visible after the next
    /// [`Signal::drain_setter_queue`] call on the owning thread.
    #[must_use]
    pub fn setter(&self) -> SignalSetter<T>
    where
        T: Send + 'static,
    {
        SignalSetter {
            tx: self.tx_factory.clone(),
        }
    }

    /// Applies every pending cross-thread op from the mpsc queue in FIFO
    /// order. Backend hook: called once per frame tick by the render loop
    /// (Phase 1.1). A no-op if the queue is empty.
    pub fn drain_setter_queue(&self) {
        let rx = self.rx.borrow();
        while let Ok(op) = rx.try_recv() {
            op(&self.inner);
        }
    }
}

/// A `Send` handle for writing to a [`Signal`] from another thread.
///
/// Writes are queued in an `mpsc` channel and applied on the signal's owning
/// thread the next time [`Signal::drain_setter_queue`] runs — typically once
/// per frame tick in the backend event loop (Phase 1.1). Writes are therefore
/// **not** visible immediately to observers.
///
/// # Name collision
///
/// This is a **different type** from
/// `reactive_graph::wrappers::write::SignalSetter`. That one is
/// `reactive_graph`'s synchronous signal-mutator handle; this one is our
/// cross-thread queue. They are not interchangeable and we never re-export
/// `reactive_graph`'s type.
///
/// `SignalSetter<T>` is `Send` but not `Sync` — concurrent access from
/// multiple threads requires cloning (the inner `mpsc::Sender` is cheap to
/// clone).
pub struct SignalSetter<T: Send + 'static> {
    tx: mpsc::Sender<PendingOp<T>>,
}

impl<T: Send + 'static> Clone for SignalSetter<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<T: Send + 'static> SignalSetter<T> {
    /// Queues a write of `value` onto the signal. Visible to observers after
    /// the owning thread next calls [`Signal::drain_setter_queue`].
    ///
    /// If the owning signal has already been dropped the write is silently
    /// discarded — by design, since a dropped signal has no observers left.
    pub fn set(&self, value: T) {
        // Ignore send errors — a dropped Signal means "no longer interested".
        let _ = self.tx.send(Box::new(move |sig| sig.set(value)));
    }

    /// Queues an in-place update onto the signal. The closure runs on the
    /// owning thread when [`Signal::drain_setter_queue`] is next called.
    pub fn update(&self, f: impl FnOnce(&mut T) + Send + 'static) {
        let _ = self.tx.send(Box::new(move |sig| sig.update(f)));
    }
}
