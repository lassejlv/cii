use std::collections::HashMap;
use crate::expr::LiteralValue;

pub struct Environment<'a> {
    values: HashMap<&'a str, LiteralValue<'a>>,
}


impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new()
        }
    }


    pub fn define(&mut self, name: &'a str, value: LiteralValue<'a>) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&LiteralValue<'a>> {
        self.values.get(name)
    }
}
