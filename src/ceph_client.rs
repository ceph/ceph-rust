use std::collections::HashMap;

use crate::ceph::{connect_to_ceph, Rados};
use crate::cmd;
use crate::rados;

use libc::c_char;
use std::ffi::CString;
use std::{ptr, str};

use crate::error::RadosError;
use crate::{CephVersion, MonCommand, OsdOption, PoolOption};

/// A CephClient is a struct that handles communicating with Ceph
/// in a nicer, Rustier way
///
/// ```rust,no_run
/// # use ceph::CephClient;
/// # use ceph::cmd::CrushTree;
/// # use ceph::error::RadosError;
/// # fn main() {
/// #   let _ = run();
/// # }
/// # fn run() -> Result<CrushTree, RadosError> {
/// let client = CephClient::new("admin", "/etc/ceph/ceph.conf")?;
/// let tree = client.osd_tree()?;
/// # Ok(tree)
/// # }
/// ```
pub struct CephClient {
    rados_t: Rados,
    simulate: bool,
    version: CephVersion,
}

macro_rules! min_version {
    ($version:ident, $self:ident) => {{
        if $self.version < CephVersion::$version {
            return Err(RadosError::MinVersion(CephVersion::$version, $self.version));
        }
    }};
}

impl CephClient {
    pub fn new<T1: AsRef<str>, T2: AsRef<str>>(
        user_id: T1,
        config_file: T2,
    ) -> Result<CephClient, RadosError> {
        let rados_t = match connect_to_ceph(&user_id.as_ref(), &config_file.as_ref()) {
            Ok(rados_t) => rados_t,
            Err(e) => return Err(e),
        };
        let version: CephVersion = match cmd::version(&rados_t)?.parse() {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        Ok(CephClient {
            rados_t,
            simulate: false,
            version,
        })
    }

    pub fn simulate(mut self) -> Self {
        self.simulate = true;
        self
    }

    pub fn osd_out(&self, osd_id: u64) -> Result<(), RadosError> {
        let osd_id = osd_id.to_string();
        let cmd = MonCommand::new()
            .with_prefix("osd out")
            .with("ids", &osd_id);

        if !self.simulate {
            self.run_command(cmd)?;
        }
        Ok(())
    }

    pub fn osd_crush_remove(&self, osd_id: u64) -> Result<(), RadosError> {
        let osd_id = format!("osd.{}", osd_id);
        let cmd = MonCommand::new()
            .with_prefix("osd crush remove")
            .with_name(&osd_id);
        if !self.simulate {
            self.run_command(cmd)?;
        }
        Ok(())
    }

    /// Query a ceph pool.
    pub fn osd_pool_get(&self, pool: &str, choice: &PoolOption) -> Result<String, RadosError> {
        let cmd = MonCommand::new()
            .with_prefix("osd pool get")
            .with("pool", pool)
            .with("var", choice.as_ref());
        if let Ok(result) = self.run_command(cmd) {
            let mut l = result.lines();
            match l.next() {
                Some(res) => return Ok(res.into()),
                None => {
                    return Err(RadosError::Error(format!(
                        "Unable to parse osd pool get output: {:?}",
                        result,
                    )))
                }
            }
        }

        Err(RadosError::Error(
            "No response from ceph for osd pool get".into(),
        ))
    }
    /// Set a pool value
    pub fn osd_pool_set(&self, pool: &str, key: &str, value: &str) -> Result<(), RadosError> {
        let cmd = MonCommand::new()
            .with_prefix("osd pool set")
            .with("pool", pool)
            .with("var", key)
            .with("value", value);
        if !self.simulate {
            self.run_command(cmd)?;
        }
        Ok(())
    }

    /// Can be used to set options on an OSD
    ///
    /// ```rust,no_run
    /// # use ceph::{OsdOption, CephClient};
    /// # use ceph::error::RadosError;
    /// # fn main() {
    /// #   let _ = run();
    /// # }
    /// # fn run() -> Result<(), RadosError> {
    /// let client = CephClient::new("admin", "/etc/ceph/ceph.conf")?;
    /// client.osd_set(OsdOption::NoDown, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn osd_set(&self, key: OsdOption, force: bool) -> Result<(), RadosError> {
        let key = key.to_string();
        let cmd = {
            let mut c = MonCommand::new().with_prefix("osd set").with("key", &key);
            if force {
                c = c.with("sure", "--yes-i-really-mean-it");
            }
            c
        };
        if !self.simulate {
            self.run_command(cmd)?;
        }
        Ok(())
    }

    /// Can be used to unset options on an OSD
    ///
    /// ```rust,no_run
    /// # use ceph::{OsdOption, CephClient};
    /// # use ceph::error::RadosError;
    /// # fn main() {
    /// #   let _ = run();
    /// # }
    /// # fn run() -> Result<(), RadosError> {
    /// let client = CephClient::new("admin", "/etc/ceph/ceph.conf")?;
    /// client.osd_unset(OsdOption::NoDown)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn osd_unset(&self, key: OsdOption) -> Result<(), RadosError> {
        cmd::osd_unset(&self.rados_t, &key, self.simulate).map_err(|a| a)
    }

    pub fn osd_tree(&self) -> Result<cmd::CrushTree, RadosError> {
        cmd::osd_tree(&self.rados_t).map_err(|a| a)
    }

    /// Get cluster status
    pub fn status(&self) -> Result<String, RadosError> {
        let cmd = MonCommand::new().with_prefix("status").with_format("json");
        let return_data = self.run_command(cmd)?;
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => Ok(res.into()),
            None => Err(RadosError::Error("No response from ceph for status".into())),
        }
    }

    /// List all the monitors in the cluster and their current rank
    pub fn mon_dump(&self) -> Result<cmd::MonDump, RadosError> {
        Ok(cmd::mon_dump(&self.rados_t)?)
    }

    /// Get the mon quorum
    pub fn mon_quorum(&self) -> Result<String, RadosError> {
        Ok(cmd::mon_quorum(&self.rados_t)?)
    }

    /// Show mon daemon version
    pub fn version(&self) -> Result<CephVersion, RadosError> {
        cmd::version(&self.rados_t)?.parse()
    }

    pub fn osd_pool_quota_get(&self, pool: &str) -> Result<u64, RadosError> {
        Ok(cmd::osd_pool_quota_get(&self.rados_t, pool)?)
    }

    pub fn auth_del(&self, osd_id: u64) -> Result<(), RadosError> {
        Ok(cmd::auth_del(&self.rados_t, osd_id, self.simulate)?)
    }

    pub fn osd_rm(&self, osd_id: u64) -> Result<(), RadosError> {
        Ok(cmd::osd_rm(&self.rados_t, osd_id, self.simulate)?)
    }

    pub fn osd_create(&self, id: Option<u64>) -> Result<u64, RadosError> {
        Ok(cmd::osd_create(&self.rados_t, id, self.simulate)?)
    }

    // Add a new mgr to the cluster
    pub fn mgr_auth_add(&self, mgr_id: &str) -> Result<(), RadosError> {
        Ok(cmd::mgr_auth_add(&self.rados_t, mgr_id, self.simulate)?)
    }

    // Add a new osd to the cluster
    pub fn osd_auth_add(&self, osd_id: u64) -> Result<(), RadosError> {
        Ok(cmd::osd_auth_add(&self.rados_t, osd_id, self.simulate)?)
    }

    /// Get a ceph-x key.  The id parameter can be either a number or a string
    /// depending on the type of client so I went with string.
    pub fn auth_get_key(&self, client_type: &str, id: &str) -> Result<String, RadosError> {
        Ok(cmd::auth_get_key(&self.rados_t, client_type, id)?)
    }

    // ceph osd crush add {id-or-name} {weight}  [{bucket-type}={bucket-name} ...]
    /// add or update crushmap position and weight for an osd
    pub fn osd_crush_add(&self, osd_id: u64, weight: f64, host: &str) -> Result<(), RadosError> {
        Ok(cmd::osd_crush_add(
            &self.rados_t,
            osd_id,
            weight,
            host,
            self.simulate,
        )?)
    }

    // ceph osd crush reweight {id} {weight}
    /// reweight an osd in the CRUSH map
    pub fn osd_crush_reweight(&self, osd_id: u64, weight: f64) -> Result<(), RadosError> {
        Ok(cmd::osd_crush_reweight(
            &self.rados_t,
            osd_id,
            weight,
            self.simulate,
        )?)
    }

    /// check if a single osd is safe to destroy/remove
    pub fn osd_safe_to_destroy(&self, osd_id: u64) -> bool {
        cmd::osd_safe_to_destroy(&self.rados_t, osd_id)
    }

    // Luminous + only

    pub fn mgr_dump(&self) -> Result<cmd::MgrDump, RadosError> {
        min_version!(Luminous, self);
        Ok(cmd::mgr_dump(&self.rados_t)?)
    }

    pub fn mgr_fail(&self, mgr_id: &str) -> Result<(), RadosError> {
        min_version!(Luminous, self);
        Ok(cmd::mgr_fail(&self.rados_t, mgr_id, self.simulate)?)
    }

    pub fn mgr_list_modules(&self) -> Result<Vec<String>, RadosError> {
        min_version!(Luminous, self);
        Ok(cmd::mgr_list_modules(&self.rados_t)?)
    }

    pub fn mgr_list_services(&self) -> Result<Vec<String>, RadosError> {
        min_version!(Luminous, self);
        Ok(cmd::mgr_list_services(&self.rados_t)?)
    }

    pub fn mgr_enable_module(&self, module: &str, force: bool) -> Result<(), RadosError> {
        min_version!(Luminous, self);
        Ok(cmd::mgr_enable_module(
            &self.rados_t,
            module,
            force,
            self.simulate,
        )?)
    }

    pub fn mgr_disable_module(&self, module: &str) -> Result<(), RadosError> {
        min_version!(Luminous, self);
        Ok(cmd::mgr_disable_module(
            &self.rados_t,
            module,
            self.simulate,
        )?)
    }

    pub fn mgr_metadata(&self) -> Result<Vec<cmd::MgrMetadata>, RadosError> {
        min_version!(Luminous, self);
        Ok(cmd::mgr_metadata(&self.rados_t)?)
    }

    pub fn osd_metadata(&self) -> Result<Vec<cmd::OsdMetadata>, RadosError> {
        Ok(cmd::osd_metadata(&self.rados_t)?)
    }

    pub fn mgr_count_metadata(&self, property: &str) -> Result<HashMap<String, u64>, RadosError> {
        min_version!(Luminous, self);
        Ok(cmd::mgr_count_metadata(&self.rados_t, property)?)
    }

    pub fn mgr_versions(&self) -> Result<HashMap<String, u64>, RadosError> {
        min_version!(Luminous, self);
        Ok(cmd::mgr_versions(&self.rados_t)?)
    }

    pub fn run_command(&self, command: MonCommand) -> Result<String, RadosError> {
        let cmd = command.as_json();
        let data: Vec<*mut c_char> = Vec::with_capacity(1);

        debug!("Calling rados_mon_command with {:?}", cmd);
        let cmds = CString::new(cmd).unwrap();

        let mut outbuf = ptr::null_mut();
        let mut outs = ptr::null_mut();
        let mut outbuf_len = 0;
        let mut outs_len = 0;

        // Ceph librados allocates these buffers internally and the pointer that comes
        // back must be freed by call `rados_buffer_free`
        let mut str_outbuf: String = String::new();
        let mut str_outs: String = String::new();

        let ret_code = unsafe {
            // cmd length is 1 because we only allow one command at a time.
            rados::rados_mon_command(
                *self.rados_t.inner(),
                &mut cmds.as_ptr(),
                1,
                data.as_ptr() as *mut c_char,
                data.len() as usize,
                &mut outbuf,
                &mut outbuf_len,
                &mut outs,
                &mut outs_len,
            )
        };
        debug!("return code: {}", ret_code);
        if ret_code < 0 {
            if outs_len > 0 && !outs.is_null() {
                let slice =
                    unsafe { ::std::slice::from_raw_parts(outs as *const u8, outs_len as usize) };
                str_outs = String::from_utf8_lossy(slice).into_owned();

                unsafe {
                    rados::rados_buffer_free(outs);
                }
            }
            return Err(RadosError::new(format!(
                "{:?} : {}",
                RadosError::from(ret_code),
                str_outs
            )));
        }

        // Copy the data from outbuf and then  call rados_buffer_free instead libc::free
        if outbuf_len > 0 && !outbuf.is_null() {
            let slice =
                unsafe { ::std::slice::from_raw_parts(outbuf as *const u8, outbuf_len as usize) };
            str_outbuf = String::from_utf8_lossy(slice).into_owned();

            unsafe {
                rados::rados_buffer_free(outbuf);
            }
        }

        // if outs_len > 0 && !outs.is_null() {
        //     let slice = unsafe {
        //         ::std::slice::from_raw_parts(outs as *const u8, outs_len as usize)
        //     };
        //     str_outs = String::from_utf8_lossy(slice).into_owned();

        //     unsafe { rados::rados_buffer_free(outs); }
        // }
        // println!("outs: {}", str_outs);

        // Ok((str_outbuf, str_outs))
        Ok(str_outbuf)
    }
}
