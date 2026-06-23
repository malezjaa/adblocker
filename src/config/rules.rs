use crate::app_error;
use crate::config::Config;
use crate::context::Context;
use crate::dashboard::AppError;
use crate::dashboard::auth::AuthGuard;
use anyhow::anyhow;
use axum::Json;
use axum::extract::{Path, Query, State};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Rule {
  pub domain: String,
  pub action: RuleAction,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum RuleAction {
  #[serde(rename = "block")]
  Block,
  #[serde(rename = "allow")]
  Allow,
}

impl Rule {
  pub fn adblock_rule(&self) -> String {
    match self.action {
      RuleAction::Block => self.domain.clone(),
      RuleAction::Allow => format!("@@{}", self.domain),
    }
  }
}

impl Config {
  pub fn validate_rules(&self) {
    if let Some(rules) = &self.rules {
      rules.par_iter().for_each(|rule| {
        if rule.domain.starts_with("@@") {
          match rule.action {
            RuleAction::Block => error!(
              "block rule '{}' begins with @@ which inverts the condition; \
                     use Allow action instead and remove the @@ prefix",
              rule.domain
            ),
            RuleAction::Allow => {
              error!(
                "allow rule '{}' already contains @@; remove the prefix",
                rule.domain
              )
            }
          }
        }
      });
    }
  }
}

#[derive(Serialize)]
pub struct PaginatedRules {
  total: i64,
  page: u32,
  per_page: u32,
  items: Vec<Rule>,
}

#[derive(Deserialize)]
pub struct RulesQuery {
  page: Option<u32>,
  per_page: Option<u32>,
  domain: Option<String>,
}

pub async fn rule_handler(
  _guard: AuthGuard,
  State(ctx): State<Context>,
  Query(query): Query<RulesQuery>,
) -> anyhow::Result<Json<PaginatedRules>, AppError> {
  let config = ctx.config();

  let page = query.page.unwrap_or(1).max(1);
  let per_page = query.per_page.unwrap_or(50).clamp(1, 500);

  let all_rules = config.rules.clone().unwrap_or_default();

  let filtered: Vec<Rule> = match &query.domain {
    Some(domain) => {
      all_rules.into_iter().filter(|r| r.domain.contains(domain.as_str())).collect()
    }
    None => all_rules,
  };

  let total = filtered.len() as i64;
  let start = ((page - 1) as usize) * (per_page as usize);
  let items = filtered.into_iter().skip(start).take(per_page as usize).collect();

  Ok(Json(PaginatedRules { total, page, per_page, items }))
}

pub async fn create_rule(
  _guard: AuthGuard,
  State(ctx): State<Context>,
  Json(body): Json<Rule>,
) -> anyhow::Result<Json<Rule>, AppError> {
  let old_config = ctx.config().clone();
  let mut new_config = old_config.clone();

  if let Some(rules) = &mut new_config.rules {
    if rules.iter().any(|rule| rule.domain == body.domain) {
      app_error!("Rule with domain '{}' already exists", body.domain);
    }

    rules.push(body.clone());
  } else {
    new_config.rules = Some(vec![body.clone()]);
  }

  fs_err::write(ctx.config_path(), toml::to_string(&new_config)?)?;
  ctx.apply_config_change(old_config, new_config).await?;

  Ok(Json(body))
}

#[derive(Deserialize)]
pub struct UpdateRuleBody {
  action: RuleAction,
}

pub async fn update_rule(
  _guard: AuthGuard,
  State(ctx): State<Context>,
  Path(domain): Path<String>,
  Json(body): Json<UpdateRuleBody>,
) -> anyhow::Result<Json<Rule>, AppError> {
  let old_config = ctx.config().clone();
  let mut new_config = old_config.clone();

  let rules = new_config.rules.as_mut().ok_or_else(|| anyhow!("No rules configured"))?;

  let rule = rules
    .iter_mut()
    .find(|r| r.domain == domain)
    .ok_or_else(|| anyhow!("Rule with domain '{}' not found", domain))?;

  rule.action = body.action;
  let updated = rule.clone();

  fs_err::write(ctx.config_path(), toml::to_string(&new_config)?)?;
  ctx.apply_config_change(old_config, new_config).await?;

  Ok(Json(updated))
}

pub async fn delete_rule(
  _guard: AuthGuard,
  State(ctx): State<Context>,
  Path(domain): Path<String>,
) -> anyhow::Result<Json<()>, AppError> {
  let old_config = ctx.config().clone();
  let mut new_config = old_config.clone();

  let rules = new_config.rules.as_mut().ok_or_else(|| anyhow!("No rules configured"))?;

  let len_before = rules.len();
  rules.retain(|r| r.domain != domain);

  if rules.len() == len_before {
    app_error!("Rule with domain '{}' not found", domain);
  }

  fs_err::write(ctx.config_path(), toml::to_string(&new_config)?)?;
  ctx.apply_config_change(old_config, new_config).await?;

  Ok(Json(()))
}
