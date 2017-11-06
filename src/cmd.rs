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
use std::str::FromStr;

#[derive(Deserialize, Debug)]
pub struct CephMon {
    pub rank: i64,
    pub name: String,
    pub addr: String,
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

/// Possible values for the key are:
/// full|pause|noup|nodown|noout|noin|nobackfill|norebalance|norecover|noscrub|nodeep-scrub
/// |notieragent|sortbitwise|recovery_deletes|require_jewel_osds|require_kraken_osds.
/// Check src/mon/MonCommands.h in the ceph github repo for more possible
/// options
/// Setting force=true will add --yes-i-really-mean-it
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

pub fn osd_tree(cluster_handle: rados_t) -> Result<CrushTree, RadosError>{
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

/// List all the monitors in the cluster and their current rank
pub fn mon_dump(cluster_handle: rados_t) -> Result<MonDump, RadosError>{
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

pub fn auth_add(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> Result<(), RadosError> {
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

pub fn auth_get_key(cluster_handle: rados_t, osd_id: u64) -> Result<String, RadosError> {
    let cmd = json!({
        "prefix": "auth get-key",
        "entity": format!("osd.{}", osd_id),
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
