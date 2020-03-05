use serde_json::Value;

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub value: Value,
}

/// All variables storing inside this
pub struct Variables {
    pub variables: Vec<Variable>,
    pub js_used_variable_count: usize,
}

impl Variables {
    pub fn new() -> Variables {
        Variables {
            variables: vec![],
            js_used_variable_count: 0,
        }
    }

    /// Adding a variable to the collection
    pub fn add(&mut self, name: &str, value: Value) {
        let exist = self.get(name);

        match exist {
            Some(_) => {
                let index = self.variables.iter().position(|x| x.name == name).unwrap();

                self.variables.remove(index);

                self.add(name, value);
            }
            None => {
                self.variables.push(Variable {
                    name: String::from(name),
                    value: value,
                });
            }
        }
    }

    pub fn get(&self, name: &str) -> Option<&Variable> {
        self.variables.iter().find(|&r| r.name == name)
    }

    pub fn get_js_definitions(&mut self) -> String {
        let mut declare = String::from("");

        for var in self.variables.iter() {
            declare.push_str(&format!("var {} = {};\n", &var.name, var.value));
        }

        declare
    }
}

#[cfg(test)]
mod tests {

    use super::Variables;
    use serde_json::Number;
    use serde_json::Value;

    #[test]
    pub fn test_variable_add() {
        let mut variables = Variables::new();

        variables.add("foo", Value::String(String::from("Foo")));

        let variable = variables.get("foo");

        assert_eq!(variable.is_none(), false);

        match variable {
            Some(var) => {
                assert_eq!(format!("{}", var.value), "\"Foo\"");
            }
            None => {}
        }
    }

    #[test]
    pub fn test_existing_variable_add() {
        let mut variables = Variables::new();

        variables.add("foo", Value::String(String::from("Bar")));

        variables.add("foo", Value::String(String::from("Foo")));

        let variable = variables.get("foo");

        assert_eq!(variable.is_none(), false);

        match variable {
            Some(var) => {
                assert_eq!(format!("{}", var.value), "\"Foo\"");
            }
            None => {}
        }
    }

    #[test]
    pub fn test_get_js_definition() {
        let mut variables = Variables::new();

        variables.add("foo", Value::String(String::from("Foo")));

        variables.add("bar", Value::Number(Number::from(12)));

        assert_eq!(
            &variables.get_js_definitions(),
            "var foo = \"Foo\";\nvar bar = 12;\n"
        );
    }
}
