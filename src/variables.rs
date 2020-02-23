pub enum VariableType {
    Str,
    Number,
    Any
}

pub struct Variable {
    name: String,
    variable_type: VariableType,
    value: String
}

pub struct Variables {
    variables: Vec<Variable>
}

impl Variables {
    pub fn new()->Variables {
        Variables {
            variables: vec!()
        }
    }

    pub fn add(&mut self,name:&str,value:&str,variable_type:VariableType){
        self.variables.push(Variable {
            name: String::from(name),
            value: String::from(value),
            variable_type
        });
    }

    pub fn get(&self, name: &str)-> Option<&Variable>{
        self.variables.iter().find(|&r| r.name == name)
    }
}
