# JSON API Tester

A framework for unit testing your JSON REST APIs.

## Example

You can run our working example by executing following commands.

```bash
$ cargo build --all
$ cd example
$ cargo run
// Open an another terminal in current directory and run bellow command parallely.
$ ../target/debug/json-api-tester

```

You can see an example test case below. 

```jsonc
{
    "url":"http://127.0.0.1:8000/api/user/login",
    "method":"POST",
    "request":{
        "body":{
            "username":"dev",
            // Asking a user input during test
            // You can access this value as a variable
            "password":"{{ > password }}"
        },
        "headers":{
            "Accept": "application/json",
            "Content-Type": "application/json"
        }
    },
    "response": {
        "body":{
            // Creating a new variable named token and checking the type
            "token":"{{ token: string }}",
            // Checking the type without creating a new variable
            "mileage": "{{ number }}",
            "limit": "{{ limit:number }}",
            // Save the value to a variable named anotherLimit and 
            // compare it with previously created variable
            "anotherLimit": "{{ anotherLimit:number && (anotherLimit >= limit) }}"
        }
    }
}

```

This is the output of our example project.

```
STARTED : example/apiTest/a_loginTest.jsonc
INPUT : password?
hbh
PASSED : Assertion: "{{limit:number}}" , Fails:0, Assertions:1, TotFails:0, TotAssertions:1
PASSED : Assertion: "{{string}}" , Fails:0, Assertions:2, TotFails:0, TotAssertions:2
PASSED : Assertion: true , Fails:0, Assertions:3, TotFails:0, TotAssertions:3
PASSED : Assertion: "{{token:string}}" , Fails:0, Assertions:4, TotFails:0, TotAssertions:4
PASSED : Assertion: "{{usage:number&&usage<=limit}}" , Fails:0, Assertions:5, TotFails:0, TotAssertio
ns:5
PASSED : Assertion: "{{string}}" , Fails:0, Assertions:6, TotFails:0, TotAssertions:6
PASSED TEST CASE : Name: example/apiTest/a_loginTest.jsonc
STARTED : example/apiTest/b_userDetails.jsonc
PASSED : Assertion: "{{string}}" , Fails:0, Assertions:1, TotFails:0, TotAssertions:7
PASSED : Assertion: true , Fails:0, Assertions:2, TotFails:0, TotAssertions:8
PASSED : Assertion: "{{string}}" , Fails:0, Assertions:3, TotFails:0, TotAssertions:9
FAILED TEST CASE : Name: example/apiTest/b_userDetails.jsonc, Reason: Some assertion(s) failed.
STARTED : example/apiTest/c_categories.jsonc
STARTED : example/apiTest/a_loginTest.jsonc
INPUT : password?
jnj
PASSED : Assertion: "{{limit:number}}" , Fails:0, Assertions:1, TotFails:0, TotAssertions:1
PASSED : Assertion: "{{string}}" , Fails:0, Assertions:2, TotFails:0, TotAssertions:2
PASSED : Assertion: true , Fails:0, Assertions:3, TotFails:0, TotAssertions:3
PASSED : Assertion: "{{token:string}}" , Fails:0, Assertions:4, TotFails:0, TotAssertions:4
PASSED : Assertion: "{{usage:number&&usage<=limit}}" , Fails:0, Assertions:5, TotFails:0, TotAssertio
ns:5
PASSED : Assertion: "{{string}}" , Fails:0, Assertions:6, TotFails:0, TotAssertions:6
PASSED TEST CASE : Name: example/apiTest/a_loginTest.jsonc
STARTED : example/apiTest/b_userDetails.jsonc
FAILED : Assertion: "{{ limit:number }}" , Fails:0, Assertions:0, TotFails:0, TotAssertions:6
PASSED : Assertion: "{{string}}" , Fails:1, Assertions:2, TotFails:1, TotAssertions:8
PASSED : Assertion: true , Fails:1, Assertions:3, TotFails:1, TotAssertions:9
FAILED : Assertion: "{{usage: number && usage<=limit}}" , Fails:1, Assertions:3, TotFails:1, TotAssertions:9
PASSED : Assertion: "{{string}}" , Fails:2, Assertions:5, TotFails:2, TotAssertions:11
FAILED TEST CASE : Name: example/apiTest/b_userDetails.jsonc, Reason: Some assertion(s) failed.
STARTED : example/apiTest/c_categories.jsonc
PASSED : Assertion: "{{number}}" , Fails:0, Assertions:1, TotFails:2, TotAssertions:12
PASSED : Assertion: "{{number}}" , Fails:0, Assertions:2, TotFails:2, TotAssertions:13
PASSED : Assertion: "{{string}}" , Fails:0, Assertions:3, TotFails:2, TotAssertions:14
PASSED : Assertion: "{{number}}" , Fails:0, Assertions:4, TotFails:2, TotAssertions:15
PASSED : Assertion: "{{number}}" , Fails:0, Assertions:5, TotFails:2, TotAssertions:16
PASSED : Assertion: "{{string}}" , Fails:0, Assertions:6, TotFails:2, TotAssertions:17
PASSED : Assertion: "{{number}}" , Fails:0, Assertions:7, TotFails:2, TotAssertions:18
PASSED : Assertion: "{{number}}" , Fails:0, Assertions:8, TotFails:2, TotAssertions:19
PASSED : Assertion: "{{string}}" , Fails:0, Assertions:9, TotFails:2, TotAssertions:20
PASSED : Assertion: true , Fails:0, Assertions:10, TotFails:2, TotAssertions:21
PASSED TEST CASE : Name: example/apiTest/c_categories.jsonc
Error: "Some test cases failed."

```

## Type Checking

You can check the exact type of your data. Available types are `string`,`number`,`null`,`any`,`array`,`object`.

Example:-

```jsonc
{
    "body":{

        "name" : "{{string}}",
        "limit" : "{{number}}",
        "nick_name" : "{{string|null}}"
    }
}

```

## Variable Creation

Create variables with the data coming from the REST API.

```jsonc
{
    "body":{
        "name": "{{name:string}}",
        "limit": "{{ limit: number }}",
        "nick_name": "{{ nickName: string|null }}"
    }
}

```

You can use these variables to validate next test cases.

## Comparisons

Compare data with previously created variables or other values.

```jsonc
{
    "body":{
        // mileageLimit is a previously created variable
        "mileage":"{{mileage:number && mileage <= mileageLimit}}",
        "billCount": "{{billCount:number && billCount > 0}}",
        "name": "{{name: string && name == 'Abrahm'}}"
    }
}

```

## User Inputs

Requesting user inputs before sending the data to the API

```jsonc 

{
    "params":{
        "username" : "dev",
        "password" : "{{ > password:string}}"
    }
}

```

A new variable creating with the user input after user entered the value. You can use this variable for test cases.