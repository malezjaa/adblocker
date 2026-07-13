use anyhow::{Result, bail};
use hickory_proto::{
  op::Query,
  rr::{
    Name, RData, Record, RecordType,
    rdata::{A, AAAA, CNAME, HTTPS, MX, PTR, SRV, SVCB, TXT},
  },
};
use vox_shared::config::rewrite::{RewriteRecord, RewriteRecordType, RewriteRecordValue};

pub const DEFAULT_REWRITE_TTL: u32 = 300;

pub fn construct_structured_rewrite_records(
  query: &Query,
  records: &[RewriteRecord],
  default_ttl: u32,
) -> Result<Vec<Record>> {
  let mut answers = Vec::new();

  for record in records {
    let value_type = record_value_type(&record.value);
    if record.ty != value_type {
      bail!(
        "rewrite record type {:?} does not match value type {:?}",
        record.ty,
        value_type
      );
    }

    let record_type = rewrite_record_type_to_hickory(record.ty);
    if !record_matches_query(record_type, query.query_type()) {
      continue;
    }

    answers.push(Record::from_rdata(
      query.name().clone(),
      record.ttl.unwrap_or(default_ttl),
      record_value_to_rdata(&record.value)?,
    ));
  }

  Ok(answers)
}

pub fn construct_alias_record(query: &Query, target: &str, ttl: u32) -> Result<Record> {
  Ok(Record::from_rdata(
    query.name().clone(),
    ttl,
    RData::CNAME(CNAME(Name::from_str_relaxed(target)?)),
  ))
}

pub fn rewrite_record_type_to_hickory(record_type: RewriteRecordType) -> RecordType {
  match record_type {
    RewriteRecordType::A => RecordType::A,
    RewriteRecordType::AAAA => RecordType::AAAA,
    RewriteRecordType::CNAME => RecordType::CNAME,
    RewriteRecordType::MX => RecordType::MX,
    RewriteRecordType::TXT => RecordType::TXT,
    RewriteRecordType::PTR => RecordType::PTR,
    RewriteRecordType::SRV => RecordType::SRV,
    RewriteRecordType::HTTPS => RecordType::HTTPS,
    RewriteRecordType::SVCB => RecordType::SVCB,
  }
}

fn record_matches_query(record_type: RecordType, query_type: RecordType) -> bool {
  query_type == RecordType::ANY
    || record_type == query_type
    || record_type == RecordType::CNAME
}

fn record_value_type(value: &RewriteRecordValue) -> RewriteRecordType {
  match value {
    RewriteRecordValue::A { .. } => RewriteRecordType::A,
    RewriteRecordValue::AAAA { .. } => RewriteRecordType::AAAA,
    RewriteRecordValue::CNAME { .. } => RewriteRecordType::CNAME,
    RewriteRecordValue::MX { .. } => RewriteRecordType::MX,
    RewriteRecordValue::TXT { .. } => RewriteRecordType::TXT,
    RewriteRecordValue::PTR { .. } => RewriteRecordType::PTR,
    RewriteRecordValue::SRV { .. } => RewriteRecordType::SRV,
    RewriteRecordValue::HTTPS { .. } => RewriteRecordType::HTTPS,
    RewriteRecordValue::SVCB { .. } => RewriteRecordType::SVCB,
  }
}

fn record_value_to_rdata(value: &RewriteRecordValue) -> Result<RData> {
  Ok(match value {
    RewriteRecordValue::A { value } => RData::A(A(value.parse()?)),
    RewriteRecordValue::AAAA { value } => RData::AAAA(AAAA(value.parse()?)),
    RewriteRecordValue::CNAME { value } => {
      RData::CNAME(CNAME(Name::from_str_relaxed(value)?))
    }
    RewriteRecordValue::MX { exchange, preference } => {
      RData::MX(MX::new(*preference, Name::from_str_relaxed(exchange)?))
    }
    RewriteRecordValue::TXT { value } => RData::TXT(TXT::new(value.clone())),
    RewriteRecordValue::PTR { value } => RData::PTR(PTR(Name::from_str_relaxed(value)?)),
    RewriteRecordValue::SRV { priority, weight, port, target } => {
      RData::SRV(SRV::new(*priority, *weight, *port, Name::from_str_relaxed(target)?))
    }
    RewriteRecordValue::HTTPS { priority, target, params } => {
      RData::HTTPS(HTTPS(build_svcb(*priority, target, params)?))
    }
    RewriteRecordValue::SVCB { priority, target, params } => {
      RData::SVCB(build_svcb(*priority, target, params)?)
    }
  })
}

fn build_svcb(priority: u16, target: &str, params: &[String]) -> Result<SVCB> {
  if !params.is_empty() {
    bail!("SVCB/HTTPS rewrite params are not supported yet");
  }

  Ok(SVCB::new(priority, Name::from_str_relaxed(target)?, Vec::new()))
}

#[cfg(test)]
mod tests {
  use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
  };

  use hickory_proto::{
    op::Query,
    rr::rdata::{A, AAAA},
  };

  use super::*;

  fn query(record_type: RecordType) -> Query {
    Query::query(Name::from_str("service.test.").unwrap(), record_type)
  }

  fn rewrite_record(
    ty: RewriteRecordType,
    value: RewriteRecordValue,
    ttl: Option<u32>,
  ) -> RewriteRecord {
    RewriteRecord { ty, value, ttl }
  }

  #[test]
  fn query_type_filters_records_but_keeps_cname_answers() {
    let records = vec![
      rewrite_record(
        RewriteRecordType::A,
        RewriteRecordValue::A { value: "10.0.0.2".into() },
        Some(30),
      ),
      rewrite_record(
        RewriteRecordType::AAAA,
        RewriteRecordValue::AAAA { value: "2001:db8::2".into() },
        None,
      ),
      rewrite_record(
        RewriteRecordType::CNAME,
        RewriteRecordValue::CNAME { value: "target.test.".into() },
        Some(45),
      ),
    ];

    let answers =
      construct_structured_rewrite_records(&query(RecordType::A), &records, 300).unwrap();

    assert_eq!(answers.len(), 2);
    assert_eq!(answers[0].record_type(), RecordType::A);
    assert_eq!(answers[0].ttl, 30);
    assert_eq!(answers[0].data, RData::A(A(Ipv4Addr::new(10, 0, 0, 2))));
    assert_eq!(answers[1].record_type(), RecordType::CNAME);
    assert_eq!(answers[1].ttl, 45);
  }

  #[test]
  fn any_query_returns_records_using_default_ttl_when_unset() {
    let records = vec![
      rewrite_record(
        RewriteRecordType::A,
        RewriteRecordValue::A { value: "10.0.0.2".into() },
        None,
      ),
      rewrite_record(
        RewriteRecordType::AAAA,
        RewriteRecordValue::AAAA { value: "2001:db8::2".into() },
        None,
      ),
    ];

    let answers =
      construct_structured_rewrite_records(&query(RecordType::ANY), &records, 120)
        .unwrap();

    assert_eq!(answers.len(), 2);
    assert_eq!(answers[0].ttl, 120);
    assert_eq!(answers[1].ttl, 120);
    assert_eq!(
      answers[1].data,
      RData::AAAA(AAAA(Ipv6Addr::from_str("2001:db8::2").unwrap()))
    );
  }

  #[test]
  fn mismatched_record_type_and_value_is_rejected() {
    let records = vec![rewrite_record(
      RewriteRecordType::A,
      RewriteRecordValue::AAAA { value: "2001:db8::2".into() },
      None,
    )];

    let err = construct_structured_rewrite_records(&query(RecordType::A), &records, 300)
      .unwrap_err();

    assert!(
      err.to_string().contains("rewrite record type A does not match value type AAAA")
    );
  }

  #[test]
  fn svcb_params_are_rejected_until_supported() {
    let records = vec![rewrite_record(
      RewriteRecordType::HTTPS,
      RewriteRecordValue::HTTPS {
        priority: 1,
        target: "svc.test.".into(),
        params: vec!["alpn=h2".into()],
      },
      None,
    )];

    let err =
      construct_structured_rewrite_records(&query(RecordType::HTTPS), &records, 300)
        .unwrap_err();

    assert!(err.to_string().contains("params are not supported yet"));
  }
}
