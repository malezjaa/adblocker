use std::time::Duration;

use hickory_proto::rr::RData;
use hickory_resolver::net::NoRecords;

use crate::cache::MAX_NEGATIVE_TTL;

pub fn negative_ttl(no: &NoRecords) -> Duration {
  let secs = no
    .authorities
    .as_ref()
    .map(|auths| {
      auths
        .iter()
        .filter_map(
          |r| if let RData::SOA(soa) = &r.data { Some(soa.minimum) } else { None },
        )
        .min()
        .unwrap_or(60)
    })
    .unwrap_or(60);
  Duration::from_secs(secs.min(MAX_NEGATIVE_TTL) as u64)
}
