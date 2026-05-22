use crate::context::Context;
use crate::rewrite::record::construct_rewrite_records;
use crate::rewrite::{Rewrite, RewriteAction, RewriteMatchWhenType};
use anyhow::Result;
use hickory_proto::op::{Message, Query, ResponseCode};
use hickory_proto::rr::{Name, Record};
use std::str::FromStr;

pub struct RewriteResult {
  pub synthetic_response: bool,
  pub restore_original_queries: bool,
}

pub fn apply_rewrites(ctx: &Context, msg: &mut Message) -> Result<RewriteResult> {
  let Some(rewrites) = &ctx.config().rewrites else {
    return Ok(RewriteResult {
      synthetic_response: false,
      restore_original_queries: false,
    });
  };

  let mut synthetic_answers = Vec::new();
  let mut rewritten_queries = Vec::new();
  let mut replace_queries = false;

  for query in &msg.queries {
    let host = query.name().to_string();
    let host = host.trim_end_matches('.');

    for rewrite in rewrites {
      if !rewrite_matches(rewrite, host)? {
        continue;
      }

      match classify_actions(&rewrite.actions) {
        RewriteBehavior::NxDomain => {
          msg.metadata.response_code = ResponseCode::NXDomain;
          *msg = msg.clone().into_response();

          return Ok(RewriteResult {
            synthetic_response: true,
            restore_original_queries: false,
          });
        }

        RewriteBehavior::NoError => {
          *msg = msg.clone().into_response();

          return Ok(RewriteResult {
            synthetic_response: true,
            restore_original_queries: false,
          });
        }

        RewriteBehavior::Rewrite(name) => {
          replace_queries = true;

          rewritten_queries.push(Query::query(Name::from_str(name)?, query.query_type()));
        }

        RewriteBehavior::None => {}
      }

      synthetic_answers.extend(
        construct_rewrite_records(query, rewrite)?
          .into_iter()
          .map(|rdata| Record::from_rdata(query.name().clone(), 300, rdata)),
      );
    }
  }

  msg.answers.extend(synthetic_answers);

  if replace_queries {
    msg.queries = rewritten_queries;
  }

  Ok(RewriteResult {
    synthetic_response: !msg.answers.is_empty(),
    restore_original_queries: replace_queries,
  })
}

fn rewrite_matches(rewrite: &Rewrite, host: &str) -> Result<bool> {
  Ok(match rewrite.when.ty {
    RewriteMatchWhenType::Exact => rewrite.when.value == host,
    RewriteMatchWhenType::Regex => rewrite
      .regex
      .as_ref()
      .expect("should be compiled after config was loaded")
      .is_match(host),
  })
}

enum RewriteBehavior<'a> {
  NxDomain,
  NoError,
  Rewrite(&'a str),
  None,
}

fn classify_actions(actions: &[RewriteAction]) -> RewriteBehavior<'_> {
  for action in actions {
    match action {
      RewriteAction::NXDOMAIN => return RewriteBehavior::NxDomain,
      RewriteAction::NOERROR => return RewriteBehavior::NoError,
      RewriteAction::Rewrite { value } => return RewriteBehavior::Rewrite(value),
      _ => {}
    }
  }

  RewriteBehavior::None
}

pub fn restore_original_queries(response: &mut Message, original_queries: &[Query]) {
  response.queries = original_queries.to_vec();

  if let Some(first) = original_queries.first() {
    for answer in &mut response.answers {
      answer.name = first.name().clone();
    }
  }
}
