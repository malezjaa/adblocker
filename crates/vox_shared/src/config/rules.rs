use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::config::Config;

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
              "block rule '{}' begins with @@ which inverts the condition; use Allow \
               action instead and remove the @@ prefix",
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
