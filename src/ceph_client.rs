use ceph_rust::rados::rados_t;
use ceph_rust::ceph::connect_to_ceph;
use ceph_rust::cmd;

use errors::*;
use CephVersion;

/// A CephClient is a struct that handles communicating with Ceph
/// in a nicer, Rustier way
///
/// ```rust,no_run
/// # use ceph_client::errors::*;
/// # use ceph_client::{CephClient, CrushTree};
/// # fn main() { 
/// #   let _ = run();
/// # }
/// # fn run() -> Result<CrushTree> {
/// let client = CephClient::new("admin", "/etc/ceph/ceph.conf")?;
/// let tree = client.osd_tree()?;
/// # Ok(tree)
/// # }
/// ```
pub struct CephClient {
    rados_t: rados_t,
    simulate: bool,
    version: CephVersion
}

impl CephClient {
    pub fn new<T1: AsRef<str>, T2: AsRef<str>>(user_id: T1, config_file: T2) -> Result<CephClient> {
        let rados_t = match connect_to_ceph(&user_id.as_ref(), &config_file.as_ref()) {
            Ok(rados_t) => rados_t,
            Err(e) => return Err(e.into()),
        };
        let version_s = match cmd::version(rados_t) {
            Ok(v) => v,
            Err(e) => return Err(e.into()),
        };
        let version: CephVersion = match version_s.parse() {
            Ok(v) => v,
            Err(e) => return Err(e.into())
        };

        Ok(CephClient {
            rados_t: rados_t,
            simulate: false,
            version: version,
        })
    }

    pub fn simulate(mut self) -> Self {
        self.simulate = true;
        self
    }

    pub fn osd_out(&self, osd_id: u64) -> Result<()> {
        cmd::osd_out(self.rados_t, osd_id, self.simulate).map_err(|a| a.into())
    }

    pub fn osd_crush_remove(&self, osd_id: u64) -> Result<()> {
        cmd::osd_crush_remove(self.rados_t, osd_id, self.simulate).map_err(|a| a.into())
    }

    /// Query a ceph pool.
    pub fn osd_pool_get(&self, pool: &str, choice: &str) -> Result<String> {
        cmd::osd_pool_get(self.rados_t, pool, choice).map_err(|a| a.into())
    }
    /// Set a pool value
    pub fn osd_pool_set(&self, pool: &str, key: &str, value: &str) -> Result<()> {
        cmd::osd_pool_set(self.rados_t, pool, key, value, self.simulate).map_err(|a| a.into())
    }

    pub fn osd_set(&self, key: &str, force: bool) -> Result<()> {
        cmd::osd_set(self.rados_t, key, force, self.simulate).map_err(|a| a.into())
    }

    /// Possible values for the key are:
    /// full|pause|noup|nodown|noout|noin|nobackfill|norebalance|norecover|noscrub|
    /// nodeep-scrub|notieragent
    /// Check src/mon/MonCommands.h in the ceph github repo for more possible
    /// options
    pub fn osd_unset(&self, key: &str) -> Result<()> {
        cmd::osd_unset(self.rados_t, key, self.simulate).map_err(|a| a.into())
    }

    pub fn osd_tree(&self) -> Result<cmd::CrushTree> {
        cmd::osd_tree(self.rados_t).map_err(|a| a.into())
    }

    // Get cluster status
    pub fn status(&self) -> Result<String> {
        cmd::status(self.rados_t).map_err(|e| e.into())
    }

    /// List all the monitors in the cluster and their current rank
    pub fn mon_dump(&self) -> Result<cmd::MonDump> {
        cmd::mon_dump(self.rados_t).map_err(|e| e.into())
    }

    /// Get the mon quorum
    pub fn mon_quorum(&self) -> Result<String> {
        cmd::mon_quorum(self.rados_t).map_err(|e| e.into())
    }

    // Show mon daemon version
    pub fn version(&self) -> Result<String> {
        cmd::version(self.rados_t).map_err(|e| e.into())
    }

    pub fn osd_pool_quota_get(&self, pool: &str) -> Result<u64>  {
        cmd::osd_pool_quota_get(self.rados_t, pool).map_err(|e| e.into())
    }

    pub fn auth_del(&self, osd_id: u64) -> Result<()> {
        cmd::auth_del(self.rados_t, osd_id, self.simulate).map_err(|e| e.into())
    }

    pub fn osd_rm(&self, osd_id: u64) -> Result<()> {
        cmd::osd_rm(self.rados_t, osd_id, self.simulate).map_err(|e| e.into())
    }

    pub fn osd_create(&self, id: Option<u64>) -> Result<u64> {
        cmd::osd_create(self.rados_t, id, self.simulate).map_err(|e| e.into())
    }

    // Add a new mgr to the cluster
    pub fn mgr_auth_add(&self, mgr_id: &str) -> Result<()> {
        cmd::mgr_auth_add(self.rados_t, mgr_id, self.simulate).map_err(|e| e.into())
    }

    // Add a new osd to the cluster
    pub fn osd_auth_add(&self, osd_id: u64) -> Result<()> {
        cmd::osd_auth_add(self.rados_t, osd_id, self.simulate).map_err(|e| e.into())
    }

    // Luminous + only

    pub fn mgr_dump(&self) -> Result<cmd::MgrDump> {
        if self.version < CephVersion::Luminous {
            return Err(ErrorKind::MinVersion(CephVersion::Luminous, self.version).into());
        }
        cmd::mgr_dump(self.rados_t).map_err(|e| e.into())
    }
    pub fn mgr_fail(&self, mgr_id: &str) -> Result<()> {
        if self.version < CephVersion::Luminous {
            return Err(ErrorKind::MinVersion(CephVersion::Luminous, self.version).into());
        }
        cmd::mgr_fail(self.rados_t, mgr_id, self.simulate).map_err(|e| e.into())
    }
    pub fn mgr_list_modules(&self) -> Result<Vec<String>> {
        if self.version < CephVersion::Luminous {
            return Err(ErrorKind::MinVersion(CephVersion::Luminous, self.version).into());
        }
        cmd::mgr_list_modules(self.rados_t).map_err(|e| e.into())
    }
}