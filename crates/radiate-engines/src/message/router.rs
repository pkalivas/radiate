// //! `Router`: a `TypeId -> Addr`s table and nothing else.
// //!
// //! ```text
// //! Router
// //!     │
// //!    Addr           <- the only mailbox in the system lives here
// //!     │
// //! Actor Mailbox
// //! ```
// //!
// //! `publish` does one `Arc::new`, one `TypeId` lookup, and a loop of
// //! `Addr::send` — no mailbox, no executor, no scheduling of its own.
// //! Serialization comes entirely from each actor's own `Mailbox` (in
// //! `actor.rs`).
// //!
// //! Delivery is via a named trait, not a closure:
// //!
// //! ```rust,ignore
// //! pub trait ErasedRecipient: Send + Sync {
// //!     fn deliver(&self, msg: &Arc<dyn Any + Send + Sync>);
// //! }
// //! ```
// //!
// //! One correction to the sketch this grew out of: the trait itself can't
// //! stay non-generic while `impl<A, M> ErasedRecipient for Addr<A> where
// //! A: Handler<Arc<M>>` tries to pick `M` out of thin air — `M` doesn't
// //! appear in `Self` (`Addr<A>`) or in the trait, so rustc rejects it as
// //! an unconstrained type parameter (E0207), and separately, if `A`
// //! handled two message types you'd need two conflicting impls of the
// //! same trait for the same concrete type. The fix is `TypedRecipient<A,
// //! M>` below: a small named struct that *bakes M into Self* via
// //! `PhantomData`, so `ErasedRecipient` can stay non-generic (needed for
// //! uniform storage in the `TypeId` map) while each `(A, M)` pair still
// //! gets its own concrete, coherent impl.
// //!
// //! This supersedes `Shared`/`EventStream::attach` and
// //! `impl EventHandler<M> for Addr<A>` from earlier revisions of
// //! `actor.rs` — removed there rather than kept as a second, redundant
// //! pub/sub path.

// use crate::message::actor::Receive;

// use super::actor::{ActorContext, ActorSystem, Addr, Ask, Message};
// use super::stream::Event;
// use std::any::{Any, TypeId};
// use std::collections::HashMap;
// use std::marker::PhantomData;
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::{Arc, RwLock};

// /// Any `Arc<T>` is a valid message with no response — what every
// /// `Handler<Arc<M>>` actor receives.
// impl<T: Send + Sync + 'static> Message for Arc<T> {
//     type Response = ();
// }

// /// What `Router` actually stores: fully erased over both `A` and `M`.
// /// The downcast happens inside `deliver`, once per delivery, same as it
// /// always did — the only thing that changed is *where* that logic lives
// /// (a named impl instead of a closure body).
// pub trait ErasedRecipient: Send + Sync {
//     fn deliver(&self, msg: &Arc<dyn Any + Send + Sync>);

//     /// Concrete, human-readable type name — `TypedRecipient<LoggingHandler,
//     /// CheckpointSaved>`, not an opaque compiler-generated closure name.
//     /// Free with a named type; not available with a closure.
//     fn type_name(&self) -> &'static str {
//         std::any::type_name::<Self>()
//     }
// }

// struct TypedRecipient<A, M> {
//     addr: Addr<A>,
//     _message: PhantomData<fn(M)>,
// }

// impl<A, M> ErasedRecipient for TypedRecipient<A, M>
// where
//     A: Receive<M>,
//     M:,
// {
//     fn deliver(&self, msg: &Arc<dyn Any + Send + Sync>) {
//         if let Some(msg) = Arc::clone(msg).downcast_ref::<M>() {
//             self.addr.send(msg.clone());
//         } else {
//             panic!(
//                 "Router delivered a message of the wrong type to {}: expected {}, got {}",
//                 self.type_name(),
//                 std::any::type_name::<M>(),
//                 msg.type_id()
//             );
//         }
//     }
// }

// // struct TypedRecipient<A, M> {
// //     addr: Addr<A>,
// //     _message: PhantomData<fn(M)>,
// // }

// // impl<A, M> ErasedRecipient for TypedRecipient<A, M>
// // where
// //     A: Receive<M>,
// //     M: ,
// // {
// //     fn deliver(&self, msg: &Arc<dyn Any + Send + Sync>) {
// //         if let Some(msg) = Arc::clone(msg).downcast_ref::<M>() {
// //             self.addr.send(msg.clone());
// //         } else {
// //             panic!(
// //                 "Router delivered a message of the wrong type to {}: expected {}, got {}",
// //                 self.type_name(),
// //                 std::any::type_name::<M>(),
// //                 msg.type_id()
// //             );
// //         }
// //     }
// // }

// struct Route {
//     // Independent of the actor's own `active` flag on `Addr` — lets you
//     // detach from *one topic* without stopping the actor, which might
//     // still be addressable directly or subscribed to other topics.
//     active: Arc<AtomicBool>,
//     recipient: Arc<dyn ErasedRecipient>,
// }

// type RouteGroup = Arc<Vec<Arc<Route>>>;
// type RouteMap = HashMap<TypeId, RouteGroup>;

// pub struct RouterSubscription {
//     active: Arc<AtomicBool>,
// }

// impl RouterSubscription {
//     pub fn unsubscribe(&self) {
//         self.active.store(false, Ordering::Release);
//     }
// }

// #[derive(Clone, Default)]
// pub struct Router {
//     routes: Arc<RwLock<RouteMap>>,
// }

// impl Router {
//     pub fn new() -> Self {
//         Router::default()
//     }

//     /// Registers `addr` to receive broadcasts of type `M`, delivered as
//     /// `Arc<M>` via `Handler<Arc<M>>`.
//     pub fn subscribe<A, M>(&self, addr: &Addr<A>) -> RouterSubscription
//     where
//         A: Receive<M>,
//         M: Message,
//     {
//         let recipient: Arc<dyn ErasedRecipient> = Arc::new(TypedRecipient {
//             addr: addr.clone(),
//             _message: PhantomData,
//         });
//         let active = Arc::new(AtomicBool::new(true));
//         let route_active = Arc::clone(&active);

//         let mut routes = self.routes.write().unwrap();
//         let group = routes
//             .entry(TypeId::of::<M>())
//             .or_insert_with(|| Arc::new(Vec::new()));
//         Arc::make_mut(group).push(Arc::new(Route {
//             active: route_active,
//             recipient,
//         }));

//         RouterSubscription { active }
//     }

//     /// Wraps `message` in a single `Arc`, looks up subscribers for its
//     /// type, and hands each a cheap `Arc` clone of the *same* allocation
//     /// — the message itself is never copied or cloned, no matter how
//     /// many subscribers there are.
//     pub fn publish<M: Event>(&self, message: M) {
//         let group = {
//             let routes = self.routes.read().unwrap();
//             match routes.get(&TypeId::of::<M>()) {
//                 Some(group) => Arc::clone(group),
//                 None => return,
//             }
//         };

//         let payload = message.into_arc();
//         for route in group.iter() {
//             if route.active.load(Ordering::Acquire) {
//                 route.recipient.deliver(&payload);
//             }
//         }
//     }
// }

// /// `ActorSystem` (spawn, lifetime) + `Router` (`TypeId` routing) wired
// /// together, with `ActorContext::publish` bridged so any actor spawned
// /// through `Bus::spawn` can broadcast back onto the same `Router` it's
// /// reachable through. This is the single field `GeneticEngine` would
// /// hold in place of today's `stream: EventStream` — everything below is
// /// what changes at that call site, and nothing else in the engine does.
// // Deliberately not `#[derive(Default)]`: that would build `actors` and
// // `router` independently and leave `actors` unwired to `router` (its
// // `with_bus` never called), silently breaking `ActorContext::publish`
// // for anything spawned through it. `new`/`Default` both go through the
// // same wiring path below instead.
// #[derive(Clone)]
// pub struct Bus {
//     pub actors: ActorSystem,
//     pub router: Router,
// }

// impl Bus {
//     pub fn new(executor: Arc<radiate_core::Executor>) -> Self {
//         let router = Router::new();
//         let actors = ActorSystem::new(executor).with_bus(router.clone());
//         Bus { actors, router }
//     }

//     pub fn spawn<A: super::actor::Actor>(&self, actor: A) -> Addr<A> {
//         self.actors.spawn(actor)
//     }

//     /// Registers `addr` for broadcasts of type `M`. Call once per
//     /// message type on the same `Addr` — that's how one actor ends up
//     /// handling several message types off a single mailbox, the same
//     /// way `LoggingHandler` does today, just spawned once instead of
//     /// re-instantiated per `subscribe` call.
//     pub fn subscribe<A, M>(&self, addr: &Addr<A>) -> RouterSubscription
//     where
//         A: Receive<M>,
//         M: Event,
//     {
//         self.router.subscribe::<A, M>(addr)
//     }

//     pub fn publish<M: Event>(&self, message: M) {
//         self.router.publish(message);
//     }
// }

// impl Default for Bus {
//     fn default() -> Self {
//         Bus::new(Arc::default())
//     }
// }

// // ---------------------------------------------------------------------
// // Migration sketch: porting `EventHandler<E>` (old, `&E` + `&EventCtx`)
// // to `Handler<Arc<E>>` (new, owned `Arc<E>` + `&ActorContext<Self>`).
// // Field access is unchanged (`Arc<E>` derefs to `E`); `ctx.publish(...)`
// // already exists on `ActorContext` with the same signature.
// // ---------------------------------------------------------------------
// //
// // // Before, in handlers.rs:
// // impl EventHandler<CheckpointSaved> for LoggingHandler {
// //     fn handle(&mut self, message: &CheckpointSaved, ctx: &EventCtx) {
// //         ctx.publish(LogEvent(
// //             LogLevel::Info,
// //             format!("Checkpoint saved at index {}: {}", message.index, message.path),
// //         ));
// //     }
// // }
// //
// // // After:
// // impl Handler<Arc<CheckpointSaved>> for LoggingHandler {
// //     fn handle(&mut self, message: Arc<CheckpointSaved>, ctx: &ActorContext<Self>) {
// //         ctx.publish(LogEvent(
// //             LogLevel::Info,
// //             format!("Checkpoint saved at index {}: {}", message.index, message.path),
// //         ));
// //     }
// // }
// //
// // // Wiring, e.g. in GeneticEngineBuilder — was `stream.subscribe(handler)`:
// // let logger = actor_system.spawn(LoggingHandler::default());
// // router.subscribe::<_, CheckpointSaved>(&logger);
// // router.subscribe::<_, LogEvent>(&logger);
// // router.subscribe::<_, Warning>(&logger);

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use radiate_core::Executor;
//     use std::sync::atomic::AtomicUsize;

//     // Deliberately *not* `Clone` — proves fan-out never needs to clone
//     // the payload, only the `Arc` pointing at it.
//     #[derive(Debug)]
//     struct BigPayload {
//         data: Vec<u8>,
//     }

//     impl Message for BigPayload {
//         type Response = ();
//     }

//     struct Collector {
//         seen: Arc<AtomicUsize>,
//     }

//     impl Ask<BigPayload> for Collector {
//         fn handle(&mut self, msg: &BigPayload, _ctx: &ActorContext<Self>) {
//             self.seen.fetch_add(msg.data.len(), Ordering::SeqCst);
//         }
//     }

//     #[test]
//     fn broadcast_fans_out_without_cloning_the_payload() {
//         let system = ActorSystem::new(Arc::new(Executor::Serial));
//         let router = Router::new();

//         let seen_a = Arc::new(AtomicUsize::new(0));
//         let seen_b = Arc::new(AtomicUsize::new(0));

//         let a = system.spawn(Collector {
//             seen: Arc::clone(&seen_a),
//         });
//         let b = system.spawn(Collector {
//             seen: Arc::clone(&seen_b),
//         });

//         router.subscribe::<_, BigPayload>(&a);
//         router.subscribe::<_, BigPayload>(&b);

//         router.publish(BigPayload {
//             data: vec![0u8; 1024],
//         });

//         assert_eq!(seen_a.load(Ordering::SeqCst), 1024);
//         assert_eq!(seen_b.load(Ordering::SeqCst), 1024);
//     }

//     #[test]
//     fn unsubscribe_stops_delivery_without_stopping_the_actor() {
//         let system = ActorSystem::new(Arc::new(Executor::Serial));
//         let router = Router::new();
//         let seen = Arc::new(AtomicUsize::new(0));
//         let addr = system.spawn(Collector {
//             seen: Arc::clone(&seen),
//         });

//         let sub = router.subscribe::<_, BigPayload>(&addr);
//         router.publish(BigPayload { data: vec![0u8; 4] });
//         assert_eq!(seen.load(Ordering::SeqCst), 4);

//         sub.unsubscribe();
//         router.publish(BigPayload { data: vec![0u8; 4] });
//         assert_eq!(seen.load(Ordering::SeqCst), 4); // unchanged

//         // Detached from this topic, but the actor itself is untouched —
//         // still directly addressable via `addr.send`/`addr.ask`.
//         assert!(addr.is_alive());
//     }

//     #[test]
//     fn recipient_type_name_is_readable_not_opaque() {
//         let system = ActorSystem::new(Arc::new(Executor::Serial));
//         let addr = system.spawn(Collector {
//             seen: Arc::new(AtomicUsize::new(0)),
//         });

//         let recipient: Arc<dyn ErasedRecipient> =
//             Arc::new(TypedRecipient::<Collector, BigPayload> {
//                 addr,
//                 _message: PhantomData,
//             });

//         // A closure's type name is compiler-generated and opaque; this
//         // is a real, greppable type.
//         assert!(recipient.type_name().contains("TypedRecipient"));
//         assert!(recipient.type_name().contains("Collector"));
//         assert!(recipient.type_name().contains("BigPayload"));
//     }

//     // --- Bus: the GeneticEngine-shaped call site -----------------------

//     #[test]
//     fn bus_spawns_and_subscribes_through_one_handle() {
//         let bus = Bus::new(Arc::new(Executor::Serial));

//         let seen = Arc::new(AtomicUsize::new(0));
//         let collector = bus.spawn(Collector {
//             seen: Arc::clone(&seen),
//         });
//         bus.subscribe::<_, BigPayload>(&collector);

//         bus.publish(BigPayload { data: vec![0u8; 3] });
//         assert_eq!(seen.load(Ordering::SeqCst), 3);
//     }

//     #[derive(Debug, Clone)]
//     struct Trigger;
//     #[derive(Debug, Clone)]
//     struct Echoed(u32);

//     struct RelayActor;
//     impl Ask<Trigger> for RelayActor {
//         fn handle(&mut self, _msg: &Trigger, ctx: &ActorContext<Self>) {
//             ctx.publish(Echoed(1));
//         }
//     }

//     struct EchoCollector {
//         seen: Arc<AtomicUsize>,
//     }
//     impl Ask<Echoed> for EchoCollector {
//         fn handle(&mut self, msg: &Echoed, _ctx: &ActorContext<Self>) {
//             self.seen.fetch_add(msg.0 as usize, Ordering::SeqCst);
//         }
//     }

//     // The point of wiring `ActorSystem::with_bus` to the same `Router`
//     // actors are subscribed through: a handler's `ctx.publish(...)`
//     // (from a broadcast delivery, same as a point-to-point one) reaches
//     // every other `Bus` subscriber, not just whoever called `publish`
//     // directly. This is what lets `LoggingHandler`-style relaying
//     // (`Warning` -> `LogEvent`) keep working unchanged.
//     #[test]
//     fn ctx_publish_closes_the_loop_through_bus() {
//         let bus = Bus::new(Arc::new(Executor::Serial));

//         let seen = Arc::new(AtomicUsize::new(0));
//         let relay = bus.spawn(RelayActor);
//         let collector = bus.spawn(EchoCollector {
//             seen: Arc::clone(&seen),
//         });

//         bus.subscribe::<_, Trigger>(&relay);
//         bus.subscribe::<_, Echoed>(&collector);

//         bus.publish(Trigger);

//         assert_eq!(seen.load(Ordering::SeqCst), 1);
//     }
// }
