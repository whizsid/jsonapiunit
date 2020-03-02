use super::variables::Variables;
use std::io::stdin;
use serde_json::Value;
use serde_json::Map;
use boa::exec;
use serde_json::from_str;
use regex::Regex;
use colored::*;

pub struct Interpreter {
    variables: Variables,
    tot_fails: i32,
    cur_fails: i32,
    tot_asserts: i32,
    cur_asserts: i32
}

impl Interpreter {
    pub fn new(pre_variables_opt: Option< Variables>)->Interpreter {
        let mut variables = Variables::new();

        match pre_variables_opt {
            Some(pre_variables)=>{

                for var in pre_variables.variables {
                    variables.add(&var.name,var.value);
                }
            }
            None=>{}
        }

        Interpreter {
            variables,
            tot_fails: 0,
            cur_fails: 0,
            tot_asserts: 0,
            cur_asserts: 0
        }
    }

    pub fn request_value(&mut self,format:&Value)->String{
        let mut format = format!("{}",format);

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

                                let mut variable_value = String::new();

                                println!("{} : {}?","INPUT".blue(),variable_name);

                                stdin().read_line(&mut variable_value)
                                    .ok()
                                    .expect(&format!("Couldn't read the value for {}:{}",variable_name,variable_type));

                                variable_value = String::from(variable_value.trim());
                                
                                // Wrapping the value by quotes if string
                                if variable_type == "string" {
                                    let mut new_variable_value = String::from("\"");
                                    new_variable_value.push_str(&variable_value);
                                    new_variable_value.push_str("\"");
                                    variable_value = new_variable_value;
                                }

                                let mut entered_value = from_str(&variable_value.clone()).expect("Invalid value entered!");

                                let check_type = type_check(variable_type, &entered_value);

                                if !check_type {
                                    variable_value= get_default_value(variable_type).expect(&format!("Unsupported type given for {}.",variable_name));
                                    println!("{} : Type not matching for the variable '{}'. Assigned the default value.","WARN".yellow(),variable_name);
                                    entered_value = from_str(&variable_value).unwrap();
                                }

                                self.variables.add(variable_name,entered_value);

                                variable_value
                            }
                            _=>{
                                match self.variables.get(&format.get(3..(format.len()-3)).unwrap()) {
                                    Some(variable)=>{
                                        format!("{}",variable.value)
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

    pub fn request_header(&mut self, format:&str)->String {
        let mut string = String::new();

        let mut found = 0;
        for mat in Regex::new(r"\{\{(.*?)\}\}").unwrap().find_iter(format) {
            let json_val = Value::String(String::from(mat.as_str()));
            let value:Value = from_str(&self.request_value(&json_val)).unwrap();

            string.push_str(format.get(0..mat.start()).unwrap());
            string.push_str(value.as_str().unwrap());

            string.push_str(format.get(mat.end()..format.len()).unwrap());
            found+=1;
        }
        
        if found == 0 {
            String::from(format)
        } else {
            string
        }

    }

    fn add_response_variable(&mut self,variable_name:&str,types:&str,value:Value)->bool{

        let types = types.split("|");
        let mut type_matched = false;
        let mut conflict_types = false;
        let mut default_value = String::from("null");

        for var_type in types {
            match var_type {
                "null"=>{
                    if let Value::Null = value {
                        if !type_matched {
                            type_matched = true;
                        }
                    }
                }
                _=>{
                    if conflict_types {
                        panic!("Conflicting types supplied for the variable {}",variable_name);
                    }

                    conflict_types = true;

                    default_value = match get_default_value(var_type) {
                        Ok(string)=>{string}
                        Err(_)=>{
                            panic!("Unsupported type given for {}",variable_name)
                        }
                    };

                    if ! type_matched {
                        type_matched =  type_check(var_type,&value);
                    }
                    
                }
            }
        }

        if !type_matched {
            self.variables.add(&variable_name,from_str(&default_value).unwrap());
            println!("{} : Type not matching for the variable '{}'. Assigned the default value.","WARN".yellow(),variable_name);
        } else {
            self.variables.add(&variable_name,value);
        }

        type_matched
    }

    pub fn response_value(&mut self,format: Value, response_value_org: Value)->bool{
        let mut format = format!("{}",format);
        let response_value = format!("{}",response_value_org);
        let passed:bool;

        if format.len()<=6 {
            passed = format==response_value
        } else {
            passed = match format.get(0..3).unwrap() {
                "\"{{"=>{
                    match format.get(format.len()-3..format.len()).unwrap() {
                        "}}\""=>{
                            format.retain(|c| !c.is_whitespace());

                            match format.find(":") {
                                Some(colon_position)=>{
                                    match format.find("&&") {
                                        Some(first_and_position)=>{
                                            let comparison = format.get((first_and_position+2)..(format.len()-3)).unwrap();
                                            let types = format.get((colon_position+1)..first_and_position).unwrap();

                                            let variable_name = format.get(3..colon_position).unwrap();

                                            let type_checked = self.add_response_variable(variable_name, types, response_value_org);

                                            if !type_checked {
                                                return type_checked;
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
                                            let types = format.get((colon_position+1)..format.len()-3).unwrap();

                                            let type_checked = self.add_response_variable(variable_name, types, response_value_org);

                                            type_checked
                                        }
                                    }
                                },
                                None => {
                                    let types = format.get(3..(format.len()-3)).unwrap().split("|");
                                    let mut type_matched = false;

                                    for var_type in types {
                                        
                                    let type_checked = type_check(var_type, &response_value_org);

                                    if type_checked {
                                        type_matched = true;
                                    }
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
            };
        }

        self.cur_asserts +=1;
        self.tot_asserts += 1;

        if !passed {
            self.cur_fails += 1;
            self.tot_fails += 1;
        } 

        println!("{} : Assertion: {} , Fails:{}, Assertions:{}, TotFails:{}, TotAssertions:{}",
            if passed {"PASSED".green()} else {"FAILED".red()},
            format,
            self.cur_fails,
            self.cur_asserts, 
            self.tot_fails,
            self.tot_asserts
        );

        passed
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
                        Value::Array(arr.iter().map(|val|{ from_str(&this.request_value(val)).unwrap()}).collect::<Vec<_>>())
                    }
                    _=>{
                        let value = this.request_value(&v);
                        
                        from_str(&value).unwrap()
                    }
                };

                map.insert(k, new_val);
            }

            map
        }

        parse_body(self, body)
    }

    pub fn parse_response_body(&mut self, test_body: Map<String,Value>, res_body:Map<String,Value>)->bool{

        self.cur_asserts = 0;
        self.cur_fails = 0;

        fn parse_body(this:&mut Interpreter,test_body: Map<String,Value> ,res_body: Map<String,Value>)->bool{
            let mut passed = true;
            let mut test_body_iter = test_body.iter();

            while let Some((tk,tv)) = test_body_iter.next() {
                let checked = match res_body.get(tk) {
                    Some(val)=>{
                        match val {
                            Value::Object(obj)=>{
                                if let Value::Object(obj2) = tv {
                                    parse_body(this, obj2.to_owned(), obj.to_owned())
                                } else {
                                    this.response_value(tv.to_owned(), Value::Object(obj.to_owned()))
                                }
                            }
                            Value::Array(arr)=>{
                                if let Value::Array(arr2) = tv {
                                    let mut arr_iter = arr.iter();
                                    let mut arr2_iter = arr2.iter();
                                    let mut arr_passed = true;

                                    while let Some(val) = arr_iter.next() {
                                        while let Some(val2) = arr2_iter.next() {
                                            if ! this.response_value(val2.to_owned(), val.to_owned()){
                                                arr_passed = false;
                                            }
                                        }
                                    }

                                    arr_passed
                                    
                                } else {
                                    this.response_value(tv.to_owned(), Value::Array(arr.to_owned()))
                                }
                            }
                            _=>{
                                this.response_value(tv.to_owned(), val.to_owned())
                            }
                        }
                    }
                    None=>{
                        false
                    }

                };

                if !checked {
                    passed = false;
                }
            }

            passed
        }

        parse_body(self,test_body ,res_body)

    }
}

pub fn type_check(var_type:&str,value:&Value)->bool {
    match value {
        Value::Number(_)=>{
            var_type == "number" || var_type =="any"
        }
        Value::Array(_)=>{
            var_type == "array" || var_type =="any"
        }
        Value::Bool(_)=>{
            var_type == "boolean" || var_type =="any"
        }
        Value::Null=>{
            var_type == "null" || var_type =="any"
        }
        Value::String(_)=>{
            var_type == "string" || var_type =="any"
        }
        Value::Object(_)=>{
            var_type == "object" || var_type =="any"
        }
    }
}

pub fn get_default_value(var_type:&str)->Result<String,&str>{
    match var_type {
        "boolean"=>{
            Ok(String::from("false"))
        }
        "number"=>{
            Ok(String::from("0"))
        }
        "string"=>{
            Ok(String::from("\"\""))
        }
        "object"=>{
            Ok(String::from("{}"))
        }
        "array"=>{
            Ok(String::from("[]"))
        }
        "null"=>{
            Ok(String::from("null"))
        }
        "any"=>{
            Ok(String::from("null"))
        }
        _=>{
            Err("Unssupported Type.")
        }
    }
}
