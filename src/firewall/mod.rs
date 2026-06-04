pub mod external_dns;
pub mod override_dns;
pub mod open_port;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Protocol {
  UDP,
  TCP,
}