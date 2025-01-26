# choreo validate

TODO: only apply to structs and enums


1. rules are predefined in a registry 
    - per rule
        - handler
        - support types
        - any type
        - option only
2. compilation -> per field -> based on attributes it is determined which rules are applied to the field
    - No rule
    - Single or Multiple rules
    - we allow the rules to be defined as a single parameter, multiple parameters, etc etc
3. per rule:
    compilation errors
    - some rules are dependent on the type
    - some parameters need to match integer for numbers, etc
    - ...
    runtime code expansion
    - option has a special expansion logic as we need to check for Some(...) within the expansion code
    - we want to expand per rule and per field


# TODO

[draft-bhutton-json-schema-validation-](https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00)

## openapi types
- type: boolean, object, array, number, string, integer (non float)
- enum: array of unique elements
- const: 

## validation (number/integer)

OK - multipleof: float divide number by the value provide in the validation
OK - maximum: <= (ge)
OK - exclusiveMaximum: < (gt)
OK - minimum: >= (le)
OK - exclusiveMinimum: > (lt)

## validation (string)

OK - maxLength
OK - minLength
OK - pattern

## arrays

OK - maxItems: value of keyword > 0, valid if <= to the value of keyword
OK - minItems: value of keyword > 0, valid if <= to the value of keyword
- uniqueItems: value of keyword = bool
??- maxContains: ??
??- minContains: ??

## objects

- maxProperties: value of keyword > 0, <=
- minProperties: value of keyword > 0, >=
- required

## complex

- depreciated
- immutable
- oneOf