use std::str::FromStr;

use crate::error::RadosError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_compares() {
        assert!(CephVersion::Argonaut < CephVersion::Bobtail);
        assert!(CephVersion::Luminous > CephVersion::Jewel);
    }

    #[test]
    fn it_parses_jewel() {
        let version: CephVersion = "ceph version 10.2.9 (2ee413f77150c0f375ff6f10edd6c8f9c7d060d0)"
            .parse()
            .unwrap();
        assert_eq!(version, CephVersion::Jewel);
    }
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CephVersion {
    Argonaut,
    Bobtail,
    Cuttlefish,
    Dumpling,
    Emperor,
    Firefly,
    Giant,
    Hammer,
    Infernalis,
    Jewel,
    Kraken,
    Luminous,
    Mimic,
    Nautilus,
    Octopus,
    Pacific,
}

impl FromStr for CephVersion {
    type Err = RadosError;

    /// Expects an input in the form that the `ceph --version` command, or the
    /// rados version commands give them:
    /// `ceph version 10.2.9 (2ee413f77150c0f375ff6f10edd6c8f9c7d060d0)`
    fn from_str(s: &str) -> Result<Self, RadosError> {
        use crate::CephVersion::*;
        let mut parts = s.split(' ');
        if let (Some(_ceph), Some(_version), Some(version_str)) =
            (parts.next(), parts.next(), parts.next())
        {
            let mut version_parts = version_str.split('.');
            if let (Some(major), Some(minor), Some(_patch)) = (
                version_parts.next(),
                version_parts.next(),
                version_parts.next(),
            ) {
                match major {
                    "16" => return Ok(Pacific),
                    "15" => return Ok(Octopus),
                    "14" => return Ok(Nautilus),
                    "13" => return Ok(Mimic),
                    "12" => return Ok(Luminous),
                    "11" => return Ok(Kraken),
                    "10" => return Ok(Jewel),
                    "9" => return Ok(Infernalis),
                    "0" => match minor {
                        "94" => return Ok(Hammer),
                        "97" => return Ok(Giant),
                        "80" => return Ok(Firefly),
                        "72" => return Ok(Emperor),
                        "67" => return Ok(Dumpling),
                        "61" => return Ok(Cuttlefish),
                        "56" => return Ok(Bobtail),
                        "48" => return Ok(Argonaut),
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        Err(RadosError::Parse(s.into()))
    }
}
