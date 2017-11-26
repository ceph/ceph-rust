use std::collections::HashMap;

use ceph_rust::rados::rados_t;
use ceph_rust::ceph::connect_to_ceph;
use ceph_rust::cmd;

use errors::*;
use {CephChoices, CephVersion};

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
    version: CephVersion,
}

impl CephClient {
    pub fn new<T1: AsRef<str>, T2: AsRef<str>>(user_id: T1, config_file: T2) -> Result<CephClient> {
        let rados_t = match connect_to_ceph(&user_id.as_ref(), &config_file.as_ref()) {
            Ok(rados_t) => rados_t,
            Err(e) => return Err(e.into()),
        };
        let version: CephVersion = match cmd::version(rados_t)?.parse() {
            Ok(v) => v,
            Err(e) => return Err(e.into()),
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

    /// Can be used to set options on an OSD
    ///
    /// ```rust,no_run
    /// # use ceph_client::errors::*;
    /// # use ceph_client::{CephChoices, CephClient};
    /// # fn main() {
    /// #   let _ = run();
    /// # }
    /// # fn run() -> Result<()> {
    /// let client = CephClient::new("admin", "/etc/ceph/ceph.conf")?;
    /// client.osd_set(CephChoices::NoDown, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn osd_set(&self, key: CephChoices, force: bool) -> Result<()> {
        cmd::osd_set(self.rados_t, key.as_ref(), force, self.simulate).map_err(|a| a.into())
    }

    /// Can be used to unset options on an OSD
    ///
    /// ```rust,no_run
    /// # use ceph_client::errors::*;
    /// # use ceph_client::{CephChoices, CephClient};
    /// # fn main() {
    /// #   let _ = run();
    /// # }
    /// # fn run() -> Result<()> {
    /// let client = CephClient::new("admin", "/etc/ceph/ceph.conf")?;
    /// client.osd_unset(CephChoices::NoDown)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn osd_unset(&self, key: CephChoices) -> Result<()> {
        cmd::osd_unset(self.rados_t, key.as_ref(), self.simulate).map_err(|a| a.into())
    }

    pub fn osd_tree(&self) -> Result<cmd::CrushTree> {
        cmd::osd_tree(self.rados_t).map_err(|a| a.into())
    }

    /// Get cluster status
    pub fn status(&self) -> Result<String> {
        Ok(cmd::status(self.rados_t)?)
    }

    /// List all the monitors in the cluster and their current rank
    pub fn mon_dump(&self) -> Result<cmd::MonDump> {
        Ok(cmd::mon_dump(self.rados_t)?)
    }

    /// Get the mon quorum
    pub fn mon_quorum(&self) -> Result<String> {
        Ok(cmd::mon_quorum(self.rados_t)?)
    }

    /// Show mon daemon version
    pub fn version(&self) -> Result<CephVersion> {
        cmd::version(self.rados_t)?
            .parse()
    }

    pub fn osd_pool_quota_get(&self, pool: &str) -> Result<u64> {
        Ok(cmd::osd_pool_quota_get(self.rados_t, pool)?)
    }

    pub fn auth_del(&self, osd_id: u64) -> Result<()> {
        Ok(cmd::auth_del(self.rados_t, osd_id, self.simulate)?)
    }

    pub fn osd_rm(&self, osd_id: u64) -> Result<()> {
        Ok(cmd::osd_rm(self.rados_t, osd_id, self.simulate)?)
    }

    pub fn osd_create(&self, id: Option<u64>) -> Result<u64> {
        Ok(cmd::osd_create(self.rados_t, id, self.simulate)?)
    }

    // Add a new mgr to the cluster
    pub fn mgr_auth_add(&self, mgr_id: &str) -> Result<()> {
        Ok(cmd::mgr_auth_add(self.rados_t, mgr_id, self.simulate)?)
    }

    // Add a new osd to the cluster
    pub fn osd_auth_add(&self, osd_id: u64) -> Result<()> {
        Ok(cmd::osd_auth_add(self.rados_t, osd_id, self.simulate)?)
    }

    /// Get a ceph-x key.  The id parameter can be either a number or a string
    /// depending on the type of client so I went with string.
    pub fn auth_get_key(&self, client_type: &str, id: &str) -> Result<String> {
        Ok(cmd::auth_get_key(self.rados_t, client_type, id)?)
    }

    // ceph osd crush add {id-or-name} {weight}  [{bucket-type}={bucket-name} ...]
    /// add or update crushmap position and weight for an osd
    pub fn osd_crush_add(&self, osd_id: u64, weight: f64, host: &str) -> Result<()> {
        Ok(cmd::osd_crush_add(self.rados_t, osd_id, weight, host, self.simulate)?)
    }

    // Luminous + only

    pub fn mgr_dump(&self) -> Result<cmd::MgrDump> {
        if self.version < CephVersion::Luminous {
            return Err(
                ErrorKind::MinVersion(CephVersion::Luminous, self.version).into(),
            );
        }
        Ok(cmd::mgr_dump(self.rados_t)?)
    }

    pub fn mgr_fail(&self, mgr_id: &str) -> Result<()> {
        if self.version < CephVersion::Luminous {
            return Err(
                ErrorKind::MinVersion(CephVersion::Luminous, self.version).into(),
            );
        }
        Ok(cmd::mgr_fail(self.rados_t, mgr_id, self.simulate)?)
    }

    pub fn mgr_list_modules(&self) -> Result<Vec<String>> {
        if self.version < CephVersion::Luminous {
            return Err(
                ErrorKind::MinVersion(CephVersion::Luminous, self.version).into(),
            );
        }
        Ok(cmd::mgr_list_modules(self.rados_t)?)
    }

    pub fn mgr_list_services(&self) -> Result<Vec<String>> {
        Ok(cmd::mgr_list_services(self.rados_t)?)
    }

    pub fn mgr_enable_module(&self, module: &str, force: bool) -> Result<()> {
        Ok(cmd::mgr_enable_module(self.rados_t, module, force, self.simulate)?)
    }

    pub fn mgr_disable_module(&self, module: &str) -> Result<()> {
        Ok(cmd::mgr_disable_module(self.rados_t, module, self.simulate)?)
    }

    pub fn mgr_metadata(&self) -> Result<cmd::MgrMetadata> {
        Ok(cmd::mgr_metadata(self.rados_t)?)
    }

    pub fn mgr_count_metadata(&self, property: &str) -> Result<HashMap<String, u64>> {
        Ok(cmd::mgr_count_metadata(self.rados_t, property)?)
    }

    pub fn mgr_versions(&self) -> Result<HashMap<String, u64>> {
        Ok(cmd::mgr_versions(self.rados_t)?)
    }
}
