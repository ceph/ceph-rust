//! Ceph has a command system defined
//! in https://github.com/ceph/ceph/blob/master/src/mon/MonCommands.h
//! The cli commands mostly use this json based system.  This allows you to
//! make the exact
//! same calls without having to shell out with std::process::Command.
//! Many of the commands defined in this file have a simulate parameter to
//! allow you to test without actually calling Ceph.
extern crate serde_json;

use crate::ceph::Rados;
use crate::error::{RadosError, RadosResult};
use crate::CephVersion;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

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

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Mem {
    MemNum {
        mem_swap_kb: u64,
        mem_total_kb: u64,
    },
    MemStr {
        mem_swap_kb: String,
        mem_total_kb: String,
    },
}

#[derive(Deserialize, Debug)]
/// Manager Metadata
pub struct MgrMetadata {
    #[serde(alias = "name")]
    pub id: String,
    pub addr: Option<String>, //nautilous
    pub addrs: Option<String>,
    pub arch: String,
    pub ceph_release: Option<String>,
    pub ceph_version: String,
    pub ceph_version_short: Option<String>,
    pub cpu: String,
    pub distro: String,
    pub distro_description: String,
    pub distro_version: String,
    pub hostname: String,
    pub kernel_description: String,
    pub kernel_version: String,
    #[serde(flatten)]
    pub mem: Mem,
    pub os: String,
    // other metadata not captured through the above attributes
    #[serde(flatten)]
    other_meta: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ObjectStoreType {
    Bluestore,
    Filestore,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged, rename_all = "lowercase")]
pub enum ObjectStoreMeta {
    Bluestore {
        bluefs: String,
        bluefs_db_access_mode: String,
        bluefs_db_block_size: String,
        bluefs_db_dev: Option<String>, //Not in Nautilous
        bluefs_db_dev_node: String,
        bluefs_db_driver: String,
        bluefs_db_model: Option<String>, //Not in Nautilous
        bluefs_db_partition_path: String,
        bluefs_db_rotational: String,
        bluefs_db_serial: Option<String>, //Not in Nautilous
        bluefs_db_size: String,
        bluefs_db_support_discard: Option<String>, //Nautilous
        bluefs_db_type: String,
        bluefs_single_shared_device: String,
        bluefs_slow_access_mode: Option<String>, //Not in Nautilous
        bluefs_slow_block_size: Option<String>,  //Not in Nautilous
        bluefs_slow_dev: Option<String>,         //Not in Nautilous
        bluefs_slow_dev_node: Option<String>,    //Not in Nautilous
        bluefs_slow_driver: Option<String>,      //Not in Nautilous
        bluefs_slow_model: Option<String>,       //Not in Nautilous
        bluefs_slow_partition_path: Option<String>, //Not in Nautilous
        bluefs_slow_rotational: Option<String>,  //Not in Nautilous
        bluefs_slow_size: Option<String>,        //Not in Nautilous
        bluefs_slow_type: Option<String>,        //Not in Nautilous
        bluefs_wal_access_mode: Option<String>,  //Not in Nautilous
        bluefs_wal_block_size: Option<String>,   //Not in Nautilous
        bluefs_wal_dev: Option<String>,          //Not in Nautilous
        bluefs_wal_dev_node: Option<String>,     //Not in Nautilous
        bluefs_wal_driver: Option<String>,       //Not in Nautilous
        bluefs_wal_model: Option<String>,        //Not in Nautilous
        bluefs_wal_partition_path: Option<String>, //Not in Nautilous
        bluefs_wal_rotational: Option<String>,   //Not in Nautilous
        bluefs_wal_serial: Option<String>,       //Not in Nautilous
        bluefs_wal_size: Option<String>,         //Not in Nautilous
        bluefs_wal_type: Option<String>,         //Not in Nautilous
        bluestore_bdev_access_mode: String,
        bluestore_bdev_block_size: String,
        bluestore_bdev_dev: Option<String>, //Not in Nautilous
        bluestore_bdev_dev_node: String,
        bluestore_bdev_driver: String,
        bluestore_bdev_model: Option<String>, //Not in Nautilous
        bluestore_bdev_partition_path: String,
        bluestore_bdev_rotational: String,
        bluestore_bdev_size: String,
        bluestore_bdev_support_discard: Option<String>, //Nautilous
        bluestore_bdev_type: String,
    },
    Filestore {
        backend_filestore_dev_node: String,
        backend_filestore_partition_path: String,
        filestore_backend: String,
        filestore_f_type: String,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct OsdMetadata {
    pub id: u64,
    pub arch: String,
    pub back_addr: String,
    pub back_iface: Option<String>,   //not in Jewel
    pub ceph_release: Option<String>, //Nautilous
    pub ceph_version: String,
    pub ceph_version_short: Option<String>, //Nautilous
    pub cpu: String,
    pub default_device_class: Option<String>, //not in Jewel
    pub device_ids: Option<String>,           //Nautilous
    pub devices: Option<String>,              //Nautilous
    pub distro: String,
    pub distro_description: String,
    pub distro_version: String,
    pub front_addr: String,
    pub front_iface: Option<String>, //not in Jewel
    pub hb_back_addr: String,
    pub hb_front_addr: String,
    pub hostname: String,
    pub journal_rotational: Option<String>, //not in Jewel
    pub kernel_description: String,
    pub kernel_version: String,
    pub mem_swap_kb: String,
    pub mem_total_kb: String,
    pub os: String,
    pub osd_data: String,
    pub osd_journal: Option<String>, //not usually in bluestore
    pub osd_objectstore: ObjectStoreType,
    pub rotational: Option<String>, //Not in Jewel
    #[serde(flatten)]
    pub objectstore_meta: ObjectStoreMeta,
    // other metadata not captured through the above attributes
    #[serde(flatten)]
    other_meta: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PgState {
    pub name: String,
    pub num: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PgSummary {
    pub num_pg_by_state: Vec<PgState>,
    pub num_pgs: u64,
    pub num_bytes: u64,
    pub total_bytes: Option<u64>,          //Nautilous
    pub total_avail_bytes: Option<u64>,    //Nautilous
    pub total_used_bytes: Option<u64>,     //Nautilous
    pub total_used_raw_bytes: Option<u64>, //Nautilous
    pub raw_bytes_used: Option<u64>,
    pub raw_bytes_avail: Option<u64>,
    pub raw_bytes: Option<u64>,
    pub read_bytes_sec: Option<u64>,
    pub write_bytes_sec: Option<u64>,
    pub io_sec: Option<u64>,
    pub version: Option<u64>, //Jewel
    pub degraded_objects: Option<u64>,
    pub degraded_total: Option<u64>,
    pub degraded_ratio: Option<f64>,
    pub misplaced_objects: Option<u64>,
    pub misplaced_total: Option<u64>,
    pub misplaced_ratio: Option<f64>,
    pub recovering_objects_per_sec: Option<u64>,
    pub recovering_bytes_per_sec: Option<u64>,
    pub recovering_keys_per_sec: Option<u64>,
    pub num_objects_recovered: Option<u64>,
    pub num_bytes_recovered: Option<u64>,
    pub num_keys_recovered: Option<u64>,
    // other metadata not captured through the above attributes
    #[serde(flatten)]
    other_meta: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PgStat {
    Wrapped {
        pg_ready: bool,
        pg_summary: PgSummary,
    },
    UnWrapped {
        #[serde(flatten)]
        pg_summary: PgSummary,
    },
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

#[derive(Deserialize, Debug)]
pub struct MonStatus {
    pub name: String,
    pub rank: u64,
    pub state: MonState,
    pub election_epoch: u64,
    pub quorum: Vec<u64>,
    pub outside_quorum: Vec<String>,
    pub extra_probe_peers: Vec<ExtraProbePeer>,
    pub sync_provider: Vec<u64>,
    pub monmap: MonMap,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ExtraProbePeer {
    Present { addrvec: Vec<AddrVec> },
    Absent(String),
}

#[derive(Deserialize, Debug)]
pub struct AddrVec {
    r#type: String,
    addr: String,
    nonce: i32,
}

#[derive(Deserialize, Debug)]
pub struct MonMap {
    pub epoch: u64,
    pub fsid: Uuid,
    pub modified: String,
    pub created: String,
    pub mons: Vec<Mon>,
}

#[derive(Deserialize, Debug)]
pub struct Mon {
    pub rank: u64,
    pub name: String,
    pub addr: String,
}

#[derive(Deserialize, Debug)]
pub enum HealthStatus {
    #[serde(rename = "HEALTH_ERR")]
    Err,
    #[serde(rename = "HEALTH_WARN")]
    Warn,
    #[serde(rename = "HEALTH_OK")]
    Ok,
}

#[derive(Deserialize, Debug)]
pub struct ClusterHealth {
    pub health: Health,
    pub timechecks: TimeChecks,
    pub summary: Vec<String>,
    pub overall_status: HealthStatus,
    pub detail: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Health {
    pub health_services: Vec<ServiceHealth>,
}

#[derive(Deserialize, Debug)]
pub struct TimeChecks {
    pub epoch: u64,
    pub round: u64,
    pub round_status: RoundStatus,
    pub mons: Vec<MonTimeChecks>,
}

#[derive(Deserialize, Debug)]
pub struct MonTimeChecks {
    pub name: String,
    pub skew: f64,
    pub latency: f64,
    pub health: HealthStatus,
}

#[derive(Deserialize, Debug)]
pub struct ServiceHealth {
    pub mons: Vec<MonHealth>,
}

#[derive(Deserialize, Debug)]
pub struct MonHealth {
    pub name: String,
    pub kb_total: u64,
    pub kb_used: u64,
    pub kb_avail: u64,
    pub avail_percent: u8,
    pub last_updated: String,
    pub store_stats: StoreStats,
    pub health: HealthStatus,
}

#[derive(Deserialize, Debug)]
pub struct StoreStats {
    pub bytes_total: u64,
    pub bytes_sst: u64,
    pub bytes_log: u64,
    pub bytes_misc: u64,
    pub last_updated: String,
}

#[derive(Deserialize, Debug)]
pub enum RoundStatus {
    #[serde(rename = "finished")]
    Finished,
    #[serde(rename = "on-going")]
    OnGoing,
}

#[derive(Deserialize, Debug)]
pub enum MonState {
    #[serde(rename = "probing")]
    Probing,
    #[serde(rename = "synchronizing")]
    Synchronizing,
    #[serde(rename = "electing")]
    Electing,
    #[serde(rename = "leader")]
    Leader,
    #[serde(rename = "peon")]
    Peon,
    #[serde(rename = "shutdown")]
    Shutdown,
}

#[derive(Deserialize, Debug, Serialize)]
pub enum OsdOption {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "pause")]
    Pause,
    #[serde(rename = "noup")]
    NoUp,
    #[serde(rename = "nodown")]
    NoDown,
    #[serde(rename = "noout")]
    NoOut,
    #[serde(rename = "noin")]
    NoIn,
    #[serde(rename = "nobackfill")]
    NoBackfill,
    #[serde(rename = "norebalance")]
    NoRebalance,
    #[serde(rename = "norecover")]
    NoRecover,
    #[serde(rename = "noscrub")]
    NoScrub,
    #[serde(rename = "nodeep-scrub")]
    NoDeepScrub,
    #[serde(rename = "notieragent")]
    NoTierAgent,
    #[serde(rename = "sortbitwise")]
    SortBitwise,
    #[serde(rename = "recovery_deletes")]
    RecoveryDeletes,
    #[serde(rename = "require_jewel_osds")]
    RequireJewelOsds,
    #[serde(rename = "require_kraken_osds")]
    RequireKrakenOsds,
}

impl fmt::Display for OsdOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OsdOption::Full => write!(f, "full"),
            OsdOption::Pause => write!(f, "pause"),
            OsdOption::NoUp => write!(f, "noup"),
            OsdOption::NoDown => write!(f, "nodown"),
            OsdOption::NoOut => write!(f, "noout"),
            OsdOption::NoIn => write!(f, "noin"),
            OsdOption::NoBackfill => write!(f, "nobackfill"),
            OsdOption::NoRebalance => write!(f, "norebalance"),
            OsdOption::NoRecover => write!(f, "norecover"),
            OsdOption::NoScrub => write!(f, "noscrub"),
            OsdOption::NoDeepScrub => write!(f, "nodeep-scrub"),
            OsdOption::NoTierAgent => write!(f, "notieragent"),
            OsdOption::SortBitwise => write!(f, "sortbitwise"),
            OsdOption::RecoveryDeletes => write!(f, "recovery_deletes"),
            OsdOption::RequireJewelOsds => write!(f, "require_jewel_osds"),
            OsdOption::RequireKrakenOsds => write!(f, "require_kraken_osds"),
        }
    }
}

impl AsRef<str> for OsdOption {
    fn as_ref(&self) -> &str {
        match *self {
            OsdOption::Full => "full",
            OsdOption::Pause => "pause",
            OsdOption::NoUp => "noup",
            OsdOption::NoDown => "nodown",
            OsdOption::NoOut => "noout",
            OsdOption::NoIn => "noin",
            OsdOption::NoBackfill => "nobackfill",
            OsdOption::NoRebalance => "norebalance",
            OsdOption::NoRecover => "norecover",
            OsdOption::NoScrub => "noscrub",
            OsdOption::NoDeepScrub => "nodeep-scrub",
            OsdOption::NoTierAgent => "notieragent",
            OsdOption::SortBitwise => "sortbitwise",
            OsdOption::RecoveryDeletes => "recovery_deletes",
            OsdOption::RequireJewelOsds => "require_jewel_osds",
            OsdOption::RequireKrakenOsds => "require_kraken_osds",
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub enum PoolOption {
    #[serde(rename = "size")]
    Size,
    #[serde(rename = "min_size")]
    MinSize,
    #[serde(rename = "crash_replay_interval")]
    CrashReplayInterval,
    #[serde(rename = "pg_num")]
    PgNum,
    #[serde(rename = "pgp_num")]
    PgpNum,
    #[serde(rename = "crush_rule")]
    CrushRule,
    #[serde(rename = "hashpspool")]
    HashPsPool,
    #[serde(rename = "nodelete")]
    NoDelete,
    #[serde(rename = "nopgchange")]
    NoPgChange,
    #[serde(rename = "nosizechange")]
    NoSizeChange,
    #[serde(rename = "write_fadvice_dontneed")]
    WriteFadviceDontNeed,
    #[serde(rename = "noscrub")]
    NoScrub,
    #[serde(rename = "nodeep-scrub")]
    NoDeepScrub,
    #[serde(rename = "hit_set_type")]
    HitSetType,
    #[serde(rename = "hit_set_period")]
    HitSetPeriod,
    #[serde(rename = "hit_set_count")]
    HitSetCount,
    #[serde(rename = "hit_set_fpp")]
    HitSetFpp,
    #[serde(rename = "use_gmt_hitset")]
    UseGmtHitset,
    #[serde(rename = "target_max_bytes")]
    TargetMaxBytes,
    #[serde(rename = "target_max_objects")]
    TargetMaxObjects,
    #[serde(rename = "cache_target_dirty_ratio")]
    CacheTargetDirtyRatio,
    #[serde(rename = "cache_target_dirty_high_ratio")]
    CacheTargetDirtyHighRatio,
    #[serde(rename = "cache_target_full_ratio")]
    CacheTargetFullRatio,
    #[serde(rename = "cache_min_flush_age")]
    CacheMinFlushAge,
    #[serde(rename = "cachem_min_evict_age")]
    CacheMinEvictAge,
    #[serde(rename = "auid")]
    Auid,
    #[serde(rename = "min_read_recency_for_promote")]
    MinReadRecencyForPromote,
    #[serde(rename = "min_write_recency_for_promote")]
    MinWriteRecencyForPromte,
    #[serde(rename = "fast_read")]
    FastRead,
    #[serde(rename = "hit_set_decay_rate")]
    HitSetGradeDecayRate,
    #[serde(rename = "hit_set_search_last_n")]
    HitSetSearchLastN,
    #[serde(rename = "scrub_min_interval")]
    ScrubMinInterval,
    #[serde(rename = "scrub_max_interval")]
    ScrubMaxInterval,
    #[serde(rename = "deep_scrub_interval")]
    DeepScrubInterval,
    #[serde(rename = "recovery_priority")]
    RecoveryPriority,
    #[serde(rename = "recovery_op_priority")]
    RecoveryOpPriority,
    #[serde(rename = "scrub_priority")]
    ScrubPriority,
    #[serde(rename = "compression_mode")]
    CompressionMode,
    #[serde(rename = "compression_algorithm")]
    CompressionAlgorithm,
    #[serde(rename = "compression_required_ratio")]
    CompressionRequiredRatio,
    #[serde(rename = "compression_max_blob_size")]
    CompressionMaxBlobSize,
    #[serde(rename = "compression_min_blob_size")]
    CompressionMinBlobSize,
    #[serde(rename = "csum_type")]
    CsumType,
    #[serde(rename = "csum_min_block")]
    CsumMinBlock,
    #[serde(rename = "csum_max_block")]
    CsumMaxBlock,
    #[serde(rename = "allow_ec_overwrites")]
    AllocEcOverwrites,
}

impl fmt::Display for PoolOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PoolOption::Size => write!(f, "size"),
            PoolOption::MinSize => write!(f, "min_size"),
            PoolOption::CrashReplayInterval => write!(f, "crash_replay_interval"),
            PoolOption::PgNum => write!(f, "pg_num"),
            PoolOption::PgpNum => write!(f, "pgp_num"),
            PoolOption::CrushRule => write!(f, "crush_rule"),
            PoolOption::HashPsPool => write!(f, "hashpspool"),
            PoolOption::NoDelete => write!(f, "nodelete"),
            PoolOption::NoPgChange => write!(f, "nopgchange"),
            PoolOption::NoSizeChange => write!(f, "nosizechange"),
            PoolOption::WriteFadviceDontNeed => write!(f, "write_fadvice_dontneed"),
            PoolOption::NoScrub => write!(f, "noscrub"),
            PoolOption::NoDeepScrub => write!(f, "nodeep-scrub"),
            PoolOption::HitSetType => write!(f, "hit_set_type"),
            PoolOption::HitSetPeriod => write!(f, "hit_set_period"),
            PoolOption::HitSetCount => write!(f, "hit_set_count"),
            PoolOption::HitSetFpp => write!(f, "hit_set_fpp"),
            PoolOption::UseGmtHitset => write!(f, "use_gmt_hitset"),
            PoolOption::TargetMaxBytes => write!(f, "target_max_bytes"),
            PoolOption::TargetMaxObjects => write!(f, "target_max_objects"),
            PoolOption::CacheTargetDirtyRatio => write!(f, "cache_target_dirty_ratio"),
            PoolOption::CacheTargetDirtyHighRatio => write!(f, "cache_target_dirty_high_ratio"),
            PoolOption::CacheTargetFullRatio => write!(f, "cache_target_full_ratio"),
            PoolOption::CacheMinFlushAge => write!(f, "cache_min_flush_age"),
            PoolOption::CacheMinEvictAge => write!(f, "cachem_min_evict_age"),
            PoolOption::Auid => write!(f, "auid"),
            PoolOption::MinReadRecencyForPromote => write!(f, "min_read_recency_for_promote"),
            PoolOption::MinWriteRecencyForPromte => write!(f, "min_write_recency_for_promote"),
            PoolOption::FastRead => write!(f, "fast_read"),
            PoolOption::HitSetGradeDecayRate => write!(f, "hit_set_decay_rate"),
            PoolOption::HitSetSearchLastN => write!(f, "hit_set_search_last_n"),
            PoolOption::ScrubMinInterval => write!(f, "scrub_min_interval"),
            PoolOption::ScrubMaxInterval => write!(f, "scrub_max_interval"),
            PoolOption::DeepScrubInterval => write!(f, "deep_scrub_interval"),
            PoolOption::RecoveryPriority => write!(f, "recovery_priority"),
            PoolOption::RecoveryOpPriority => write!(f, "recovery_op_priority"),
            PoolOption::ScrubPriority => write!(f, "scrub_priority"),
            PoolOption::CompressionMode => write!(f, "compression_mode"),
            PoolOption::CompressionAlgorithm => write!(f, "compression_algorithm"),
            PoolOption::CompressionRequiredRatio => write!(f, "compression_required_ratio"),
            PoolOption::CompressionMaxBlobSize => write!(f, "compression_max_blob_size"),
            PoolOption::CompressionMinBlobSize => write!(f, "compression_min_blob_size"),
            PoolOption::CsumType => write!(f, "csum_type"),
            PoolOption::CsumMinBlock => write!(f, "csum_min_block"),
            PoolOption::CsumMaxBlock => write!(f, "csum_max_block"),
            PoolOption::AllocEcOverwrites => write!(f, "allow_ec_overwrites"),
        }
    }
}

impl AsRef<str> for PoolOption {
    fn as_ref(&self) -> &str {
        match *self {
            PoolOption::Size => "size",
            PoolOption::MinSize => "min_size",
            PoolOption::CrashReplayInterval => "crash_replay_interval",
            PoolOption::PgNum => "pg_num",
            PoolOption::PgpNum => "pgp_num",
            PoolOption::CrushRule => "crush_rule",
            PoolOption::HashPsPool => "hashpspool",
            PoolOption::NoDelete => "nodelete",
            PoolOption::NoPgChange => "nopgchange",
            PoolOption::NoSizeChange => "nosizechange",
            PoolOption::WriteFadviceDontNeed => "write_fadvice_dontneed",
            PoolOption::NoScrub => "noscrub",
            PoolOption::NoDeepScrub => "nodeep-scrub",
            PoolOption::HitSetType => "hit_set_type",
            PoolOption::HitSetPeriod => "hit_set_period",
            PoolOption::HitSetCount => "hit_set_count",
            PoolOption::HitSetFpp => "hit_set_fpp",
            PoolOption::UseGmtHitset => "use_gmt_hitset",
            PoolOption::TargetMaxBytes => "target_max_bytes",
            PoolOption::TargetMaxObjects => "target_max_objects",
            PoolOption::CacheTargetDirtyRatio => "cache_target_dirty_ratio",
            PoolOption::CacheTargetDirtyHighRatio => "cache_target_dirty_high_ratio",
            PoolOption::CacheTargetFullRatio => "cache_target_full_ratio",
            PoolOption::CacheMinFlushAge => "cache_min_flush_age",
            PoolOption::CacheMinEvictAge => "cachem_min_evict_age",
            PoolOption::Auid => "auid",
            PoolOption::MinReadRecencyForPromote => "min_read_recency_for_promote",
            PoolOption::MinWriteRecencyForPromte => "min_write_recency_for_promote",
            PoolOption::FastRead => "fast_read",
            PoolOption::HitSetGradeDecayRate => "hit_set_decay_rate",
            PoolOption::HitSetSearchLastN => "hit_set_search_last_n",
            PoolOption::ScrubMinInterval => "scrub_min_interval",
            PoolOption::ScrubMaxInterval => "scrub_max_interval",
            PoolOption::DeepScrubInterval => "deep_scrub_interval",
            PoolOption::RecoveryPriority => "recovery_priority",
            PoolOption::RecoveryOpPriority => "recovery_op_priority",
            PoolOption::ScrubPriority => "scrub_priority",
            PoolOption::CompressionMode => "compression_mode",
            PoolOption::CompressionAlgorithm => "compression_algorithm",
            PoolOption::CompressionRequiredRatio => "compression_required_ratio",
            PoolOption::CompressionMaxBlobSize => "compression_max_blob_size",
            PoolOption::CompressionMinBlobSize => "compression_min_blob_size",
            PoolOption::CsumType => "csum_type",
            PoolOption::CsumMinBlock => "csum_min_block",
            PoolOption::CsumMaxBlock => "csum_max_block",
            PoolOption::AllocEcOverwrites => "allow_ec_overwrites",
        }
    }
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HealthStatus::Err => write!(f, "HEALTH_ERR"),
            HealthStatus::Ok => write!(f, "HEALTH_OK"),
            HealthStatus::Warn => write!(f, "HEALTH_WARN"),
        }
    }
}

impl AsRef<str> for HealthStatus {
    fn as_ref(&self) -> &str {
        match *self {
            HealthStatus::Err => "HEALTH_ERR",
            HealthStatus::Ok => "HEALTH_OK",
            HealthStatus::Warn => "HEALTH_WARN",
        }
    }
}

impl fmt::Display for MonState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MonState::Probing => write!(f, "probing"),
            MonState::Synchronizing => write!(f, "synchronizing"),
            MonState::Electing => write!(f, "electing"),
            MonState::Leader => write!(f, "leader"),
            MonState::Peon => write!(f, "peon"),
            MonState::Shutdown => write!(f, "shutdown"),
        }
    }
}

impl AsRef<str> for MonState {
    fn as_ref(&self) -> &str {
        match *self {
            MonState::Probing => "probing",
            MonState::Synchronizing => "synchronizing",
            MonState::Electing => "electing",
            MonState::Leader => "leader",
            MonState::Peon => "peon",
            MonState::Shutdown => "shutdown",
        }
    }
}

impl fmt::Display for RoundStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RoundStatus::Finished => write!(f, "finished"),
            RoundStatus::OnGoing => write!(f, "on-going"),
        }
    }
}

impl AsRef<str> for RoundStatus {
    fn as_ref(&self) -> &str {
        match *self {
            RoundStatus::Finished => "finished",
            RoundStatus::OnGoing => "on-going",
        }
    }
}

pub fn cluster_health(cluster_handle: &Rados) -> RadosResult<ClusterHealth> {
    let cmd = json!({
        "prefix": "health",
        "format": "json"
    });
    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

/// Check with the monitor whether a given key exists
pub fn config_key_exists(cluster_handle: &Rados, key: &str) -> RadosResult<bool> {
    let cmd = json!({
        "prefix": "config-key exists",
        "key": key,
    });

    let result = match cluster_handle.ceph_mon_command_without_data(&cmd) {
        Ok(data) => data,
        Err(e) => {
            match e {
                RadosError::Error(e) => {
                    // Ceph returns ENOENT here but RadosError masks that
                    // by turning it into a string first
                    if e.contains("doesn't exist") {
                        return Ok(false);
                    } else {
                        return Err(RadosError::Error(e));
                    }
                }
                _ => return Err(e),
            }
        }
    };
    // I don't know why but config-key exists uses the status message
    // and not the regular output buffer
    match result.1 {
        Some(status) => {
            if status.contains("exists") {
                Ok(true)
            } else {
                Err(RadosError::Error(format!(
                    "Unable to parse config-key exists output: {}",
                    status,
                )))
            }
        }
        None => Err(RadosError::Error(format!(
            "Unable to parse config-key exists output: {:?}",
            result.1,
        ))),
    }
}

/// Ask the monitor for the value of the configuration key
pub fn config_key_get(cluster_handle: &Rados, key: &str) -> RadosResult<String> {
    let cmd = json!({
        "prefix": "config-key get",
        "key": key,
    });

    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    let mut l = return_data.lines();
    match l.next() {
        Some(val) => Ok(val.to_string()),
        None => Err(RadosError::Error(format!(
            "Unable to parse config-key get output: {:?}",
            return_data,
        ))),
    }
}

/// Remove a given configuration key from the monitor cluster
pub fn config_key_remove(cluster_handle: &Rados, key: &str, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "config-key rm",
        "key": key,
        "format": "json"
    });

    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

/// Set a given configuration key in the monitor cluster
pub fn config_key_set(
    cluster_handle: &Rados,
    key: &str,
    value: &str,
    simulate: bool,
) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "config-key set",
        "key": key,
        "val": value,
        "format": "json"
    });

    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

pub fn osd_out(cluster_handle: &Rados, osd_id: u64, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "osd out",
        "ids": [osd_id.to_string()]
    });

    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

pub fn osd_crush_remove(cluster_handle: &Rados, osd_id: u64, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "osd crush remove",
        "name": format!("osd.{}", osd_id),
    });
    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

/// Get a list of all pools in the cluster
pub fn osd_pool_ls(cluster_handle: &Rados) -> RadosResult<Vec<String>> {
    let cmd = json!({
        "prefix": "osd pool ls",
        "format": "json",
    });
    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

/// Query a ceph pool.
pub fn osd_pool_get(
    cluster_handle: &Rados,
    pool: &str,
    choice: &PoolOption,
) -> RadosResult<String> {
    let cmd = json!({
        "prefix": "osd pool get",
        "pool": pool,
        "var": choice,
    });
    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    let mut l = return_data.lines();
    match l.next() {
        Some(res) => Ok(res.into()),
        None => Err(RadosError::Error(format!(
            "Unable to parse osd pool get output: {:?}",
            return_data,
        ))),
    }
}

/// Set a pool value
pub fn osd_pool_set(
    cluster_handle: &Rados,
    pool: &str,
    key: &PoolOption,
    value: &str,
    simulate: bool,
) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "osd pool set",
        "pool": pool,
        "var": key,
        "val": value,
    });
    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

pub fn osd_set(
    cluster_handle: &Rados,
    key: &OsdOption,
    force: bool,
    simulate: bool,
) -> RadosResult<()> {
    let cmd = if force {
        json!({
            "prefix": "osd set",
            "key": key,
            "sure": "--yes-i-really-mean-it",
        })
    } else {
        json!({
            "prefix": "osd set",
            "key": key,
        })
    };
    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

pub fn osd_unset(cluster_handle: &Rados, key: &OsdOption, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "osd unset",
        "key": key,
    });
    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

pub enum CrushNodeStatus {
    Up,
    Down,
    In,
    Out,
    Destroyed,
}

impl CrushNodeStatus {
    pub fn to_string(&self) -> String {
        match self {
            CrushNodeStatus::Up => "up".to_string(),
            CrushNodeStatus::Down => "down".to_string(),
            CrushNodeStatus::In => "in".to_string(),
            CrushNodeStatus::Out => "out".to_string(),
            CrushNodeStatus::Destroyed => "destroyed".to_string(),
        }
    }
}

/// get a crush tree of all osds that have the given status
pub fn osd_tree_status(cluster_handle: &Rados, status: CrushNodeStatus) -> RadosResult<CrushTree> {
    let cmd = json!({
        "prefix": "osd tree",
        "states" : &[&status.to_string()],
        "format": "json-pretty"
    });
    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

pub fn osd_tree(cluster_handle: &Rados) -> RadosResult<CrushTree> {
    let cmd = json!({
        "prefix": "osd tree",
        "format": "json"
    });
    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

// Get cluster status
pub fn status(cluster_handle: &Rados) -> RadosResult<String> {
    let cmd = json!({
        "prefix": "status",
        "format": "json"
    });
    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    let mut l = return_data.lines();
    match l.next() {
        Some(res) => Ok(res.into()),
        None => Err(RadosError::Error(format!(
            "Unable to parse status output: {:?}",
            return_data,
        ))),
    }
}

/// List all the monitors in the cluster and their current rank
pub fn mon_dump(cluster_handle: &Rados) -> RadosResult<MonDump> {
    let cmd = json!({
        "prefix": "mon dump",
        "format": "json"
    });
    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

pub fn mon_getmap(cluster_handle: &Rados, epoch: Option<u64>) -> RadosResult<Vec<u8>> {
    let mut cmd = json!({
        "prefix": "mon getmap"
    });
    if let Some(epoch) = epoch {
        cmd["epoch"] = json!(epoch);
    }

    Ok(cluster_handle.ceph_mon_command_without_data(&cmd)?.0)
}

/// Get the mon quorum
pub fn mon_quorum(cluster_handle: &Rados) -> RadosResult<String> {
    let cmd = json!({
        "prefix": "quorum_status",
        "format": "json"
    });
    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

/// Get the mon status
pub fn mon_status(cluster_handle: &Rados) -> RadosResult<MonStatus> {
    let cmd = json!({
        "prefix": "mon_status",
    });
    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

/// Show mon daemon version
pub fn version(cluster_handle: &Rados) -> RadosResult<String> {
    let cmd = json!({
        "prefix": "version",
    });
    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    let mut l = return_data.lines();
    match l.next() {
        Some(res) => Ok(res.to_string()),
        None => Err(RadosError::Error(format!(
            "Unable to parse version output: {:?}",
            return_data,
        ))),
    }
}

pub fn osd_pool_quota_get(cluster_handle: &Rados, pool: &str) -> RadosResult<u64> {
    let cmd = json!({
        "prefix": "osd pool get-quota",
        "pool": pool
    });
    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    let mut l = return_data.lines();
    match l.next() {
        Some(res) => Ok(u64::from_str(res)?),
        None => Err(RadosError::Error(format!(
            "Unable to parse osd pool quota-get output: {:?}",
            return_data,
        ))),
    }
}

pub fn auth_del(cluster_handle: &Rados, osd_id: u64, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "auth del",
        "entity": format!("osd.{}", osd_id)
    });

    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

pub fn osd_rm(cluster_handle: &Rados, osd_id: u64, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "osd rm",
        "ids": [osd_id.to_string()]
    });

    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

pub fn osd_create(cluster_handle: &Rados, id: Option<u64>, simulate: bool) -> RadosResult<u64> {
    let cmd = match id {
        Some(osd_id) => json!({
            "prefix": "osd create",
            "id": format!("osd.{}", osd_id),
        }),
        None => json!({
            "prefix": "osd create"
        }),
    };

    if simulate {
        return Ok(0);
    }

    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    let mut l = return_data.lines();
    match l.next() {
        Some(num) => Ok(u64::from_str(num)?),
        None => Err(RadosError::Error(format!(
            "Unable to parse osd create output: {:?}",
            return_data,
        ))),
    }
}

// Add a new mgr to the cluster
pub fn mgr_auth_add(cluster_handle: &Rados, mgr_id: &str, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "auth add",
        "entity": format!("mgr.{}", mgr_id),
        "caps": ["mon", "allow profile mgr", "osd", "allow *", "mds", "allow *"],
    });

    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

// Add a new osd to the cluster
pub fn osd_auth_add(cluster_handle: &Rados, osd_id: u64, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "auth add",
        "entity": format!("osd.{}", osd_id),
        "caps": ["mon", "allow rwx", "osd", "allow *"],
    });

    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

/// Get a ceph-x key.  The id parameter can be either a number or a string
/// depending on the type of client so I went with string.
pub fn auth_get_key(cluster_handle: &Rados, client_type: &str, id: &str) -> RadosResult<String> {
    let cmd = json!({
        "prefix": "auth get-key",
        "entity": format!("{}.{}", client_type, id),
    });

    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    let mut l = return_data.lines();
    match l.next() {
        Some(key) => Ok(key.into()),
        None => Err(RadosError::Error(format!(
            "Unable to parse auth get-key: {:?}",
            return_data,
        ))),
    }
}

// ceph osd crush add {id-or-name} {weight}  [{bucket-type}={bucket-name} ...]
/// add or update crushmap position and weight for an osd
pub fn osd_crush_add(
    cluster_handle: &Rados,
    osd_id: u64,
    weight: f64,
    host: &str,
    simulate: bool,
) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "osd crush add",
        "id": osd_id,
        "weight": weight,
        "args": [format!("host={}", host)]
    });

    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

// Luminous mgr commands below

/// dump the latest MgrMap
pub fn mgr_dump(cluster_handle: &Rados) -> RadosResult<MgrDump> {
    let cmd = json!({
        "prefix": "mgr dump",
    });

    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

/// Treat the named manager daemon as failed
pub fn mgr_fail(cluster_handle: &Rados, mgr_id: &str, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "mgr fail",
        "name": mgr_id,
    });

    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

/// List active mgr modules
pub fn mgr_list_modules(cluster_handle: &Rados) -> RadosResult<Vec<String>> {
    let cmd = json!({
        "prefix": "mgr module ls",
    });

    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

/// List service endpoints provided by mgr modules
pub fn mgr_list_services(cluster_handle: &Rados) -> RadosResult<Vec<String>> {
    let cmd = json!({
        "prefix": "mgr services",
    });

    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

/// Enable a mgr module
pub fn mgr_enable_module(
    cluster_handle: &Rados,
    module: &str,
    force: bool,
    simulate: bool,
) -> RadosResult<()> {
    let cmd = if force {
        json!({
            "prefix": "mgr module enable",
            "module": module,
            "force": "--force",
        })
    } else {
        json!({
            "prefix": "mgr module enable",
            "module": module,
        })
    };

    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

/// Disable a mgr module
pub fn mgr_disable_module(cluster_handle: &Rados, module: &str, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "mgr module disable",
        "module": module,
    });

    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

/// dump metadata for all daemons.  Note this only works for Luminous+
pub fn mgr_metadata(cluster_handle: &Rados) -> RadosResult<Vec<MgrMetadata>> {
    let vrsn: CephVersion = version(cluster_handle)?.parse()?;
    if vrsn < CephVersion::Luminous {
        return Err(RadosError::MinVersion(CephVersion::Luminous, vrsn));
    }
    let cmd = json!({
        "prefix": "mgr metadata",
    });

    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

/// dump metadata for all osds
pub fn osd_metadata(cluster_handle: &Rados) -> RadosResult<Vec<OsdMetadata>> {
    let cmd = json!({
        "prefix": "osd metadata",
    });

    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

/// get osd metadata for a specific osd id
pub fn osd_metadata_by_id(cluster_handle: &Rados, osd_id: u64) -> RadosResult<OsdMetadata> {
    let cmd = json!({
        "prefix": "osd metadata",
        "id": osd_id,
    });

    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    trace!("{:?}", return_data);
    Ok(serde_json::from_str(&return_data)?)
}

/// reweight an osd in the CRUSH map
pub fn osd_crush_reweight(
    cluster_handle: &Rados,
    osd_id: u64,
    weight: f64,
    simulate: bool,
) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "osd crush reweight",
        "name":  format!("osd.{}", osd_id),
        "weight": weight,
    });
    if !simulate {
        cluster_handle.ceph_mon_command_without_data(&cmd)?;
    }
    Ok(())
}

/// check if a single osd is safe to destroy/remove
pub fn osd_safe_to_destroy(cluster_handle: &Rados, osd_id: u64) -> bool {
    let cmd = json!({
        "prefix": "osd safe-to-destroy",
        "ids": [osd_id.to_string()]
    });
    match cluster_handle.ceph_mon_command_without_data(&cmd) {
        Err(_) => false,
        Ok(_) => true,
    }
}

/// count ceph-mgr daemons by metadata field property
pub fn mgr_count_metadata(
    cluster_handle: &Rados,
    property: &str,
) -> RadosResult<HashMap<String, u64>> {
    let cmd = json!({
        "prefix": "mgr count-metadata",
        "name": property,
    });

    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

/// check running versions of ceph-mgr daemons
pub fn mgr_versions(cluster_handle: &Rados) -> RadosResult<HashMap<String, u64>> {
    let cmd = json!({
        "prefix": "mgr versions",
    });

    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}

pub fn pg_stat(cluster_handle: &Rados) -> RadosResult<PgStat> {
    let cmd = json!({ "prefix": "pg stat", "format": "json"});
    let result = cluster_handle.ceph_mon_command_without_data(&cmd)?;
    let return_data = String::from_utf8(result.0)?;
    Ok(serde_json::from_str(&return_data)?)
}
