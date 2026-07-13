use std::{sync::Arc, time::Duration};

use anyhow::Result;
use hickory_resolver::{
  Resolver, TokioResolver,
  config::{NameServerConfig, ResolverConfig},
  net::runtime::TokioRuntimeProvider,
};
use tracing::{trace, warn};
use vox_shared::config::Config;

pub type HickoryResolver = Resolver<TokioRuntimeProvider>;

pub fn create_hickory_resolver(config: &Config) -> Result<HickoryResolver> {
  let mut r_config = ResolverConfig::default();

  if !config.resolver.upstreams.is_empty() {
    for upstream in &config.resolver.upstreams {
      r_config.add_name_server(NameServerConfig::https(
        upstream.addr,
        Arc::from(upstream.name.as_str()),
        None,
      ));
      trace!(addr = %upstream.addr, name = %upstream.name, "added upstream server");
    }
  } else {
    warn!("no upstream servers specified. is this desired?")
  }

  let mut resolver_builder =
    TokioResolver::builder_with_config(r_config, TokioRuntimeProvider::default());

  let opts = resolver_builder.options_mut();
  opts.negative_min_ttl = Some(Duration::from_secs(60));
  opts.positive_min_ttl = Some(Duration::from_secs(60));
  opts.num_concurrent_reqs = 3;
  opts.validate = config.resolver.dnssec;

  Ok(resolver_builder.build()?)
}
