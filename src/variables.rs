#[derive(Clone)]
pub enum VariableType {
    Str,
    Int,
    Float,
    Null,
    Any,
    Arr,
    Obj,
    Bool
}

impl VariableType {
    pub fn from_bytes(bytes: &[u8])->VariableType {
        match bytes {
            b"integer"=>{
                VariableType::Int
            }
            b"string"=>{
                VariableType::Str
            }
            b"float"=>{
                VariableType::Float
            }
            b"null"=>{
                VariableType::Null
            }
            b"any"=>{
                VariableType::Any
            }
            b"array"=>{
                VariableType::Arr
            }
            b"object"=>{
                VariableType::Obj
            }
            b"boolean"=>{
                VariableType::Bool
            }
            _=>{
                panic!("Invalid variable type supplied.");
            }
        }
    }

    pub fn get_default_value(var_type:&VariableType)->String {
        String::from(match var_type {
            VariableType::Any=>{"\"\""}
            VariableType::Arr=>{"[]"}
            VariableType::Bool=>{"false"}
            VariableType::Float=>{"0.00"}
            VariableType::Int=>{"0"}
            VariableType::Null=>{"null"}
            VariableType::Obj=>{"{}"}
            VariableType::Str=>{"\"\""}
        })
    }
}

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub variable_type: VariableType,
    pub value: String
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
