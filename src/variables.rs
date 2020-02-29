use serde_json::Value;

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub value: Value
}

pub struct Variables {
    pub variables: Vec<Variable>
}

impl Variables {
    pub fn new()->Variables {
        Variables {
            variables: vec!()
        }
    }

    /// Adding a variable to the collection
    pub fn add(&mut self,name:&str,value:Value){
        self.variables.push(Variable {
            name: String::from(name),
            value: value
        });
    }

    pub fn get(&self, name: &str)-> Option<&Variable>{
        self.variables.iter().find(|&r| r.name == name)
    }

    pub fn len(&self)->usize{
        self.variables.len()
    }

    pub fn get_js_definitions(&self)->String{
        let mut declare = String::from("");

        for var in self.variables.iter() {
            declare.push_str(&format!("var {} = {};\n",&var.name,var.value));
        }

        declare
    }
}
