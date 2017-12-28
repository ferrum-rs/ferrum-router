use std::collections::BTreeMap;
use ferrum::Handler;

pub type Params = BTreeMap<String, String>;

pub struct RouteMatch<'a> {
    pub handler: &'a Box<Handler>,
    pub params: Params
}

impl<'a> RouteMatch<'a> {
    pub fn new(handler: &'a Box<Handler>, params: Params) -> RouteMatch {
        RouteMatch {
            handler,
            params
        }
    }
}