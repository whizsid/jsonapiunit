use super::variables::Variables;
use super::variables::VariableType;
use std::io::stdin;
use serde_hjson::Value;
use serde_hjson::Map;
use boa::exec;
use std::collections::HashMap;
use std::str::FromStr;

pub struct Interpreter {
    variables: Variables
}

impl Interpreter {
    pub fn new(pre_variables_opt: Option< Variables>)->Interpreter {
        let mut variables = Variables::new();

        match pre_variables_opt {
            Some(pre_variables)=>{

                for var in pre_variables.variables {
                    variables.add(&var.name,&var.value,var.variable_type);
                }
            }
            None=>{}
        }

        Interpreter {
            variables
        }
    }

    pub fn request_value(&mut self,format:&Value)->String{
        let mut format = parse_value(&format);

        if format.len()<=6 {return format};

        match format.get(0..3).unwrap() {
            "\"{{"=>{
                match format.get(format.len()-3..format.len()).unwrap() {
                    "}}\""=>{
                        format.retain(|c| !c.is_whitespace());
                        
                        match format.get(3..4).unwrap() {
                            ">"=>{
                                let colon_offset = format.find(":").unwrap();

                                let variable_name = format.get(4..colon_offset).unwrap();

                                let variable_type = format.get((colon_offset+1)..(format.len()-3)).unwrap();

                                let variable_type_enum = VariableType::from_bytes(variable_type.as_bytes());

                                println!("> {}:{} ?",variable_name,variable_type);

                                let mut variable_value = String::new();

                                stdin().read_line(&mut variable_value)
                                    .ok()
                                    .expect(&format!("Couldn't read the value for {}:{}",variable_name,variable_type));


                                if let VariableType::Str = variable_type_enum {
                                    let mut new_variable_value = String::from("\"");
                                    new_variable_value.push_str(&variable_value);
                                    new_variable_value.push('"');
                                    variable_value = new_variable_value;
                                }

                                self.variables.add(variable_name,&variable_value,variable_type_enum);

                                variable_value
                            }
                            _=>{
                                match self.variables.get(&format.get(3..(format.len()-3)).unwrap()) {
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

    pub fn response_value(&mut self,format: Value, response_value_org: Value)->bool{
        let mut format = parse_value(&format);
        let response_value = parse_value(&response_value_org);

        if format.len()<=6 {return format==response_value};

        match format.get(0..3).unwrap() {
            "\"{{"=>{
                match format.get(format.len()-3..format.len()).unwrap() {
                    "}}\""=>{
                        format.retain(|c| !c.is_whitespace());

                        match format.find(":") {
                            Some(colon_position)=>{
                                match format.find("&&") {
                                    Some(first_and_position)=>{
                                        let comparison = format.get((first_and_position+2)..(format.len()-3)).unwrap();
                                        let variable_name = format.get(3..colon_position).unwrap();
                                        let variable_types = parse_types(format.get((colon_position+1)..first_and_position).unwrap());

                                        let mut type_matched = false;
                                        let mut variable_type = VariableType::Null;

                                        for var_type in variable_types {
                                            match var_type {
                                                VariableType::Null=>{}
                                                _=>{
                                                    if check_type(&var_type, &response_value_org) {
                                                        type_matched = true;
                                                    }
                                                    variable_type = var_type;
                                                }
                                            }
                                        }

                                        if !type_matched {
                                            self.variables.add(&variable_name,&VariableType::get_default_value(&variable_type),variable_type);
                                            println!("WARN : Type not matching for the variable '{}'. Assigned the default value.",variable_name);
                                        } else {
                                            self.variables.add(&variable_name,&response_value,variable_type);
                                        }

                                        let mut js_definitions = self.variables.get_js_definitions();
                                        js_definitions.push_str("if(");
                                        js_definitions.push_str(comparison);
                                        js_definitions.push_str(") {return true;} else {return false;}");

                                        let output = exec(&js_definitions);

                                        if output =="true" {true} else {false}
                                    }
                                    None=>{
                                        let variable_name = format.get(3..colon_position).unwrap();
                                        let variable_type = format.get((colon_position+1)..format.len()-3).unwrap();
                                        let variable_type = VariableType::from_bytes(variable_type.as_bytes());

                                        let type_matched = check_type(&variable_type, &response_value_org);

                                        if !type_matched {
                                            self.variables.add(&variable_name,&VariableType::get_default_value(&variable_type),variable_type);
                                            println!("WARN : Type not matching for the variable '{}'. Assigned the default value.",variable_name);
                                        } else {
                                            self.variables.add(&variable_name,&response_value,variable_type);
                                        }


                                        type_matched
                                    }
                                }
                            },
                            None => {
                                let types = parse_types(format.get(3..(format.len()-3)).unwrap());
                                let mut type_matched = false;

                                for var_type in types {
                                    let type_checked = check_type(&var_type, &response_value_org);
                                    if type_checked {
                                        type_matched = true;
                                    };
                                };

                                type_matched
                            }
                        }
                    }
                    _=>{
                        format == response_value
                    }
                }
            }
            _=>{
                format == response_value
            }
        }
    }

    pub fn parse_request_body(&mut self, body:Map<String,Value>)->Map<String,Value>{
        fn parse_body(this:&mut Interpreter, body:Map<String,Value>)->Map<String,Value>{
            let mut map:Map<String,Value> = Map::new();
            for (k,v) in body {
                let new_val = match v {
                    Value::Object(obj)=>{
                        Value::Object(parse_body(this, obj))
                    }
                    Value::Array(arr)=>{
                        Value::Array(arr.iter().map(|val|{ Value::from_str(&this.request_value(val)).unwrap()}).collect::<Vec<_>>())
                    }
                    _=>{
                        let value = this.request_value(&v);
                        
                        Value::from_str(&value).unwrap()
                    }
                };

                map.insert(k, new_val);
            }

            map
        }

        parse_body(self, body)
    }
}

pub fn parse_types(types:&str)->Vec<VariableType>{
    let types = String::from(types);

    types.split("|").map(|splited|{VariableType::from_bytes(splited.as_bytes())}).collect()
}

pub fn parse_value(value:&Value)->String {
    match value {
        Value::String(val_str)=>{

            let mut prefixed = String::from("\"");

            prefixed.push_str(&val_str);

            prefixed.push('"');

            prefixed
        }
        Value::Null =>{
            String::from("null")
        }
        Value::Bool(val_bool)=>{
            String::from(if val_bool.clone() {"true"} else {"false"})
        }
        Value::Object(_)=>{
            String::from("{}")
        }
        Value::Array(_)=>{
            String::from("[]")
        }
        Value::F64(float)=>{
            float.to_string()
        }
        Value::I64(int)=>{
            int.to_string()
        }
        Value::U64(unsigned)=>{
            unsigned.to_string()
        }
    }
}

pub fn check_type(var_type:&VariableType,value:&Value)->bool {
    let mut type_matched = false;

    if let VariableType::Int = var_type {
        if let Value::I64(_) = value {
            type_matched = true;
        };
    };

    if let VariableType::Str = var_type {
        if let Value::String(_) = value {
            type_matched = true;
        }
    }

    if let VariableType::Float = var_type {
        if let Value::F64(_) = value {
            type_matched = true;
        }
    }

    if let VariableType::Null = var_type {
        if let Value::Null = value {
            type_matched = true;
        }
    }

    if let VariableType::Arr = var_type {
        if let Value::Array(_) = value {
            type_matched = true;
        }
    }

    if let VariableType::Obj = var_type {
        if let Value::Object(_) = value {
            type_matched = true;
        }
    }

    if let VariableType::Bool = var_type {
        if let Value::Bool(_) = value {
            type_matched = true;
        }
    }

    if let VariableType::Any = var_type {
        type_matched = true;
    }

    type_matched
}