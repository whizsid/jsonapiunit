# JSONAPIUnit

---
![AUR version](https://img.shields.io/aur/version/jsonapiunit)
![GitHub](https://img.shields.io/github/license/whizsid/jsonapiunit)
![Travis CI](https://travis-ci.org/whizsid/jsonapiunit.svg?branch=master)
---

A framework for unit testing your JSON REST APIs. Write test cases in typescript like language.

## Contents

- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
    - [Defining Default Request Behaviours](#defining-default-request-behaviours)
    - [Base URL](#base-url)
    - [Under Proxy](#under-proxy)
    - [Select Test Cases](#select-test-cases)
    - [Pre Variables](#pre-variables)
- [Example](#example)
    - [Example Test Case](#example-test-case)
    - [Example Output](#example-output)
    - [Example Project](#example-project)
- [Request](#request)
    - [User Inputs](#user-inputs)
    - [Using Variables](#using-variables)
- [Validating Response](#validating-response)
    - [Type Checking](#type-checking)
    - [Variable Declaration](#variable-declaration)
    - [Compare With Previously Created Variables](#comparisons)
    - [Array Validation](#array-validation)
    - [Nested Objects Validation](#nested-object-validation)
    - [Advanced Validations](#advanced-validations)
- [Todo](#todo)
- [Contributing](#contributing)

## Installation

### Ubuntu

Download the ubuntu build from [here](https://github.com/whizsid/jsonapiunit/releases/download/0.1.3/jsonapiunit_0.1.3_amd64.deb) and install it using below command.

```
$ sudo dpkg -i ./jsonapiunit_0.1.3_amd64.deb

```

### Arch Linux

Clone the AUR and install.

```
git clone https://aur.archlinux.org/jsonapiunit.git
cd jsonapiunit
makepkg -si

```

### Other Distros

Download pre-built binary from [here](https://github.com/whizsid/jsonapiunit/releases/download/0.1.3/jsonapiunit) and run it.

## Usage

Download binaries or build the JSONAPIUnit on your PC and run `jsonapiunit` on your terminal.

## Configuration

JSONAPIUnit looks for a `jsonapiunit.jsonc` configuration file in the project root folder.

### Defining Default Request Behaviours

Users can define default request headers or method by placing a property named `default` in their config file. This property is optional.

```jsonc
// jsonapiunit.jsonc
{
    // ...
    "default":{
        // Optional
        "method":"GET",
        // Optional
        "headers": {
            "Accept": "application/json"
        }
    }
}


```

### Base URL

`baseUrl` property allow users to define their base URL. After that users can define a relative URL in their test cases. This property is optional. JSONAPIUnit using [RFC3986](https://tools.ietf.org/html/rfc3986#section-5.2) standard to combine the URLs. 

```jsonc
// jsonapiunit.jsonc
{
    // ...
    "baseUrl":"http://127.0.0.1:8000/api/"
}

```


```jsonc
// In the test case
{
    // ...
    "url":"user/login"
}

```

### Under Proxy

A proxy to use for outgoing https requests. Users can define their proxy settings by adding a new property named `proxy` to their config file.

```jsonc
// jsonapiunit.jsonc
{
    "proxy":{
        // This property is required
        "uri": "proxy uri",
        // These properties are optional
        "username": "username to the proxy server",
        "password": "proxy server password"
    }
}

```

### Select test cases

Users can provide their own file path pattern for tracking test cases. By default JSONAPIUnit will tracking the `apiTest/*.jsonc` files.

```jsonc
// jsonapiunit.jsonc
{
    // ...
    "files": "tests/*.json"
}

```

### Pre Variables

Users can provide variables to use in test cases. These variables creating on initialization.

```jsonc
// jsonapiunit.jsonc
{
    // ...
    "preVariables":{
        "username":"murali"
    }
}

```

## Example

### Example Test Case

This is a example test case for JSONAPIUnit. 

```jsonc
{
    "url":"http://127.0.0.1:8000/api/user/login",
    // This property is optional if you 
    // provided a default request method 
    // in config file
    "method":"POST",
    "request":{
        "body":{
            "username":"dev",
            // Prompt user to insert a password
            // to send with the request
            // and storing it in a new variable
            // named `password`.
            // You can access this variable in
            // other test cases
            "password":"{{ > password: string }}"
        },
        // This property is optional if you 
        // provided a default request method 
        // in config file
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

### Example Output


This is a sample output of JSONAPIUnit.

```
STARTED : apiTest/b_userDetails.jsonc
PASSED : Assertion: HTTP_STATUS_200, Value: HTTP_STATUS_200
FAILED : Assertion: "{{ limit:number }}", Value: Not Supplied
PASSED : Assertion: "{{string}}", Value: "Muththaiya Muralitharan"
PASSED : Assertion: true, Value: true
FAILED : Assertion: "{{usage: number && usage<=limit}}", Value: Not Supplied
PASSED : Assertion: "{{string}}", Value: "murali"
FAILED TEST CASE : Name: apiTest/b_userDetails.jsonc, Reason: "Some assertion(s) failed.", Assertions: 5, Fails: 2, TotAssertions: 13, TotFails: 2

```

### Example Project

Users can run working example by executing following commands.

```bash
$ cargo build --all
$ cd example
$ cargo run
// Open an another terminal in current directory and run bellow command parallely.
$ ../target/debug/json-api-tester

```

## Request

JSONAPIUnit enable users to create dynamic requests based on previously requests.

Normal request:-

```jsonc
{
    // ...
    "request":{
        "body":{
            "username":"my-user"
        },
        "headers":{
            "Accept":"application/json"
        }
    }
}

```

### User Inputs

Users can insert their inputs before sending the request.

```jsonc
// Test Case
{
    // ...
    "request":{
        // ...
        "body":{
            "password":"{{> password: string}}"
        }
    }
}

```

### Using Variables

Sending previously created variables in request body or request headers.

```jsonc
// Test Case
{
    "request":{
        "body":{
            // catId is a previously created variable
            "id":"{{catId}}"
        },
        "headers":{
            // token is also a previously created variable
            "Authorization": "Bearer {{token}}"
        }
    }
}

```

## Validating Responce

### Type Checking

Check the exact type of response data. Available types are `string`,`number`,`null`,`any`,`array` and `object`.

Example:-

```jsonc
// Test Case

{
    // ...
    "response":{
        // ...
        "body":{
            "name" : "{{string}}",
            "limit" : "{{number}}",
            "nick_name" : "{{string|null}}"
        }
    }
}

```

JSONAPIUnit currently not supporting to use multiple conflicting types. Ex:- `number|string`, `object|array`

### Variable Declaration

Assign JSON value to a new variable. 

```jsonc
// Test Case
{
    // ...
    "response":{
        // ...
        "body":{
            "name": "{{name:string}}",
            "limit": "{{ limit: number }}",
            "nick_name": "{{ nickName: string|null }}"
        }
    }
}

```

You can use these variables to validate other assertions on same test case or another test cases.

### Comparisons

Compare data with previously created variables or other hard coded values.

```jsonc
// Test Case
{
    // ...
    "response":{
        // ...
        "body":{
            // mileageLimit is a previously created variable
            "mileage":"{{mileage:number && mileage <= mileageLimit}}",
            "billCount": "{{billCount:number && billCount > 0}}",
            "name": "{{name: string && name == 'Abrahm'}}"
        }
    }
}

```

### Array Validations

Validating all elements of an array.

```jsonc 

// Test Case
{
    // ...
    "response":{
        // ...
        "body":{
            "categories":[
                // JSONAPIUnit will matching all elements
                // of this array with bellow type.
                {
                    "id":"{{number}}",
                    "name":"{{string}}"
                }
            ]
        }
    }
}


```

### Nested Object Validation

Validating the nested elements of response.

```jsonc

// Test Case

{
    // ...
    "response":{
        // ...
        "body":{
            // ...
            "product":{
                // ...
                "category":{
                    // ...
                    "name":"{{string}}"
                }
            }
        }
    }
}
```

### Advanced Validations

Users can use JS functions in comparisons to validate.

```jsonc
// Test Case
{
    // ...
    "response":{
        // ...
        "body":{
            "categories":"{{categories:array && categories.length >100}}"
        }
    }
}

```

## Todo

- Response Header Validation.
- Passing config variables from command line.

## Contributing

All PRs and issues are welcome. And also stars are welcome.

Please format and test your codes before sending PRs.