//! Resource ownership outlives UI cancellation: a lease ends only with its worker.
use crate::errors::{Result, VeloError};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum JobResource {
    ChatModel,
    Whisper,
}

#[derive(Default, Clone)]
pub struct JobGate(Arc<Mutex<HashMap<JobResource, Arc<str>>>>);

pub struct JobLease {
    gate: JobGate,
    resource: JobResource,
    owner: Arc<str>,
}

impl JobGate {
    pub fn try_acquire(&self, resource: JobResource, owner: &str) -> Result<JobLease> {
        let mut active = self.0.lock();
        if active.contains_key(&resource) {
            return Err(VeloError::InvalidParameter("The model is busy".into()));
        }
        let owner: Arc<str> = Arc::from(owner);
        active.insert(resource, owner.clone());
        Ok(JobLease {
            gate: self.clone(),
            resource,
            owner,
        })
    }
}

impl Drop for JobLease {
    fn drop(&mut self) {
        let mut active = self.gate.0.lock();
        if active
            .get(&self.resource)
            .is_some_and(|owner| Arc::ptr_eq(owner, &self.owner))
        {
            active.remove(&self.resource);
        }
    }
}

#[derive(Default, Clone)]
pub struct JobCancel {
    flag: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl JobCancel {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Release);
        self.notify.notify_waiters();
    }
    pub fn flag(&self) -> Arc<AtomicBool> {
        self.flag.clone()
    }
    pub async fn cancelled(&self) {
        loop {
            // Register before checking the flag, avoiding a lost broadcast.
            let notified = self.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.flag.load(Ordering::Acquire) {
                return;
            }
            notified.await;
        }
    }
}
