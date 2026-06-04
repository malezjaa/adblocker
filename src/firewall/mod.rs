pub mod external_dns;
pub mod open_port;
pub mod override_dns;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Protocol {
  UDP,
  TCP,
}
