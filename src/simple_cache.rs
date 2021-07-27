use std::{sync::Arc, marker::PhantomData, time::{Duration, Instant}};

use parking_lot::{RwLock, RwLockUpgradableReadGuard};

pub trait Function<T>: Sync + Send
where
    T: Clone + Sync + Send,
{
    fn call(&self) -> T;
}

pub struct TimeCache<T, F>
where
    F: Function<T>,
    T: Clone + Sync + Send,
{
    last_refresh: Instant,
    ttl: Duration,
    content: Option<T>,
    generate: F,
}

impl<T, F> TimeCache<T, F>
where
    F: Function<T>,
    T: Clone + Sync + Send,
{
    pub fn new(ttl: Duration, generate: F) -> Self {
        Self {
            last_refresh: Instant::now(),
            ttl,
            content: None,
            generate,
        }
    }

    fn refresh(&mut self) {
        self.last_refresh = Instant::now();
        self.content = Some(self.generate.call());
    }

    pub fn content(&mut self) -> &T {
        if !self.is_valid() {
            self.refresh();
        }

        self.content.as_ref().unwrap()
    }

    pub fn is_valid(&self) -> bool {
        self.content.is_some() && self.last_refresh.elapsed() < self.ttl
    }

    pub fn peek(&self) -> Option<&T> {
        self.content.as_ref()
    }
}

impl<T, F> OptimisticLock<T> for TimeCache<T, F>
where
    F: Function<T>,
    T: Clone + Sync + Send,
{
    fn try_read(&self) -> Option<T> {
        if self.is_valid() {
            self.peek().cloned()
        } else {
            None
        }
    }

    fn read_mut(&mut self) -> T {
        self.content().clone()
    }
}

pub trait OptimisticLock<T> {
    fn try_read(&self) -> Option<T>;
    fn read_mut(&mut self) -> T;
}

pub struct OptimisticLockCache<T, C>
where
    T: OptimisticLock<C>,
{
    cache: RwLock<T>,
    _phantom: PhantomData<C>,
}

impl<T, C> OptimisticLockCache<T, C>
where
    T: OptimisticLock<C>,
{
    pub fn new(cache: T) -> Self {
        Self {
            cache: RwLock::new(cache),
            _phantom: PhantomData,
        }
    }

    pub fn content<'a>(&self) -> C {
        let cache = self.cache.upgradable_read();
        if let Some(content) = cache.try_read() {
            content
        } else {
            let mut cache = RwLockUpgradableReadGuard::upgrade(cache);
            cache.read_mut()
        }
    }
}

pub struct OptimisticLRU<K, V> {
    cache: RwLock<lru::LruCache<K, Arc<V>>>,
}

impl<K, V> OptimisticLRU<K, V>
where
    K: std::hash::Hash + Eq,
{
    pub fn new(cap: usize) -> Self {
        Self {
            cache: RwLock::new(lru::LruCache::new(cap)),
        }
    }

    pub fn get_or_update<F>(&self, key: K, update: F) -> Arc<V>
    where
        F: FnOnce(&K) -> V,
    {
        let cache = self.cache.upgradable_read();
        if let Some(content) = cache.peek(&key) {
            content.clone()
        }else {
            let mut cache = RwLockUpgradableReadGuard::upgrade(cache);
            let content = Arc::new((update)(&key));
            cache.put(key, content.clone());
            content
        }
    }

    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }
}
