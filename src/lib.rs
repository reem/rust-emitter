#![feature(unboxed_closures)]
#![deny(missing_docs, warnings)]

//! A synchronous event emitter for evented code.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use std::intrinsics::TypeId;
use std::mem;

/// An event and the data associated with it.
pub trait Event<X>: 'static {}

/// The actual event emitter, it contains a lookup table for events and handlers.
pub struct EventEmitter {
    events: HashMap<TypeId, Vec<Box<Fn(&()) + Send>>>
}

/// Any type that implements Eventable gets `on` and `trigger` methods.
///
/// A type is Eventable if it contains an EventEmitter.
pub trait Eventable {
    /// Get a reference to the enclosed emitter.
    fn events(&self) -> &EventEmitter;

    /// Get a mutable reference to the enclosed emitter.
    fn events_mut(&mut self) -> &mut EventEmitter;

    /// Register a callback to be fired when an event is triggered.
    ///
    /// Many callbacks can be registered for a single event.
    fn on<E: Event<X>, F: Fn(&X) + Send, X>(&mut self, callback: F) {
        let callback: Box<Fn(&X) + Send> = box callback;
        let callback: Box<Fn(&()) + Send> = unsafe { mem::transmute(callback) };

        match self.events_mut().events.entry(TypeId::of::<E>()) {
            Entry::Occupied(mut occupied) => { occupied.get_mut().push(callback); },
            Entry::Vacant(vacant) => { vacant.set(vec![callback]); }
        };
    }

    /// Trigger an event, calling all of the associated handlers.
    fn trigger<E: Event<X>, X>(&self, event: X) {
        self.events().events.get(&TypeId::of::<E>())
            .map(|handlers| unsafe { mem::transmute(handlers) })
            .map(move |handlers: &Vec<Box<Fn(&X)>>| {
                for handler in handlers.iter() {
                    handler.call((&event,))
                }
            });
    }
}

// EventEmitter is itself eventable, so can be used directly.
impl Eventable for EventEmitter {
    fn events(&self) -> &EventEmitter { self }
    fn events_mut(&mut self) -> &mut EventEmitter { self }
}

