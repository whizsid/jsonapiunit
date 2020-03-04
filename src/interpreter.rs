use super::variables::Variables;
#[cfg(not(test))]
use std::io::stdin;
use serde_json::Value;
use serde_json::Map;
use boa::exec;
use serde_json::from_str;
use regex::Regex;
use colored::*;

/// All codes are executing inside this
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

    #[cfg(not(test))]
    fn request_user_input(&self,variable_name:&str)->String{
        let mut variable_value = String::new();

        println!("{} : {}?","INPUT".blue(),variable_name);

        stdin().read_line(&mut variable_value)
            .ok()
            .expect(&format!("Couldn't read the value for {}",variable_name));

        variable_value
    }

    #[cfg(test)]
    fn request_user_input(&self,_variable_name:&str)->String{
        String::from("USER_INPUT")
    }

    /// Passing a pattern in a request body
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

                                let mut variable_value = self.request_user_input(variable_name);

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

    /// Parsing a pattern in a request header
    pub fn request_header(&mut self, format:&str)->String {
        let mut string = String::new();

        let mut found = 0;
        let mut last_end = 0;
        for mat in Regex::new(r"\{\{(.*?)\}\}").unwrap().find_iter(format) {
            let json_val = Value::String(String::from(mat.as_str()));
            let value:Value = from_str(&self.request_value(&json_val)).unwrap();

            string.push_str(format.get(last_end..mat.start()).unwrap());

            string.push_str(value.as_str().unwrap());

            last_end = mat.end();

            found+=1;
        }

        string.push_str(format.get(last_end..format.len()).unwrap());
        
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

    /// Matching a single value in the test case and actual response
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

    /// Parsing the request body and get the formated body
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

    /// Checking the weather the test case response body 
    /// and the actual response body are matching
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
                                    let mut arr_passed = true;

                                    while let Some(val) = arr_iter.next() {

                                        let mut arr2_iter = arr2.iter();

                                        while let Some(val2) = arr2_iter.next() {
                                            match val2 {
                                                Value::Object(val2_obj)=>{
                                                    if let Value::Object(val_obj) = val {
                                                        if !parse_body(this, val2_obj.to_owned(), val_obj.to_owned()) {
                                                            arr_passed = false;
                                                        }
                                                    } else {
                                                        arr_passed = false;
                                                    }
                                                }
                                                _=>{
                                                    if ! this.response_value(val2.to_owned(), val.to_owned()){
                                                        arr_passed = false;
                                                    }
                                                }
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

    #[cfg(test)]
    pub fn variables(&self)->&Variables{
        &self.variables
    }
}

/// # Checking the type of a Value
/// 
/// Checking the type of `serde_json:Value` with a string type name
/// 
/// ## Supported types 
/// 
/// - `number`
/// - `array`
/// - `boolean`
/// - `null`
/// - `string`
/// - `object`
/// - `any`
/// 
/// ```
///  let json_val:Value = serde_json::from_str("123").unwrap();
/// 
///  assert_eq!(type_check("number",&json_val),true);
/// ```
/// 
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

/// # Returning the default values for types
/// 
/// If user has passed a value with a wrong type these 
/// values are assigned for the variable.
/// 
/// ```
/// let default_value_bool = get_default_value("boolean").unwrap();
/// assert_eq!(&default_value_bool,"false");
/// 
/// let default_value_nmb = get_default_value("number").unwrap();
/// assert_eq!(&default_value_nmb,"0");
/// 
/// let default_value_str = get_default_value("string").unwrap();
/// assert_eq!(&default_value_str,"\"\"");
/// 
/// let default_value_obj = get_default_value("object").unwrap();
/// assert_eq!(&default_value_obj,"{}");
/// 
/// let default_value_arr = get_default_value("array").unwrap();
/// assert_eq!(&default_value_arr,"[]");
/// 
/// let default_value_nul = get_default_value("null").unwrap();
/// assert_eq!(&default_value_nul,"null");
/// 
/// let default_value_any = get_default_value("any").unwrap();
/// assert_eq!(&default_value_any,"null");
/// 
/// ```
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

#[cfg(test)]
mod tests {

    use super::Interpreter;
    use serde_json::from_str;
    use serde_json::Value;
    use serde_json::Map;
    use crate::variables::Variables;
    use serde_json::Number;

    fn make_interpreter_with_pre_variable()->Interpreter{
        let mut variables = Variables::new();

        let json_val_foo:Value = from_str("\"Foo\"").unwrap() ;
        let json_val_bar:Value = from_str("12").unwrap() ;

        variables.add("foo",json_val_foo);
        variables.add("bar",json_val_bar);

        Interpreter::new(Some(variables))
    }

    #[test]
    pub fn test_request_value_without_pattern(){
        let mut interpreter = Interpreter::new(None);

        let json_val:Value = from_str("\"Foo\"").unwrap() ;

        let req_value = interpreter.request_value(&json_val);

        assert_eq!(&req_value,"\"Foo\"")
    }

    #[test]
    pub fn test_request_value_with_variable(){
        
        let mut interpreter = make_interpreter_with_pre_variable();

        let var_name_val:Value = from_str("\"{{foo}}\"").unwrap() ;

        let foo_val = interpreter.request_value(&var_name_val);

        assert_eq!("\"Foo\"",&foo_val)
    }

    #[test]
    pub fn test_request_value_with_user_input(){
        let mut interpreter = Interpreter::new(None);

        let json_val:Value = from_str("\"{{>user_input:string}}\"").unwrap();

        let user_input = interpreter.request_value(&json_val);

        assert_eq!("\"USER_INPUT\"",&user_input)
    }

    #[test]
    pub fn test_request_header_without_pattern(){
        let mut interpreter = Interpreter::new(None);

        let header_value = interpreter.request_header("application/json");

        assert_eq!(&header_value,"application/json")
    }

    #[test]
    pub fn test_request_header_with_pattern(){
        let mut interpreter = make_interpreter_with_pre_variable();

        // At begining
        let header_value = interpreter.request_header("{{foo}} is Foo");
        assert_eq!(&header_value,"Foo is Foo");

        // At the middle
        let header_value = interpreter.request_header("Bar {{foo}} Bar");
        assert_eq!(&header_value,"Bar Foo Bar");

        // At the end
        let header_value = interpreter.request_header("Foo is {{foo}}");
        assert_eq!(&header_value,"Foo is Foo");

        // Whole sentences
        let header_value = interpreter.request_header("{{foo}}");
        assert_eq!(&header_value,"Foo");

        // Two patterns
        let header_value = interpreter.request_header("{{foo}} is {{foo}}");
        assert_eq!(&header_value,"Foo is Foo");

    }

    fn response_value(interpreter: &mut Interpreter,test:&str,res:&str)->bool {
        let test_value:Value = from_str(test).unwrap();
        let res_value:Value = from_str(res).unwrap();
        interpreter.response_value(test_value, res_value)
    }

    #[test]
    pub fn test_response_value_with_normal_values(){
        let mut interpreter = Interpreter::new(None);

        assert_eq!(response_value(&mut interpreter, "1232443323", "1232443323"),true);

        assert_ne!(response_value(&mut interpreter, "1232443323", "123243323"),true);

        assert_eq!(response_value(&mut interpreter, "\"ASB\"", "\"ASB\""),true);

        assert_ne!(response_value(&mut interpreter, "\"ASB\"", "\"ASA\""),true);

        assert_ne!(response_value(&mut interpreter, "\"ASB\"", "2123"),true);

    }

    #[test]
    pub fn test_response_value_with_type_check(){
        let mut interpreter = Interpreter::new(None);

        assert_eq!(response_value(&mut interpreter, "\"{{string}}\"", "\"Foo\""),true);

        assert_eq!(response_value(&mut interpreter, "\"{{string| null}}\"", "null"),true);

        assert_eq!(response_value(&mut interpreter, "\"{{string| null}}\"", "\"Foo\""),true);

        assert_eq!(response_value(&mut interpreter, "\"{{string| null}}\"", "1232"),false);

        assert_eq!(response_value(&mut interpreter, "\"{{string}}\"", "1232"),false);

        assert_eq!(response_value(&mut interpreter, "\"{{boolean}}\"", "true"),true);

        assert_eq!(response_value(&mut interpreter, "\"{{array}}\"", "[1,2,3]"),true);

        assert_eq!(response_value(&mut interpreter, "\"{{object}}\"", "[1,2,3]"),false);

        assert_eq!(response_value(&mut interpreter, "\"{{object}}\"", "{\"a\":123}"),true);

        assert_eq!(response_value(&mut interpreter, "\"{{any}}\"", "[1,2,3]"),true);
    }

    #[test]
    pub fn test_response_value_with_variable_creation(){
        let mut interpreter = Interpreter::new(None);

        assert_eq!(response_value(&mut interpreter, "\"{{fooStr:string}}\"", "\"Foo\""),true);

        assert_eq!(response_value(&mut interpreter, "\"{{barStr:string|null}}\"", "null"),true);

        assert_eq!(response_value(&mut interpreter, "\"{{fooNmb:number}}\"", "1234"),true);

        assert_eq!(response_value(&mut interpreter, "\"{{fooNmbWrong:string}}\"", "1234"),false);

        assert_eq!(response_value(&mut interpreter, "\"{{fooArr:array}}\"", "[1,2,3]"),true);

        assert_eq!(response_value(&mut interpreter, "\"{{fooArrWrong:object}}\"", "{\"foo\":123}"),true);

        let variables = interpreter.variables();

        let bar_str = variables.get("barStr").unwrap();
        assert_eq!(bar_str.value,Value::Null);

        let foo_nmb_wrong = variables.get("fooNmbWrong").unwrap();
        assert_eq!(foo_nmb_wrong.value,Value::String(String::from("")));

        let foo_nmb = variables.get("fooNmb").unwrap();
        assert_eq!(foo_nmb.value,Value::Number(Number::from(1234)));
    }

    #[test]
    pub fn test_response_value_with_comparison(){
        let mut interpreter = make_interpreter_with_pre_variable();

        assert_eq!(response_value(&mut interpreter, "\"{{barVal:number&& barVal==bar}}\"", "12"),true);
        assert_eq!(response_value(&mut interpreter, "\"{{barVal1:number&& barVal1<bar}}\"", "11"),true);
        assert_eq!(response_value(&mut interpreter, "\"{{barVal2:number&& barVal2>bar}}\"", "11"),false);
        assert_eq!(response_value(&mut interpreter, "\"{{barVal3:number&& barVal3<bar && barVal2< barVal3}}\"", "10"),false);

        assert_eq!(response_value(&mut interpreter, "\"{{fooVal1:string&& fooVal1==foo}}\"", "\"Foo\""),true);
        assert_eq!(response_value(&mut interpreter, "\"{{fooVal2:string&& (fooVal2!=foo|| fooVal2==fooVal1)}}\"", "\"Foo\""),true);
        assert_eq!(response_value(&mut interpreter, "\"{{fooVal3:string&& fooVal3!=foo}}\"", "\"Foo\""),false);

        assert_eq!(response_value(&mut interpreter, "\"{{arrVal:array&& arrVal.length==3}}\"", "[1,2,3]"),true);
        assert_eq!(response_value(&mut interpreter, "\"{{arrVal2:array&& arrVal2.length==arrVal.length}}\"", "[1,2,3,4]"),false);

        assert_eq!(response_value(&mut interpreter, "\"{{objVal1:object&& objVal1.bar==4}}\"", "{\"bar\":4}"),true);
        assert_eq!(response_value(&mut interpreter, "\"{{objVal2:object&& objVal1.bar==objVal2.foo}}\"", "{\"foo\":5}"),false);
    }

    fn response_body(body:&str)->Map<String,Value>{
        let json_val:Value = from_str(body).unwrap();

        json_val.as_object().unwrap().to_owned()
    }

    #[test]
    pub fn test_response_body_with_no_parameters(){
        let mut interpreter = Interpreter::new(None);

        let passed = interpreter.parse_response_body(
            response_body(r#"{}"#),
            response_body(r#"{}"#)
        );

        assert_eq!(passed,true);
    }

    #[test]
    pub fn test_response_body_without_parameter_on_response(){
        let mut interpreter = Interpreter::new(None);

        let passed = interpreter.parse_response_body(
            response_body(r#"{
                "foo":"{{number}}"
            }"#),
            response_body(r#"{}"#)
        );

        assert_eq!(passed,false);

        // Nested
        let passed = interpreter.parse_response_body(
            response_body(r#"{
                "foo":{
                    "bar":"{{number}}"
                }
            }"#),
            response_body(r#"{}"#)
        );

        assert_eq!(passed,false);
    }

    #[test]
    pub fn test_response_body_without_parameter_on_test_case(){
        let mut interpreter = Interpreter::new(None);

        let passed = interpreter.parse_response_body(
            response_body(r#"{}"#),
            response_body(r#"{
                "foo": 123
            }"#)
        );

        assert_eq!(passed,true);
    }

    #[test]
    pub fn test_response_body_with_pattern(){
        let mut interpreter = Interpreter::new(None);

        let passed = interpreter.parse_response_body(
            response_body(r#"{
                "foo": 123
            }"#),
            response_body(r#"{
                "foo": 123
            }"#)
        );

        assert_eq!(passed,true);

        let passed = interpreter.parse_response_body(
            response_body(r#"{
                "foo": 1233
            }"#),
            response_body(r#"{
                "foo": 123
            }"#)
        );

        assert_eq!(passed,false);

        // String
        let passed = interpreter.parse_response_body(
            response_body(r#"{
                "foo": "Foo"
            }"#),
            response_body(r#"{
                "foo": "Foo"
            }"#)
        );

        assert_eq!(passed,true);


        let passed = interpreter.parse_response_body(
            response_body(r#"{
                "foo": "Foo"
            }"#),
            response_body(r#"{
                "foo": "Bar"
            }"#)
        );

        assert_eq!(passed,false);


        // Different types
        let passed = interpreter.parse_response_body(
            response_body(r#"{
                "foo": "Foo"
            }"#),
            response_body(r#"{
                "foo": 123
            }"#)
        );

        assert_eq!(passed,false);
    }

    #[test]
    pub fn test_response_body_with_nested_patterns(){
        let mut interpreter = Interpreter::new(None);

        let passed = interpreter.parse_response_body(
            response_body(r#"{
                "foo": {
                    "bar": "{{number}}"
                }
            }"#),
            response_body(r#"{
                "foo": {
                    "bar": 123
                }
            }"#)
        );

        assert_eq!(passed,true);

        let passed = interpreter.parse_response_body(
            response_body(r#"{
                "foo": {
                    "bar": "{{number}}"
                }
            }"#),
            response_body(r#"{
                "foo": {
                    "bar": "3233"
                }
            }"#)
        );

        assert_eq!(passed,false);
    }

    #[test]
    pub fn test_response_body_with_array(){
        let mut interpreter = Interpreter::new(None);

        let passed = interpreter.parse_response_body(
            response_body(r#"{
                "foo": {
                    "bar": [
                        {
                            "foo":"{{number}}"
                        }
                    ]
                }
            }"#),
            response_body(r#"{
                "foo": {
                    "bar": [
                        {
                            "foo": 123
                        }
                    ]
                }
            }"#)
        );

        assert_eq!(passed,true);

        let passed = interpreter.parse_response_body(
            response_body(r#"{
                "foo": {
                    "bar": [
                        {
                            "foo":"{{number}}"
                        }
                    ]
                }
            }"#),
            response_body(r#"{
                "foo": {
                    "bar": [
                        {
                            "foo": "asasa"
                        }
                    ]
                }
            }"#)
        );

        assert_eq!(passed,false);

        let passed = interpreter.parse_response_body(
            response_body(r#"{
                "foo": {
                    "bar": [
                        {
                            "foo":"{{number}}"
                        }
                    ]
                }
            }"#),
            response_body(r#"{
                "foo": {
                    "bar": [
                        {
                            "foo": 123
                        },
                        {
                            "foo": "asass"
                        }
                    ]
                }
            }"#)
        );

        assert_eq!(passed,false);
    }
}
