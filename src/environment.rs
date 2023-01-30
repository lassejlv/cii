use crate::expr::LiteralValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct Environment {
    //globals: Rc<RefCell<HashMap<String, LiteralValue>>>,
    values: Rc<RefCell<HashMap<String, LiteralValue>>>,
    pub enclosing: Option<Box<Environment>>,
}

fn clock_impl(_args: &Vec<LiteralValue>) -> LiteralValue {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("Could not get system time")
        .as_millis();

    LiteralValue::Number(now as f64 / 1000.0)
}

fn get_globals() -> Rc<RefCell<HashMap<String, LiteralValue>>> {
    let mut env = HashMap::new();
    env.insert(
        "clock".to_string(),
        LiteralValue::Callable {
            name: "clock".to_string(),
            arity: 0,
            fun: Rc::new(clock_impl),
        },
    );

    Rc::new(RefCell::new(env))
}

impl Environment {
    pub fn new() -> Self {
        Self {
            //globals: Rc::new(RefCell::new(get_globals())),
            values: get_globals(),
            // values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn define(&self, name: String, value: LiteralValue) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &str, distance: Option<usize>) -> Option<LiteralValue> {
        if let None = distance {
            match &self.enclosing {
                None => self.values.borrow().get(name).cloned(),
                Some(env) => env.get(name, distance),
            }
        } else {
            let distance = distance.unwrap();
            if distance == 0 {
                self.values.borrow().get(name).cloned()
            } else {
                match &self.enclosing {
                    None => panic!("Tried to resolve a variable that was defined deeper than the current environment depth"),
                    Some(env) => {
                        assert!(distance > 0);
                        env.get(name, Some(distance - 1))
                    }
                }
            }
        }
    }

    pub fn assign(&self, name: &str, value: LiteralValue, distance: Option<usize>) -> bool {
        if let None = distance {
            match &self.enclosing {
                Some(env) => env.assign(name, value, distance),
                None => match self.values.borrow_mut().insert(name.to_string(), value) {
                    Some(_) => true,
                    None => false,
                },
            }
        } else {
            let distance = distance.unwrap();
            if distance == 0 {
                self.values.borrow_mut().insert(name.to_string(), value);
                true
            } else {
                match &self.enclosing {
                    None => panic!("Tried to define a variable in a too deep level"),
                    Some(env) => env.assign(name, value, Some(distance - 1)),
                };
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_init() {
        let _environment = Environment::new();
    }
}
