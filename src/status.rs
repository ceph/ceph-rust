// Copyright 2017 LambdaStack All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatus {
    health: CephStatusHealth,
    fsid: String,
    election_epoch: u32,
    quorum: Vec<u32>,
    quorum_names: Vec<String>,
    monmap: CephStatusMonMap,
    osdmap: CephStatusOSDMapH,
    pgmap: CephStatusPGMap,
    mdsmap: CephStatusMDSMap,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusHealth {
    health: CephStatusHealth2,
    timechecks: CephStatusHealthTimeChecks,
    summary: Vec<CephStatusHealthSummary>,
    overall_status: String,
    detail: Vec<CephStatusHealthDetail>,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusHealth2 {
    health: Vec<CephStatusHealthServices>,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusHealthServices {
    mons: Vec<CephStatusHealthServicesMon>,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusHealthServicesMon {
    name: String,
    kb_total: u32,
    kb_used: u32,
    kb_avail: u32,
    avail_percent: u16,
    last_updated: String,
    store_stats: CephStatusHealthServicesMonStats,
    health: String,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusHealthServicesMonStats {
    bytes_total: u64,
    bytes_sst: u64,
    bytes_log: u64,
    bytes_misc: u64,
    last_updated: String,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusHealthTimeChecks {
    epoch: u32,
    round: u32,
    round_status: String,
    mons: Vec<CephStatusHealthMons>,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusHealthMons {
    name: String,
    skew: f32,
    latency: f32,
    health: String,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusHealthSummary {
    dummy: String,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusHealthDetail {
    dummy: String,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusMonMap {
    epoch: u32,
    fsid: String,
    modified: String,
    created: String,
    mons: Vec<CephStatusMonRank>,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusMonRank {
    rank: u16,
    name: String,
    addr: String,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusOSDMapH {
    osdmap: CephStatusOSDMapL,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusOSDMapL {
    epoch: u32,
    num_osds: u32,
    num_up_osds: u32,
    num_in_osds: u32,
    full: bool,
    nearfull: bool,
    num_remapped_pgs: u32,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusPGMap {
    pgs_by_state: Vec<CephStatusPGState>,
    version: u32,
    num_pgs: u32,
    data_bytes: u64,
    bytes_used: u64,
    bytes_avail: u64,
    bytes_total: u64,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusPGState {
    state_name: String,
    count: u32,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusMDSMap {
    epoch: u32,
    up: u32,
    _in: u32,
    max: u32,
    by_rank: Vec<CephStatusMDSRank>,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CephStatusMDSRank {
    rank: u16,
    name: String,
    addr: String,
}
