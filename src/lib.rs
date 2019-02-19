use parking_lot::{Condvar, Mutex, MutexGuard, RawMutex, WaitTimeoutResult};
use std::{
    ops::{Deref, DerefMut},
    time::{Duration, Instant},
};

#[derive(Debug, Default)]
pub struct Monitor<T> {
    mutex: Mutex<T>,
    cv: Condvar,
}

impl<T> Monitor<T> {
    pub fn new(t: T) -> Self {
        Monitor {
            mutex: Mutex::new(t),
            cv: Condvar::new(),
        }
    }

    pub fn lock(&self) -> MonitorGuard<T> {
        MonitorGuard::new(&self.cv, self.mutex.lock())
    }

    pub fn try_lock(&self) -> Option<MonitorGuard<T>> {
        self.mutex
            .try_lock()
            .map(|g| MonitorGuard::new(&self.cv, g))
    }

    pub fn try_lock_for(&self, timeout: Duration) -> Option<MonitorGuard<T>> {
        self.mutex
            .try_lock_for(timeout)
            .map(|g| MonitorGuard::new(&self.cv, g))
    }

    pub fn try_lock_until(&self, timeout: Instant) -> Option<MonitorGuard<T>> {
        self.mutex
            .try_lock_until(timeout)
            .map(|g| MonitorGuard::new(&self.cv, g))
    }

    pub fn with_lock<U, F>(&self, f: F) -> U
    where
        F: FnOnce(MonitorGuard<T>) -> U,
    {
        f(self.lock())
    }

    pub fn try_with_lock<U, F>(&self, f: F) -> Option<U>
    where
        F: FnOnce(MonitorGuard<T>) -> U,
    {
        self.try_lock().map(f)
    }

    pub fn try_with_lock_for<U, F>(&self, timeout: Duration, f: F) -> Option<U>
    where
        F: FnOnce(MonitorGuard<T>) -> U,
    {
        self.try_lock_for(timeout).map(f)
    }

    pub fn try_with_lock_until<U, F>(&self, timeout: Instant, f: F) -> Option<U>
    where
        F: FnOnce(MonitorGuard<T>) -> U,
    {
        self.try_lock_until(timeout).map(f)
    }

    pub fn into_inner(self) -> T {
        self.mutex.into_inner()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.mutex.get_mut()
    }

    pub unsafe fn raw(&self) -> &RawMutex {
        self.mutex.raw()
    }

    pub unsafe fn force_unlock(&self) {
        self.mutex.force_unlock()
    }

    pub unsafe fn force_unlock_fair(&self) {
        self.mutex.force_unlock_fair()
    }
}

impl<T> From<T> for Monitor<T> {
    fn from(t: T) -> Self {
        Monitor::new(t)
    }
}

pub struct MonitorGuard<'a, T> {
    cv: &'a Condvar,
    guard: MutexGuard<'a, T>,
}

impl<'a, T> MonitorGuard<'a, T> {
    pub fn new(cv: &'a Condvar, guard: MutexGuard<'a, T>) -> Self {
        MonitorGuard { cv, guard }
    }

    pub fn notify_one(&self) {
        self.cv.notify_one();
    }

    pub fn notify_all(&self) {
        self.cv.notify_all();
    }

    pub fn wait(&mut self) {
        self.cv.wait(&mut self.guard);
    }

    pub fn wait_for(&mut self, timeout: Duration) -> WaitTimeoutResult {
        self.cv.wait_for(&mut self.guard, timeout)
    }

    pub fn wait_until(&mut self, timeout: Instant) -> WaitTimeoutResult {
        self.cv.wait_until(&mut self.guard, timeout)
    }
}

impl<T> Deref for MonitorGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<T> DerefMut for MonitorGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}
