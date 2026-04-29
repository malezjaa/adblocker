use crate::context::Context;
use crate::middleware::{Middleware, MiddlewareResult};

pub struct Pipeline {
  middlewares: Vec<Box<dyn Middleware>>,
}

impl Pipeline {
  pub async fn run(&self, ctx: &mut Context) -> MiddlewareResult {
    for middleware in &self.middlewares {
      match middleware.process(ctx).await {
        MiddlewareResult::Next => continue,
        result => return result,
      }
    }
    MiddlewareResult::Next
  }

  pub fn new() -> Self {
    Self { middlewares: vec![] }
  }

  pub fn add(mut self, middleware: impl Middleware + 'static) -> Self {
    let middleware: Box<dyn Middleware> = Box::new(middleware);
    self.middlewares.push(middleware);
    self
  }
}
