//! Time-limited, bounded cache shared by the window identity helpers.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::catalogue::installed_apps;
use super::process::resolve_identity_for_pid;
use super::resolve::WindowIdentity;

/// Short TTL: enough to collapse repeated polls of the same window, but short
/// enough that a reused pid or a newly installed entry is picked up promptly.
const IDENTITY_TTL: Duration = Duration::from_secs(2);
const IDENTITY_CACHE_LIMIT: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct IdentityKey {
    pub(crate) app_id: String,
    pub(crate) title: String,
    pub(crate) process_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub(crate) struct CachedIdentity {
    identity: WindowIdentity,
    resolved_at: Instant,
}

/// Small bounded, time-limited cache used by the public helpers.
#[derive(Default, Debug)]
pub(crate) struct IdentityCache {
    pub(crate) entries: HashMap<IdentityKey, CachedIdentity>,
}

impl IdentityCache {
    pub(crate) fn get(
        &self,
        key: &IdentityKey,
        now: Instant,
        ttl: Duration,
    ) -> Option<WindowIdentity> {
        self.entries
            .get(key)
            .filter(|cached| now.duration_since(cached.resolved_at) < ttl)
            .map(|cached| cached.identity.clone())
    }

    pub(crate) fn insert(
        &mut self,
        key: IdentityKey,
        identity: WindowIdentity,
        now: Instant,
        ttl: Duration,
        limit: usize,
    ) {
        self.entries
            .retain(|_, cached| now.duration_since(cached.resolved_at) < ttl);
        if self.entries.len() >= limit {
            if let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, cached)| cached.resolved_at)
                .map(|(key, _)| key.clone())
            {
                self.entries.remove(&oldest);
            }
        }
        self.entries.insert(
            key,
            CachedIdentity {
                identity,
                resolved_at: now,
            },
        );
    }
}

static IDENTITY_CACHE: OnceLock<Mutex<IdentityCache>> = OnceLock::new();

/// Cached resolution shared by the display-name and icon helpers, so the label
/// and the icon always come from the very same desktop entry.
///
/// A poisoned lock is recovered after a warning: title resolution is a
/// best-effort UI helper and must never crash the caller.
pub(super) fn resolve_window_identity(
    app_id: &str,
    title: &str,
    process_id: Option<u32>,
) -> WindowIdentity {
    let key = IdentityKey {
        app_id: app_id.to_string(),
        title: title.to_string(),
        process_id,
    };
    let now = Instant::now();
    let cache = IDENTITY_CACHE.get_or_init(|| Mutex::new(IdentityCache::default()));
    {
        let cache = cache.lock().unwrap_or_else(|err| {
            tracing::warn!("identity cache lock poisoned; recovering");
            err.into_inner()
        });
        if let Some(hit) = cache.get(&key, now, IDENTITY_TTL) {
            return hit;
        }
    }
    let identity = resolve_identity_for_pid(installed_apps().as_slice(), app_id, title, process_id);
    {
        let mut cache = cache.lock().unwrap_or_else(|err| {
            tracing::warn!("identity cache lock poisoned; recovering");
            err.into_inner()
        });
        cache.insert(
            key,
            identity.clone(),
            now,
            IDENTITY_TTL,
            IDENTITY_CACHE_LIMIT,
        );
    }
    identity
}
