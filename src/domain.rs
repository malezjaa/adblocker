use addr::parse_dns_name;

pub fn registered_domain(fqdn: &str) -> String {
  parse_dns_name(fqdn)
    .ok()
    .and_then(|n| n.root())
    .map(|r| r.to_string())
    .unwrap_or_else(|| fqdn.to_string())
}