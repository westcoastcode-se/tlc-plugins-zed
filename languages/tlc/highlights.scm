(comment) @comment

(string) @string
(character) @string

(number) @number

(keyword) @keyword
(builtin_type) @type.builtin

((keyword) @boolean
  (#any-of? @boolean "true" "false"))

((keyword) @constant
  (#eq? @constant "null"))

(identifier) @variable

(function_definition
  "func" @keyword
  type: (identifier) @type
  name: (identifier) @function)

(function_definition
  "func" @keyword
  name: (identifier) @function)

(type_definition
  "type" @keyword
  name: (identifier) @type)

(operator) @operator
(punctuation) @punctuation
