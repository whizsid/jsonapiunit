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
    variables: Vec<Variable>,
    current: u32
}

impl Variables {
    pub fn new()->Variables {
        Variables {
            variables: vec!(),
            current:0
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
}

impl Iterator for Variables {
    type Item = Variable;
    
    fn next(&mut self) -> Option<Variable> {
        self.current = self.current +1;

        let variable = self.variables.get(self.current as usize).unwrap();

        Some(variable.clone())
    }
}