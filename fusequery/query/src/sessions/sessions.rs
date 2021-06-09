// Copyright 2020-2021 The Datafuse Authors.
//
// SPDX-License-Identifier: Apache-2.0.

use std::collections::HashMap;
use std::sync::Arc;

use common_exception::ErrorCode;
use common_exception::Result;
use common_infallible::{RwLock, Mutex};
use common_planners::Partitions;
use metrics::counter;

use crate::sessions::FuseQueryContext;
use crate::sessions::FuseQueryContextRef;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct SessionManager {
    sessions: RwLock<HashMap<String, FuseQueryContextRef>>,
    max_mysql_sessions: u64,
}

pub type SessionManagerRef = Arc<SessionManager>;

impl SessionManager {
    pub fn create(max_mysql_sessions: u64) -> SessionManagerRef {
        Arc::new(SessionManager {
            sessions: RwLock::new(HashMap::with_capacity(max_mysql_sessions as usize)),
            max_mysql_sessions,
        })
    }

    pub fn max_mysql_sessions(&self) -> u64 {
        self.max_mysql_sessions
    }

    pub fn add_reject_mysql_session(&self) {

    }

    pub fn add_accepted_mysql_session(&self) {

    }

    pub fn try_create_context(&self) -> Result<FuseQueryContextRef> {
        counter!(super::metrics::METRIC_SESSION_CONNECT_NUMBERS, 1);

        let ctx = FuseQueryContext::try_create()?;
        self.sessions.write().insert(ctx.get_id()?, ctx.clone());
        Ok(ctx)
    }

    pub fn try_remove_context(&self, ctx: FuseQueryContextRef) -> Result<()> {
        counter!(super::metrics::METRIC_SESSION_CLOSE_NUMBERS, 1);

        self.sessions.write().remove(&*ctx.get_id()?);
        Ok(())
    }

    /// Fetch nums partitions from session manager by context id.
    pub fn try_fetch_partitions(&self, ctx_id: String, nums: usize) -> Result<Partitions> {
        let session_map = self.sessions.read();
        let ctx = session_map.get(&*ctx_id).ok_or_else(|| {
            ErrorCode::UnknownContextID(format!("Unsupported context id: {}", ctx_id))
        })?;
        ctx.try_get_partitions(nums)
    }
}
