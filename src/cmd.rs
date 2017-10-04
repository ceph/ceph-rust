//! Ceph has a command system defined
//! in https://github.com/ceph/ceph/blob/master/src/mon/MonCommands.h
//! The cli commands mostly use this json based system.  This allows you to
//! make the exact
//! same calls without having to shell out with std::process::Command.


use ceph::ceph_mon_command_without_data;
use rados::rados_t;
use std::str::FromStr;

pub fn osd_out(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> Result<(), String> {
    let cmd = json!({
        "prefix": "osd out",
        "ids": [osd_id.to_string()]
    });
    debug!("osd out: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn osd_crush_remove(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> Result<(), String> {
    let cmd = json!({
        "prefix": "osd crush remove",
        "name": format!("osd.{}", osd_id),
    });
    debug!("osd crush remove: {:?}", cmd.to_string());
    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn auth_del(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> Result<(), String> {
    let cmd = json!({
        "prefix": "auth del",
        "entity": format!("osd.{}", osd_id)
    });
    debug!("auth del: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn osd_rm(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> Result<(), String> {
    let cmd = json!({
        "prefix": "osd rm",
        "ids": [osd_id.to_string()]
    });
    debug!("osd rm: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())
            .map_err(|e| e.to_string())?;
    }
    Ok(())

}

pub fn osd_create(cluster_handle: rados_t, id: Option<u64>, simulate: bool) -> Result<u64, String> {
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

    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())
        .map_err(|e| e.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(num) => return Ok(u64::from_str(num).map_err(|e| e.to_string())?),
            None => {
                return Err(format!(
                "Unable to parse osd create output: {:?}",
                return_data,
            ))
            },
        }
    }
    Err(format!("Unable to parse osd create output: {:?}", result))
}

pub fn auth_add(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> Result<(), String> {
    let cmd = json!({
        "prefix": "auth add",
        "entity": format!("osd.{}", osd_id),
        "caps": ["mon", "allow rwx", "osd", "allow *"],
    });
    debug!("auth_add: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn auth_get_key(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> Result<String, String> {
    let cmd = json!({
        "prefix": "auth get-key",
        "entity": format!("osd.{}", osd_id),
    });
    debug!("auth_get_key: {:?}", cmd.to_string());

    let result = ceph_mon_command_without_data(cluster_handle, &cmd.to_string())
        .map_err(|e| e.to_string())?;
    if result.0.is_some() {
        let return_data = result.0.unwrap();
        let mut l = return_data.lines();
        match l.next() {
            Some(key) => return Ok(key.into()),
            None => {
                return Err(format!(
                "Unable to parse auth get-key: {:?}",
                return_data,
            ))
            },
        }
    }
    Err(format!("Unable to parse auth get-key output: {:?}", result))
}

// ceph osd crush add {id-or-name} {weight}  [{bucket-type}={bucket-name} ...]
/// add or update crushmap position and weight for an osd
pub fn osd_crush_add(cluster_handle: rados_t, osd_id: u64, weight: f64, host: &str, simulate: bool)
    -> Result<(), String> {
    let cmd = json!({
        "prefix": "osd crush add",
        "id": osd_id,
        "weight": weight,
        "args": [format!("host={}", host)]
    });
    debug!("osd crush add: {:?}", cmd.to_string());

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd.to_string())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
