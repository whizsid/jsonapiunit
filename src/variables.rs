use serde_json::Value;

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub value: Value
}

pub struct Variables {
    pub variables: Vec<Variable>,
    pub js_used_variable_count: usize
}

impl Variables {
    pub fn new()->Variables {
        Variables {
            variables: vec!(),
            js_used_variable_count:0
        }
    }

    /// Adding a variable to the collection
    pub fn add(&mut self,name:&str,value:Value){
        let exist = self.get(name);

        match exist {
            Some(_)=>{
                panic!("Re assignment for the {} variable.",name)
            }
            None=>{
                self.variables.push(Variable {
                    name: String::from(name),
                    value: value
                });
            }
        }
    }

    pub fn get(&self, name: &str)-> Option<&Variable>{
        self.variables.iter().find(|&r| r.name == name)
    }

    pub fn get_js_definitions(&mut self)->String{
        let mut declare = String::from("");

        for var in self.variables.iter() {
            declare.push_str(&format!("var {} = {};\n",&var.name,var.value));
        }

        declare
    }
}
