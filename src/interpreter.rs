use super::variables::Variables;
use super::variables::VariableType;
use super::config::DefaultConfig;
use std::io::stdin;

pub struct Interpreter {
    variables: Variables,
    default_config: DefaultConfig
}

impl Interpreter {
    pub fn new(default_config: DefaultConfig)->Interpreter {
        let variables = Variables::new();

        Interpreter {
            variables,
            default_config
        }
    }

    pub fn request_value(&mut self,format:&str)->String{
        let mut format = String::from(format);

        if format.len()<=5 {return format};

        match format.get(0..2).unwrap() {
            "{{"=>{
                match format.get(format.len()-2..format.len()).unwrap() {
                    "}}"=>{
                        format.retain(|c| !c.is_whitespace());

                        match format.get(2..3).unwrap() {
                            ">"=>{
                                let colon_offset = format.find(":").unwrap();


                                let variable_name = format.get(3..colon_offset).unwrap();

                                let variable_type = format.get((colon_offset+1)..(format.len()-2)).unwrap();

                                let variable_type_enum = VariableType::from_bytes(variable_type.as_bytes());

                                println!("> {}:{} ?",variable_name,variable_type);

                                let mut variable_value = String::new();

                                stdin().read_line(&mut variable_value)
                                    .ok()
                                    .expect(&format!("Couldn't read the value for {}:{}",variable_name,variable_type));

                                self.variables.add(variable_name,&variable_value,variable_type_enum);

                                variable_value.clone()
                            }
                            _=>{
                                match self.variables.get(&format.get(2..(format.len()-2)).unwrap()) {
                                    Some(variable)=>{
                                        variable.value.clone()
                                    }
                                    None=>{
                                        panic!("Can not find a variable named {}",format)
                                    }
                                }
                            }
                        }
                    }
                    _=>{
                        format
                    }
                }
            }
            _=>{
                format
            }
        }
            
    }

    pub fn response_value(){

    }
}