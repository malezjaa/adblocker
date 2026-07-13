use std::str::FromStr;

use anyhow::{Result, bail};
use hickory_proto::{
  op::{Message, Query, ResponseCode},
  rr::{Name, RecordType},
};
use vox_shared::config::rewrite::{
  Rewrite, RewriteBehavior, RewriteClientCondition, RewriteConditions,
  RewriteMatchWhenType, RewriteTransportCondition,
};

use crate::{
  block_origin::{BlockOrigin, ClientOrigin, TransportOrigin},
  rewrite::record::{
    DEFAULT_REWRITE_TTL, construct_alias_record, construct_structured_rewrite_records,
    rewrite_record_type_to_hickory,
  },
};

pub struct RewriteResult {
  pub synthetic_response: bool,
  pub restore_original_queries: bool,
  pub skip_block_lookup: bool,
  pub rewritten_names: Vec<(Name, Name)>,
}

impl RewriteResult {
  fn passthrough() -> Self {
    Self {
      synthetic_response: false,
      restore_original_queries: false,
      skip_block_lookup: false,
      rewritten_names: Vec::new(),
    }
  }

  fn apply_outcome(&mut self, outcome: RewriteOutcome) {
    self.synthetic_response |= outcome.synthetic_response;
    self.skip_block_lookup |= outcome.skip_block_lookup;

    if let Some((rewritten, original)) = outcome.rewritten_name {
      self.restore_original_queries = true;
      self.rewritten_names.push((rewritten, original));
    }
  }
}

#[derive(Clone, Copy, Default)]
pub struct RewriteContext<'a> {
  pub origin: Option<BlockOrigin>,
  pub device: Option<&'a str>,
  pub device_name: Option<&'a str>,
}

#[derive(Debug)]
struct RewriteOutcome {
  synthetic_response: bool,
  skip_block_lookup: bool,
  rewritten_name: Option<(Name, Name)>,
  terminal_response: bool,
}

impl RewriteOutcome {
  fn passthrough() -> Self {
    Self {
      synthetic_response: false,
      skip_block_lookup: false,
      rewritten_name: None,
      terminal_response: false,
    }
  }

  fn synthetic(skip_block_lookup: bool) -> Self {
    Self { synthetic_response: true, skip_block_lookup, ..Self::passthrough() }
  }

  fn terminal() -> Self {
    Self {
      synthetic_response: true,
      skip_block_lookup: true,
      terminal_response: true,
      rewritten_name: None,
    }
  }

  fn forward(rewritten: Name, original: Name) -> Self {
    Self {
      synthetic_response: false,
      skip_block_lookup: false,
      rewritten_name: Some((rewritten, original)),
      terminal_response: false,
    }
  }
}

pub fn apply_rewrites(
  rewrites: Option<&[Rewrite]>,
  msg: &mut Message,
) -> Result<RewriteResult> {
  apply_rewrites_with_context(rewrites, msg, RewriteContext::default())
}

pub fn apply_rewrites_with_context(
  rewrites: Option<&[Rewrite]>,
  msg: &mut Message,
  context: RewriteContext<'_>,
) -> Result<RewriteResult> {
  let Some(rewrites) = rewrites else {
    return Ok(RewriteResult::passthrough());
  };

  let original_queries = msg.queries.clone();
  let mut rewritten_queries = original_queries.clone();
  let mut result = RewriteResult::passthrough();

  let mut ordered_rewrites: Vec<_> = rewrites.iter().enumerate().collect();
  ordered_rewrites.sort_by_key(|(index, rewrite)| (rewrite.priority, *index));

  for (query_index, original_query) in original_queries.iter().enumerate() {
    let host = normalize_domain(&original_query.name().to_string());

    for (_, rewrite) in &ordered_rewrites {
      if !rewrite.enabled
        || !conditions_match(&rewrite.conditions, original_query, context)
        || !rewrite_matches(rewrite, &host)?
      {
        continue;
      }

      let outcome = execute_behavior(
        rewrite,
        original_query,
        &mut rewritten_queries[query_index],
        msg,
      )?;
      let terminal_response = outcome.terminal_response;
      result.apply_outcome(outcome);

      if result.restore_original_queries {
        msg.queries = rewritten_queries.clone();
      }

      if terminal_response {
        return Ok(result);
      }

      if !rewrite.continue_processing {
        break;
      }
    }
  }

  if result.restore_original_queries {
    msg.queries = rewritten_queries;
  }

  Ok(result)
}

fn execute_behavior(
  rewrite: &Rewrite,
  original_query: &Query,
  rewritten_query: &mut Query,
  msg: &mut Message,
) -> Result<RewriteOutcome> {
  match &rewrite.behavior {
    RewriteBehavior::Respond { records, ttl } => {
      let default_ttl = ttl.or(rewrite.ttl).unwrap_or(DEFAULT_REWRITE_TTL);
      msg.answers.extend(construct_structured_rewrite_records(
        original_query,
        records,
        default_ttl,
      )?);
      Ok(RewriteOutcome::synthetic(true))
    }
    RewriteBehavior::Alias { target, ttl } => {
      msg.answers.push(construct_alias_record(
        original_query,
        target,
        ttl.or(rewrite.ttl).unwrap_or(DEFAULT_REWRITE_TTL),
      )?);
      Ok(RewriteOutcome::synthetic(true))
    }
    RewriteBehavior::Forward { target } => {
      let target = Name::from_str(target)?;
      *rewritten_query = Query::query(target.clone(), original_query.query_type());
      Ok(RewriteOutcome::forward(target, original_query.name().clone()))
    }
    RewriteBehavior::NxDomain => {
      msg.answers.clear();
      msg.metadata.response_code = ResponseCode::NXDomain;
      Ok(RewriteOutcome::terminal())
    }
    RewriteBehavior::NoData => {
      msg.answers.clear();
      msg.metadata.response_code = ResponseCode::NoError;
      Ok(RewriteOutcome::terminal())
    }
  }
}

fn conditions_match(
  conditions: &RewriteConditions,
  query: &Query,
  context: RewriteContext<'_>,
) -> bool {
  if !conditions.query_types.is_empty()
    && !conditions.query_types.iter().any(|ty| {
      query.query_type() == RecordType::ANY
        || rewrite_record_type_to_hickory(*ty) == query.query_type()
    })
  {
    return false;
  }

  if !conditions.devices.is_empty() {
    let Some(device) = context.device else {
      return false;
    };

    if !conditions.devices.iter().any(|expected| {
      expected.eq_ignore_ascii_case(device)
        || context.device_name.is_some_and(|name| expected.eq_ignore_ascii_case(name))
    }) {
      return false;
    }
  }

  if !conditions.transports.is_empty() {
    let Some(origin) = context.origin else {
      return false;
    };

    if !conditions
      .transports
      .iter()
      .any(|expected| transport_matches(origin_transport(origin), *expected))
    {
      return false;
    }
  }

  if !conditions.client_origins.is_empty() {
    let Some(origin) = context.origin else {
      return false;
    };

    let Some(client) = origin_client(origin) else {
      return false;
    };

    if !conditions.client_origins.iter().any(|expected| client_matches(client, *expected))
    {
      return false;
    }
  }

  true
}

fn transport_matches(
  actual: TransportOrigin,
  expected: RewriteTransportCondition,
) -> bool {
  matches!(
    (actual, expected),
    (TransportOrigin::Plain, RewriteTransportCondition::Plain)
      | (TransportOrigin::DoH, RewriteTransportCondition::DoH)
      | (TransportOrigin::DoT, RewriteTransportCondition::DoT)
      | (TransportOrigin::DoQ, RewriteTransportCondition::DoQ)
  )
}

fn client_matches(actual: ClientOrigin, expected: RewriteClientCondition) -> bool {
  matches!(
    (actual, expected),
    (ClientOrigin::Windows, RewriteClientCondition::Windows)
      | (ClientOrigin::Linux, RewriteClientCondition::Linux)
      | (ClientOrigin::Mac, RewriteClientCondition::Mac)
  )
}

fn origin_transport(origin: BlockOrigin) -> TransportOrigin {
  match origin {
    BlockOrigin::Transport(transport) => transport,
    BlockOrigin::Client { transport, .. } => transport,
  }
}

fn origin_client(origin: BlockOrigin) -> Option<ClientOrigin> {
  match origin {
    BlockOrigin::Transport(_) => None,
    BlockOrigin::Client { client, .. } => Some(client),
  }
}

fn rewrite_matches(rewrite: &Rewrite, host: &str) -> Result<bool> {
  let value = normalize_domain(&rewrite.when.value);

  Ok(match rewrite.when.ty {
    RewriteMatchWhenType::Exact => value == host,
    RewriteMatchWhenType::Suffix => host == value || host.ends_with(&format!(".{value}")),
    RewriteMatchWhenType::Wildcard => wildcard_matches(&value, host),
    RewriteMatchWhenType::Regex => {
      let Some(regex) = rewrite.regex.as_ref() else {
        bail!("regex rewrite '{}' was not compiled", rewrite.when.value);
      };

      regex.is_match(host)
    }
  })
}

fn normalize_domain(value: &str) -> String {
  value.trim().trim_end_matches('.').to_ascii_lowercase()
}

fn wildcard_matches(pattern: &str, value: &str) -> bool {
  let pattern = pattern.as_bytes();
  let value = value.as_bytes();
  let (mut pattern_index, mut value_index) = (0, 0);
  let mut star_index = None;
  let mut star_value_index = 0;

  while value_index < value.len() {
    if pattern_index < pattern.len()
      && (pattern[pattern_index] == b'?' || pattern[pattern_index] == value[value_index])
    {
      pattern_index += 1;
      value_index += 1;
    } else if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
      star_index = Some(pattern_index);
      star_value_index = value_index;
      pattern_index += 1;
    } else if let Some(index) = star_index {
      pattern_index = index + 1;
      star_value_index += 1;
      value_index = star_value_index;
    } else {
      return false;
    }
  }

  while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
    pattern_index += 1;
  }

  pattern_index == pattern.len()
}

pub fn restore_original_queries(
  response: &mut Message,
  original_queries: &[Query],
  rewritten_names: &[(Name, Name)],
) {
  response.queries = original_queries.to_vec();

  for answer in &mut response.answers {
    let mut restored = false;
    for (rewritten, original) in rewritten_names {
      if answer.name == *rewritten {
        answer.name = original.clone();
        restored = true;
        break;
      }
    }

    if !restored
      && rewritten_names.len() == 1
      && original_queries.len() == 1
      && let Some((_, original)) = rewritten_names.first()
    {
      answer.name = original.clone();
    }
  }
}

#[cfg(test)]
mod tests {
  use std::net::Ipv4Addr;

  use hickory_proto::rr::{RData, Record, RecordType, rdata::A};
  use vox_shared::config::rewrite::{
    RewriteBehavior, RewriteClientCondition, RewriteMatchWhen, RewriteRecord,
    RewriteRecordType, RewriteRecordValue, RewriteTransportCondition,
  };

  use super::*;

  fn query(name: &str, record_type: RecordType) -> Message {
    let mut msg = Message::query();
    msg.add_query(Query::query(Name::from_str(name).unwrap(), record_type));
    msg
  }

  fn rewrite(match_type: RewriteMatchWhenType, value: &str) -> Rewrite {
    Rewrite {
      name: None,
      enabled: true,
      priority: 100,
      when: RewriteMatchWhen { ty: match_type, value: value.into() },
      conditions: RewriteConditions::default(),
      behavior: RewriteBehavior::NoData,
      ttl: None,
      continue_processing: false,
      regex: None,
    }
  }

  fn a_record(value: &str) -> RewriteRecord {
    RewriteRecord {
      ty: RewriteRecordType::A,
      value: RewriteRecordValue::A { value: value.into() },
      ttl: None,
    }
  }

  #[test]
  fn suffix_responds_with_static_record_and_skips_block_lookup() {
    let mut rule = rewrite(RewriteMatchWhenType::Suffix, "example.test");
    rule.behavior =
      RewriteBehavior::Respond { records: vec![a_record("10.0.0.2")], ttl: Some(30) };

    let mut msg = query("WWW.Example.Test.", RecordType::A);
    let result = apply_rewrites(Some(&[rule]), &mut msg).unwrap();

    assert!(result.synthetic_response);
    assert!(result.skip_block_lookup);
    assert_eq!(msg.answers.len(), 1);
    assert_eq!(msg.answers[0].ttl, 30);
    assert_eq!(msg.answers[0].data, RData::A(A(Ipv4Addr::new(10, 0, 0, 2))));
  }

  #[test]
  fn device_and_transport_conditions_must_match() {
    let mut rule = rewrite(RewriteMatchWhenType::Exact, "app.test");
    rule.conditions.devices = vec!["laptop".into()];
    rule.conditions.transports = vec![RewriteTransportCondition::DoH];
    rule.behavior =
      RewriteBehavior::Respond { records: vec![a_record("10.0.0.3")], ttl: None };

    let mut msg = query("app.test.", RecordType::A);
    let result = apply_rewrites_with_context(
      Some(&[rule.clone()]),
      &mut msg,
      RewriteContext {
        origin: Some(BlockOrigin::plain()),
        device: Some("laptop"),
        device_name: None,
      },
    )
    .unwrap();

    assert!(!result.synthetic_response);
    assert!(msg.answers.is_empty());

    let mut msg = query("app.test.", RecordType::A);
    let result = apply_rewrites_with_context(
      Some(&[rule]),
      &mut msg,
      RewriteContext {
        origin: Some(BlockOrigin::doh()),
        device: Some("laptop"),
        device_name: None,
      },
    )
    .unwrap();

    assert!(result.synthetic_response);
    assert_eq!(msg.answers.len(), 1);
  }

  #[test]
  fn device_conditions_accept_the_readable_device_name() {
    let mut rule = rewrite(RewriteMatchWhenType::Exact, "app.test");
    rule.conditions.devices = vec!["Living Room Laptop".into()];
    rule.behavior =
      RewriteBehavior::Respond { records: vec![a_record("10.0.0.3")], ttl: None };

    let mut msg = query("app.test.", RecordType::A);
    let result = apply_rewrites_with_context(
      Some(&[rule]),
      &mut msg,
      RewriteContext {
        origin: Some(BlockOrigin::doh()),
        device: Some("AbC123xY"),
        device_name: Some("living room laptop"),
      },
    )
    .unwrap();

    assert!(result.synthetic_response);
    assert_eq!(msg.answers.len(), 1);
  }

  #[test]
  fn transport_and_client_conditions_are_grouped_with_and() {
    let mut rule = rewrite(RewriteMatchWhenType::Exact, "app.test");
    rule.conditions.transports = vec![RewriteTransportCondition::DoH];
    rule.conditions.client_origins = vec![RewriteClientCondition::Windows];
    rule.behavior =
      RewriteBehavior::Respond { records: vec![a_record("10.0.0.3")], ttl: None };

    let mut msg = query("app.test.", RecordType::A);
    let result = apply_rewrites_with_context(
      Some(&[rule.clone()]),
      &mut msg,
      RewriteContext {
        origin: Some(BlockOrigin::Client {
          client: ClientOrigin::Windows,
          transport: TransportOrigin::Plain,
        }),
        device: None,
        device_name: None,
      },
    )
    .unwrap();

    assert!(!result.synthetic_response);
    assert!(msg.answers.is_empty());

    let mut msg = query("app.test.", RecordType::A);
    let result = apply_rewrites_with_context(
      Some(&[rule.clone()]),
      &mut msg,
      RewriteContext {
        origin: Some(BlockOrigin::Client {
          client: ClientOrigin::Linux,
          transport: TransportOrigin::DoH,
        }),
        device: None,
        device_name: None,
      },
    )
    .unwrap();

    assert!(!result.synthetic_response);
    assert!(msg.answers.is_empty());

    let mut msg = query("app.test.", RecordType::A);
    let result = apply_rewrites_with_context(
      Some(&[rule]),
      &mut msg,
      RewriteContext {
        origin: Some(BlockOrigin::Client {
          client: ClientOrigin::Windows,
          transport: TransportOrigin::DoH,
        }),
        device: None,
        device_name: None,
      },
    )
    .unwrap();

    assert!(result.synthetic_response);
    assert_eq!(msg.answers.len(), 1);
  }

  #[test]
  fn forward_rewrites_query_and_restores_original_response_names() {
    let mut rule = rewrite(RewriteMatchWhenType::Exact, "service.test");
    rule.behavior = RewriteBehavior::Forward { target: "internal.service.test.".into() };

    let mut msg = query("service.test.", RecordType::A);
    let original_queries = msg.queries.clone();
    let result = apply_rewrites(Some(&[rule]), &mut msg).unwrap();

    assert!(!result.synthetic_response);
    assert!(result.restore_original_queries);
    assert_eq!(msg.queries[0].name(), &Name::from_str("internal.service.test.").unwrap());

    let mut response = msg.clone().into_response();
    response.add_answer(Record::from_rdata(
      Name::from_str("internal.service.test.").unwrap(),
      60,
      RData::A(A(Ipv4Addr::new(10, 0, 0, 4))),
    ));

    restore_original_queries(&mut response, &original_queries, &result.rewritten_names);

    assert_eq!(response.queries, original_queries);
    assert_eq!(response.answers[0].name, Name::from_str("service.test.").unwrap());
  }

  #[test]
  fn lower_priority_number_runs_first() {
    let mut slower = rewrite(RewriteMatchWhenType::Exact, "app.test");
    slower.priority = 50;
    slower.behavior =
      RewriteBehavior::Respond { records: vec![a_record("10.0.0.50")], ttl: None };

    let mut faster = rewrite(RewriteMatchWhenType::Exact, "app.test");
    faster.priority = 10;
    faster.behavior =
      RewriteBehavior::Respond { records: vec![a_record("10.0.0.10")], ttl: None };

    let mut msg = query("app.test.", RecordType::A);
    apply_rewrites(Some(&[slower, faster]), &mut msg).unwrap();

    assert_eq!(msg.answers.len(), 1);
    assert_eq!(msg.answers[0].data, RData::A(A(Ipv4Addr::new(10, 0, 0, 10))));
  }

  #[test]
  fn disabled_rules_are_ignored() {
    let mut rule = rewrite(RewriteMatchWhenType::Exact, "app.test");
    rule.enabled = false;
    rule.behavior =
      RewriteBehavior::Respond { records: vec![a_record("10.0.0.2")], ttl: None };

    let mut msg = query("app.test.", RecordType::A);
    let result = apply_rewrites(Some(&[rule]), &mut msg).unwrap();

    assert!(!result.synthetic_response);
    assert!(msg.answers.is_empty());
  }
}
