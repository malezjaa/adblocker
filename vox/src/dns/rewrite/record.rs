use crate::dns::rewrite::{Rewrite, RewriteAction};
use anyhow::Error;
use hickory_proto::op::Query;
use hickory_proto::rr::rdata::{A, AAAA};
use hickory_proto::rr::{RData, RecordType};

pub fn construct_rewrite_records(
  query: &Query,
  rewrite: &Rewrite,
) -> anyhow::Result<Vec<RData>, Error> {
  use hickory_proto::rr::Name;
  use hickory_proto::rr::rdata::{CNAME, MX, PTR, SRV, TXT};

  let mut rdatas = vec![];

  for action in &rewrite.actions {
    let rdata = match (query.query_type(), action) {
      (RecordType::A, RewriteAction::A { value }) => Some(RData::A(A(value.parse()?))),
      (RecordType::AAAA, RewriteAction::AAAA { value }) => {
        Some(RData::AAAA(AAAA(value.parse()?)))
      }
      (RecordType::CNAME, RewriteAction::CNAME { value }) => {
        Some(RData::CNAME(CNAME(Name::from_str_relaxed(value)?)))
      }
      (RecordType::MX, RewriteAction::MX { exchange, preference }) => {
        Some(RData::MX(MX::new(*preference, Name::from_str_relaxed(exchange)?)))
      }
      (RecordType::TXT, RewriteAction::TXT { value }) => {
        Some(RData::TXT(TXT::new(value.clone())))
      }
      (RecordType::PTR, RewriteAction::PTR { value }) => {
        Some(RData::PTR(PTR(Name::from_str_relaxed(value)?)))
      }
      (RecordType::SRV, RewriteAction::SRV { priority, weight, port, target }) => Some(
        RData::SRV(SRV::new(*priority, *weight, *port, Name::from_str_relaxed(target)?)),
      ),
      _ => None,
    };

    if let Some(rdata) = rdata {
      rdatas.push(rdata);
    }
  }

  Ok(rdatas)
}
