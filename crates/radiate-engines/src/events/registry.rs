use crate::{
    EventScope, EventStats, StreamSnapshot, SubscriptionSnapshot,
    events::{
        Subscription,
        event::Payload,
        mailbox::{Published, Route},
    },
};
use arc_swap::{ArcSwap, Guard};
use radiate_core::SmallStr;
use smallvec::SmallVec;
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    hash::{BuildHasherDefault, Hasher},
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicU64, Ordering},
    },
};

type Groups = HashMap<TypeId, Arc<Group>, BuildHasherDefault<TypeIdHasher>>;
type Due<'a> = SmallVec<[&'a Registration; 4]>;

/// Every live registration in `map`, with the group it belongs to.
fn live(map: &Groups) -> impl Iterator<Item = (&Group, &Registration)> {
    map.values().flat_map(|group| {
        group
            .registrations
            .iter()
            .filter(|reg| reg.is_alive())
            .map(move |reg| (group.as_ref(), reg))
    })
}

/// A `TypeId` is already a hash, so the subscriber map uses its bits as-is
/// instead of running them through SipHash on every publish.
#[derive(Default)]
struct TypeIdHasher(u64);

impl Hasher for TypeIdHasher {
    fn write_u64(&mut self, n: u64) {
        self.0 = n;
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0.rotate_left(5) ^ b as u64).wrapping_mul(0x517c_c1b7_2722_0a95);
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

pub(super) enum Build {
    Always,
    IfDue,
}

#[derive(Clone)]
pub(super) struct Registration {
    pub(super) route: Arc<dyn Route>,
    pub(super) subscription: Subscription,
    pub(super) handler: TypeId,
}

impl Registration {
    pub(super) fn is_alive(&self) -> bool {
        self.subscription.is_alive()
    }
}

pub(super) struct Registry {
    groups: ArcSwap<Groups>,
    writer: Mutex<()>,
}

// --- Reading ---
impl Registry {
    #[inline]
    fn map(&self) -> Guard<Arc<Groups>> {
        self.groups.load()
    }

    pub(super) fn subscribers(&self, event: TypeId) -> Vec<SubscriptionSnapshot> {
        self.map()
            .get(&event)
            .map(|group| {
                group
                    .registrations
                    .iter()
                    .filter(|reg| reg.is_alive())
                    .map(|reg| group.info(reg))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(super) fn has_handler(&self, handler: TypeId) -> bool {
        live(&self.map()).any(|(_, reg)| reg.handler == handler)
    }

    pub(super) fn stats(&self) -> StreamSnapshot {
        let map = self.map();

        let events = map
            .values()
            .map(|group| group.counts.stats())
            .collect::<Vec<_>>();
        let subscribers = live(&map).map(|(group, reg)| group.info(reg)).collect();

        StreamSnapshot {
            event_count: events.iter().map(|e| e.published as usize).sum(),
            events,
            subscribers,
        }
    }

    pub(super) fn event_count(&self) -> usize {
        self.map()
            .values()
            .map(|group| group.counts.published.load(Ordering::Relaxed) as usize)
            .sum()
    }

    pub(super) fn event_types(&self) -> usize {
        self.map().len()
    }

    pub(super) fn mailboxes(&self) -> Vec<Arc<dyn Route>> {
        let mut seen = HashSet::new();
        live(&self.map())
            .filter(|(_, reg)| seen.insert(reg.route.id()))
            .map(|(_, reg)| Arc::clone(&reg.route))
            .collect()
    }

    pub(super) fn insert(&self, event: TypeId, name: &'static str, registration: Registration) {
        self.update(|map| {
            let group = map
                .entry(event)
                .or_insert_with(|| Arc::new(Group::new(name)));
            let registrations = &mut Arc::make_mut(group).registrations;
            registrations.retain(Registration::is_alive);
            registrations.push(registration);
        });
    }

    pub(super) fn remove_where(&self, remove: impl Fn(&Registration) -> bool) -> Vec<Subscription> {
        self.update(|map| {
            let mut removed = Vec::new();
            for group in map.values_mut() {
                if !group.registrations.iter().any(&remove) {
                    continue;
                }

                Arc::make_mut(group).registrations.retain(|reg| {
                    if remove(reg) {
                        reg.subscription.unsubscribe();
                        removed.push(reg.subscription.clone());
                        false
                    } else {
                        true
                    }
                });
            }
            removed
        })
    }

    #[inline]
    pub(super) fn publish(
        &self,
        event: TypeId,
        scope: EventScope,
        build: Build,
        payload: impl FnOnce() -> Payload,
    ) {
        let map = self.map();
        let Some(group) = map.get(&event) else {
            return;
        };

        let due = group.due();
        if due.is_empty() && matches!(build, Build::IfDue) {
            group.counts.unbuilt.fetch_add(1, Ordering::Relaxed);
            return;
        }

        group.dispatch(&due, payload(), scope);
    }

    fn update<R>(&self, f: impl FnOnce(&mut Groups) -> R) -> R {
        let (out, old) = {
            // The guarded value is (), so a poisoned lock protects nothing.
            let _writer = self.writer.lock().unwrap_or_else(PoisonError::into_inner);
            let mut next = Groups::clone(&self.map());
            let out = f(&mut next);
            (out, self.groups.swap(Arc::new(next)))
        };

        drop(old);
        out
    }
}

impl Default for Registry {
    fn default() -> Self {
        Registry {
            groups: ArcSwap::from_pointee(Groups::default()),
            writer: Mutex::default(),
        }
    }
}

#[derive(Clone)]
struct Group {
    registrations: Vec<Registration>,
    counts: Arc<EventCounts>,
}

impl Group {
    fn new(event: &'static str) -> Self {
        Group {
            registrations: Vec::new(),
            counts: Arc::new(EventCounts {
                event,
                ..Default::default()
            }),
        }
    }

    #[inline]
    fn due(&self) -> Due<'_> {
        self.registrations
            .iter()
            .filter(|reg| reg.subscription.tick())
            .collect()
    }

    #[inline]
    fn dispatch(&self, due: &[&Registration], payload: Payload, scope: EventScope) {
        let published = Published {
            payload: &payload,
            scope: &scope,
            published_at: quanta::Instant::now(),
        };

        let mut delivered = false;
        for reg in due.iter().filter(|reg| reg.is_alive()) {
            delivered = true;
            reg.subscription.record_delivered();
            reg.route.deliver(&published);
        }

        // Relaxed: drain reads these after a WaitGroup handoff, which orders them.
        self.counts.published.fetch_add(1, Ordering::Relaxed);
        if !delivered {
            self.counts.undelivered.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn info(&self, registration: &Registration) -> SubscriptionSnapshot {
        let subscription = &registration.subscription;
        SubscriptionSnapshot {
            id: subscription.id(),
            mailbox_id: subscription.mailbox_id(),
            name: subscription.name(),
            event: SmallStr::from_static(self.counts.event),
            stats: subscription.stats(),
        }
    }
}

#[derive(Default)]
struct EventCounts {
    event: &'static str,
    published: AtomicU64,
    unbuilt: AtomicU64,
    undelivered: AtomicU64,
}

impl EventCounts {
    fn stats(&self) -> EventStats {
        EventStats {
            event: SmallStr::from_static(self.event),
            published: self.published.load(Ordering::Relaxed),
            unbuilt: self.unbuilt.load(Ordering::Relaxed),
            undelivered: self.undelivered.load(Ordering::Relaxed),
        }
    }
}
