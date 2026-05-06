use addr::parse_dns_name;
use hickory_proto::op::Message;

pub fn registered_domain(fqdn: &str) -> String {
  parse_dns_name(fqdn)
    .ok()
    .and_then(|n| n.root())
    .map(|r| r.to_string())
    .unwrap_or_else(|| fqdn.to_string())
}

pub fn query_domain(msg: &Message) -> Option<String> {
  msg.queries
    .first()
    .map(|q| q.name().to_string())
    .map(|d| d.trim_end_matches('.').to_string())
}