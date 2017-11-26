
use std::fmt::{Result, Display, Formatter};
use CephChoices::*;
/// "name=key,type=CephChoices,strings=full|pause|noup|nodown|noout|noin|nobackfill|norebalance|norecover|noscrub|nodeep-scrub|notieragent|sortbitwise|recovery_deletes|require_jewel_osds|require_kraken_osds " \
/// Taken from src/mon/MonCommands.h in the ceph github repo
pub enum CephChoices {
    Full,
    Pause,
    NoUp,
    NoDown,
    NoOut,
    NoIn,
    NoBackfill,
    NoRebalance,
    NoRecover,
    NoScrub,
    NoDeepScrub,
    NoTierAgent,
    SortBitwise,
    RecoveryDeletes,
    RequireJewelOsds,
    RequireKrakenOsds,
}
impl Display for CephChoices {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}",
            match *self {
                Full => "full",
                Pause => "pause",
                NoUp => "noup",
                NoDown => "nodown",
                NoOut => "noout",
                NoIn => "noin",
                NoBackfill => "nobackfill",
                NoRebalance => "norebalance",
                NoRecover => "norecover",
                NoScrub => "noscrub",
                NoDeepScrub => "scrub",
                NoTierAgent => "notieragent",
                SortBitwise => "sortbitwise",
                RecoveryDeletes => "recovery_deletes",
                RequireJewelOsds => "require_jewel_osds",
                RequireKrakenOsds => "require_kraken_osds",
            }
        )
    }
}

impl AsRef<str> for CephChoices {
    fn as_ref(&self) -> &'static str {
        match *self {
            Full => "full",
            Pause => "pause",
            NoUp => "noup",
            NoDown => "nodown",
            NoOut => "noout",
            NoIn => "noin",
            NoBackfill => "nobackfill",
            NoRebalance => "norebalance",
            NoRecover => "norecover",
            NoScrub => "noscrub",
            NoDeepScrub => "scrub",
            NoTierAgent => "notieragent",
            SortBitwise => "sortbitwise",
            RecoveryDeletes => "recovery_deletes",
            RequireJewelOsds => "require_jewel_osds",
            RequireKrakenOsds => "require_kraken_osds",
        }
    }
}
