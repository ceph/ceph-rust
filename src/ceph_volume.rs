extern crate serde_json;

use crate::ceph::Rados;
use crate::cmd;
use crate::error::{RadosError, RadosResult};
use crate::json::*;
use crate::CephVersion;
use crate::JsonData;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// ceph_volume is a wrapper around the ceph-volume commands
/// ceph-volume is a command line tool included in ceph versions Luminous+
/// it used to deploy and inspect OSDs using logical volumes

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LvmTags {
    #[serde(rename = "ceph.block_device")]
    pub block_device: Option<String>,
    #[serde(rename = "ceph.block_uuid")]
    pub block_uuid: Option<String>,
    #[serde(rename = "ceph.cephx_lockbox_secret")]
    pub cephx_lockbox_secret: Option<String>,
    #[serde(rename = "ceph.cluster_fsid")]
    pub cluster_fsid: Option<String>,
    #[serde(rename = "ceph.cluster_name")]
    pub cluster_name: Option<String>,
    #[serde(rename = "ceph.crush_device_class")]
    pub crush_device_class: Option<String>,
    #[serde(rename = "ceph.data_device")]
    pub data_device: Option<String>,
    #[serde(rename = "ceph.data_uuid")]
    pub data_uuid: Option<String>,
    #[serde(rename = "ceph.db_device")]
    pub db_device: Option<String>,
    #[serde(rename = "ceph.db_uuid")]
    pub db_uuid: Option<String>,
    #[serde(rename = "ceph.encrypted")]
    pub encrypted: Option<String>,
    #[serde(rename = "ceph.journal_device")]
    pub journal_device: Option<String>,
    #[serde(rename = "ceph.journal_uuid")]
    pub journal_uuid: Option<String>,
    #[serde(rename = "ceph.osd_fsid")]
    pub osd_fsid: Option<String>,
    #[serde(rename = "ceph.osd_id")]
    pub osd_id: Option<String>,
    #[serde(rename = "ceph.type")]
    pub c_type: Option<String>,
    #[serde(rename = "ceph.vdo")]
    pub vdo: Option<String>,
    #[serde(rename = "ceph.wal_device")]
    pub wal_device: Option<String>,
    #[serde(rename = "ceph.wal_uuid")]
    pub wal_uuid: Option<String>,
    //Other tags that are not listed here
    #[serde(flatten)]
    pub other_tags: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LvmMeta {
    pub devices: Vec<String>,
    pub lv_name: String,
    pub lv_path: String,
    pub lv_tags: String,
    pub lv_uuid: String,
    pub name: String,
    pub path: String,
    pub tags: LvmTags,
    #[serde(rename = "type")]
    pub lv_type: String,
    pub vg_name: String,
    // other metadata not captured through the above attributes
    #[serde(flatten)]
    pub other_meta: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum LvmData {
    Osd(LvmMeta),
    Journal {
        path: Option<String>,
        tags: Option<HashMap<String, String>>,
        #[serde(rename = "type")]
        j_type: Option<String>,
        // other metadata not captured through the above attributes
        #[serde(flatten)]
        other_meta: Option<HashMap<String, String>>,
    },
    // unknown type of ceph-volume lvm list output
    Unknown {
        //unknown metadata not captured through the above attributes
        #[serde(flatten)]
        unknown_meta: Option<HashMap<String, String>>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Lvm {
    #[serde(flatten)]
    pub metadata: LvmData,
}

// Check the cluster version. If version < Luminous, error out
fn check_version(cluster_handle: &Rados) -> RadosResult<()> {
    let version: CephVersion = cmd::version(cluster_handle)?.parse()?;
    if version < CephVersion::Luminous {
        return Err(RadosError::MinVersion(CephVersion::Luminous, version));
    }
    Ok(())
}

/// List all LVM devices (logical and physical) that may be associated with a
/// ceph cluster assuming they contain enough metadata to allow for discovery
/// Does not show devices that aren't associated with Ceph.
/// NOTE: This requires Ceph version Luminous+
pub fn ceph_volume_list(cluster_handle: &Rados) -> RadosResult<HashMap<String, Vec<Lvm>>> {
    check_version(cluster_handle)?;
    let output = Command::new("ceph-volume")
        .args(&["lvm", "list", "--format=json"])
        .output()?;
    let lvms: HashMap<String, Vec<Lvm>> =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;
    Ok(lvms)
}

/// Scan and capture important details on deployed OSDs
/// Input path, if given, must be the path to the ceph data partition,
/// so /var/lib/ceph/osd/ceph-{osd_id}
pub fn ceph_volume_scan(
    cluster_handle: &Rados,
    osd_path: Option<PathBuf>,
) -> RadosResult<JsonData> {
    check_version(cluster_handle)?;
    let output;
    if let Some(p) = osd_path {
        let path = format!("{}", p.display());
        output = Command::new("ceph-volume")
            .args(&["simple", "scan", "--stdout", &path])
            .output()?;
    } else {
        output = Command::new("ceph-volume")
            .args(&["simple", "scan", "--stdout"])
            .output()?;
    }
    let json = String::from_utf8_lossy(&output.stdout);
    let index: usize = match json.find("{") {
        Some(i) => i,
        None => 0,
    };
    // Skip stderr's.  The last output is Json
    let json = json.split_at(index);
    match json_data(&json.1) {
        Some(jsondata) => Ok(jsondata),
        _ => Err(RadosError::new("JSON data not found.".to_string())),
    }
}
