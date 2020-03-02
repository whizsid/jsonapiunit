# JSON API Tester

A framework for unit testing your JSON REST APIs.

## Example

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