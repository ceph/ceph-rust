//! Ceph has a command system defined
//! in https://github.com/ceph/ceph/blob/master/src/mon/MonCommands.h
//! The cli commands mostly use this json based system.  This allows you to
//! make the exact
//! same calls without having to shell out with std::process::Command.
//! Many of the commands defined in this file have a simulate parameter to
//! allow you to test without actually calling Ceph.
extern crate serde_json;

use ceph::ceph_mon_command_without_data;
use error::RadosError;
use rados::rados_t;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Deserialize, Debug)]
pub struct CephMon {
    pub rank: i64,
    pub name: String,
    pub addr: String,
}

#[derive(Deserialize, Debug)]
pub struct CrushNode {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub crush_type: String,
    pub type_id: i64,
    pub children: Option<Vec<i64>>,
    pub crush_weight: Option<f64>,
    pub depth: Option<i64>,
    pub exists: Option<i64>,
    pub status: Option<String>,
    pub reweight: Option<f64>,
    pub primary_affinity: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct CrushTree {
    pub nodes: Vec<CrushNode>,
    pub stray: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct MgrMetadata {
    pub id: String,
    pub arch: String,
    pub ceph_version: String,
    pub cpu: String,
    pub distro: String,
    pub distro_description: String,
    pub distro_version: String,
    pub hostname: String,
    pub kernel_description: String,
    pub kernel_version: String,
    pub mem_swap_kb: u64,
    pub mem_total_kb: u64,
    pub os: String,
}

#[derive(Deserialize, Debug)]
pub struct MgrStandby {
    pub gid: u64,
    pub name: String,
    pub available_modules: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct MgrDump {
    pub epoch: u64,
    pub active_gid: u64,
    pub active_name: String,
    pub active_addr: String,
    pub available: bool,
    pub standbys: Vec<MgrStandby>,
    pub modules: Vec<String>,
    pub available_modules: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct MonDump {
    pub epoch: i64,
    pub fsid: String,
    pub modified: String,
    pub created: String,
    pub mons: Vec<CephMon>,
    pub quorum: Vec<i64>,
}

pub fn osd_out(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> Result<(), RadosError> {
    let cmd = json!({
        "prefix": "osd out",
        "ids": [osd_id.to_string()]
    });
    debug!("osd out: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    }
    Ok(())
}

pub fn osd_crush_remove(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> Result<(), RadosError> {
    let cmd = json!({
        "prefix": "osd crush remove",
        "name": format!("osd.{}", osd_id),
    });
    debug!("osd crush remove: {:?}", cmd.to_string());
    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    }
    Ok(())
}

/// Query a ceph pool.
pub fn osd_pool_get(cluster_handle: rados_t, pool: &str, choice: &str) -> Result<String, RadosError> {
    let cmd = json!({
        "prefix": "osd pool get",
        "pool": pool,
        "var": choice,
    });
    debug!("osd pool get: {:?}", cmd.to_string());
    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(res.into()),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse osd pool get output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for osd pool get".into()))
}

/// Set a pool value
pub fn osd_pool_set(cluster_handle: rados_t, pool: &str, key: &str, value: &str, simulate: bool)
    -> Result<(), RadosError> {
    let cmd = json!({
        "prefix": "osd pool set",
        "pool": pool,
        "var": key,
        "val": value,
    });
    debug!("osd pool set: {:?}", cmd.to_string());
    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    }
    Ok(())
}

pub fn osd_set(cluster_handle: rados_t, key: &str, force: bool, simulate: bool) -> Result<(), RadosError> {
    let cmd = match force {
        true => {
            json!({
                "prefix": "osd set",
                "key": key,
                "sure": "--yes-i-really-mean-it",
            })
        },
        false => {
            json!({
                "prefix": "osd set",
                "key": key,
            })
        },
    };
    debug!("osd set: {:?}", cmd.to_string());
    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    }
    Ok(())
}

/// Possible values for the key are:
/// full|pause|noup|nodown|noout|noin|nobackfill|norebalance|norecover|noscrub|
/// nodeep-scrub|notieragent
/// Check src/mon/MonCommands.h in the ceph github repo for more possible
/// options
pub fn osd_unset(cluster_handle: rados_t, key: &str, simulate: bool) -> Result<(), RadosError> {
    let cmd = json!({
        "prefix": "osd unset",
        "key": key,
    });
    debug!("osd unset: {:?}", cmd.to_string());
    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    }
    Ok(())
}

pub fn osd_tree(cluster_handle: rados_t) -> Result<CrushTree, RadosError> {
    let cmd = json!({
        "prefix": "osd tree",
        "format": "json"
    });
    debug!("osd tree {:?}", cmd.to_string());
    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse osd tree output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for osd tree".into()))
}

// Get cluster status
pub fn status(cluster_handle: rados_t) -> Result<String, RadosError> {
    let cmd = json!({
        "prefix": "status",
        "format": "json"
    });
    debug!("status {:?}", cmd.to_string());
    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(res.into()),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse status output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for status".into()))
}

/// List all the monitors in the cluster and their current rank
pub fn mon_dump(cluster_handle: rados_t) -> Result<MonDump, RadosError> {
    let cmd = json!({
        "prefix": "mon dump",
        "format": "json"
    });
    debug!("mon dump {:?}", cmd.to_string());
    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mon dump output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for mon dump".into()))
}

/// Get the mon quorum
pub fn mon_quorum(cluster_handle: rados_t) -> Result<String, RadosError> {
    let cmd = json!({
        "prefix": "quorum_status",
        "format": "json"
    });
    debug!("quorum_status {:?}", cmd.to_string());
    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse quorum_status output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for quorum_status".into()))
}

// Show mon daemon version
pub fn version(cluster_handle: rados_t) -> Result<String, RadosError> {
    let cmd = json!({
        "prefix": "version",
    });
    debug!("version: {:?}", cmd.to_string());
    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(res.to_string()),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse version output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for version".into()))
}


pub fn osd_pool_quota_get(cluster_handle: rados_t, pool: &str) -> Result<u64, RadosError> {
    let cmd = json!({
        "prefix": "osd pool get-quota",
        "pool": pool
    });
    debug!("osd pool quota-get: {:?}", cmd.to_string());
    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(u64::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse osd pool quota-get output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for osd pool quota-get".into()))
}

pub fn auth_del(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> Result<(), RadosError> {
    let cmd = json!({
        "prefix": "auth del",
        "entity": format!("osd.{}", osd_id)
    });
    debug!("auth del: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    }
    Ok(())
}

pub fn osd_rm(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> Result<(), RadosError> {
    let cmd = json!({
        "prefix": "osd rm",
        "ids": [osd_id.to_string()]
    });
    debug!("osd rm: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    }
    Ok(())

}

pub fn osd_create(cluster_handle: rados_t, id: Option<u64>, simulate: bool) -> Result<u64, RadosError> {
    let cmd = match id {
        Some(osd_id) => {
            json!({
                "prefix": "osd create",
                "id": format!("osd.{}", osd_id),
            })
        },
        None => {
            json!({
                "prefix": "osd create"
            })
        },
    };
    debug!("osd create: {:?}", cmd.to_string());

    if simulate {
        return Ok(0);
    }

    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(num) => return Ok(u64::from_str(num)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse osd create output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse osd create output: {:?}", result)))
}

// Add a new mgr to the cluster
pub fn mgr_auth_add(cluster_handle: rados_t, mgr_id: &str, simulate: bool) -> Result<(), RadosError> {
    let cmd = json!({
        "prefix": "auth add",
        "entity": format!("mgr.{}", mgr_id),
        "caps": ["mon", "allow profile mgr", "osd", "allow *", "mds", "allow *"],
    });
    debug!("auth_add: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    }
    Ok(())
}

// Add a new osd to the cluster
pub fn osd_auth_add(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> Result<(), RadosError> {
    let cmd = json!({
        "prefix": "auth add",
        "entity": format!("osd.{}", osd_id),
        "caps": ["mon", "allow rwx", "osd", "allow *"],
    });
    debug!("auth_add: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    }
    Ok(())
}

/// Get a ceph-x key.  The id parameter can be either a number or a string
/// depending on the type of client so I went with string.
pub fn auth_get_key(cluster_handle: rados_t, client_type: &str, id: &str) -> Result<String, RadosError> {
    let cmd = json!({
        "prefix": "auth get-key",
        "entity": format!("{}.{}", client_type, id),
    });
    debug!("auth_get_key: {:?}", cmd.to_string());

    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(key) => return Ok(key.into()),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse auth get-key: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse auth get-key output: {:?}", result)))
}

// ceph osd crush add {id-or-name} {weight}  [{bucket-type}={bucket-name} ...]
/// add or update crushmap position and weight for an osd
pub fn osd_crush_add(cluster_handle: rados_t, osd_id: u64, weight: f64, host: &str, simulate: bool)
    -> Result<(), RadosError> {
    let cmd = json!({
        "prefix": "osd crush add",
        "id": osd_id,
        "weight": weight,
        "args": [format!("host={}", host)]
    });
    debug!("osd crush add: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    }
    Ok(())
}

// Luminous mgr commands below

/// dump the latest MgrMap
pub fn mgr_dump(cluster_handle: rados_t) -> Result<MgrDump, RadosError> {
    let cmd = json!({
        "prefix": "mgr dump",
    });
    debug!("mgr dump: {:?}", cmd.to_string());

    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mgr dump: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse mgr dump output: {:?}", result)))
}

/// Treat the named manager daemon as failed
pub fn mgr_fail(cluster_handle: rados_t, mgr_id: &str, simulate: bool) -> Result<(), RadosError> {
    let cmd = json!({
        "prefix": "mgr fail",
        "name": mgr_id,
    });
    debug!("mgr fail cmd: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    }
    Ok(())
}

/// List active mgr modules
pub fn mgr_list_modules(cluster_handle: rados_t) -> Result<Vec<String>, RadosError> {
    let cmd = json!({
        "prefix": "mgr module ls",
    });
    debug!("mgr module ls: {:?}", cmd.to_string());

    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mgr module ls: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse mgr ls output: {:?}", result)))
}

/// List service endpoints provided by mgr modules
pub fn mgr_list_services(cluster_handle: rados_t) -> Result<Vec<String>, RadosError> {
    let cmd = json!({
        "prefix": "mgr services",
    });
    debug!("mgr services: {:?}", cmd.to_string());

    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mgr services: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse mgr services output: {:?}", result)))
}

/// Enable a mgr module
pub fn mgr_enable_module(cluster_handle: rados_t, module: &str, force: bool, simulate: bool) -> Result<(), RadosError> {
    let cmd = match force {
        true => {
            json!({
                    "prefix": "mgr module enable",
                    "module": module,
                    "force": "--force",
                })
        },
        false => {
            json!({
                    "prefix": "mgr module enable",
                    "module": module,
                })
        },
    };
    debug!("mgr module enable cmd: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    }
    Ok(())
}

/// Disable a mgr module
pub fn mgr_disable_module(cluster_handle: rados_t, module: &str, simulate: bool) -> Result<(), RadosError> {
    let cmd = json!({
        "prefix": "mgr module disable",
        "module": module,
    });
    debug!("mgr module disable cmd: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    }
    Ok(())
}

/// dump metadata for all daemons
pub fn mgr_metadata(cluster_handle: rados_t) -> Result<MgrMetadata, RadosError> {
    let cmd = json!({
        "prefix": "mgr metadata",
    });
    debug!("mgr metadata: {:?}", cmd.to_string());

    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mgr metadata: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse mgr metadata output: {:?}", result)))
}

/// count ceph-mgr daemons by metadata field property
pub fn mgr_count_metadata(cluster_handle: rados_t, property: &str) -> Result<HashMap<String, u64>, RadosError> {
    let cmd = json!({
        "prefix": "mgr count-metadata",
        "name": property,
    });
    debug!("mgr count-metadata: {:?}", cmd.to_string());

    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mgr count-metadata: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse mgr count-metadata output: {:?}", result)))
}

/// check running versions of ceph-mgr daemons
pub fn mgr_versions(cluster_handle: rados_t) -> Result<HashMap<String, u64>, RadosError> {
    let cmd = json!({
        "prefix": "mgr versions",
    });
    debug!("mgr versions: {:?}", cmd.to_string());

    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mgr versions: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse mgr versions output: {:?}", result)))
}
