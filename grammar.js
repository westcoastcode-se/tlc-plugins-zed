// A small grammar for opening and running TLC files in Zed.
// It tokenizes the file and keeps function and type names as real nodes.

const BUILTIN_TYPES = [
  "bool",
  "void",
  "i8",
  "u8",
  "i16",
  "u16",
  "i32",
  "u32",
  "i64",
  "u64",
  "f32",
  "f64",
];

const KEYWORDS = [
  "var",
  "const",
  "struct",
  "enum",
  "union",
  "trait",
  "if",
  "else",
  "while",
  "for",
  "do",
  "switch",
  "case",
  "default",
  "return",
  "break",
  "continue",
  "defer",
  "import",
  "package",
  "export",
  "extern",
  "test",
  "assert",
  "sizeof",
  "typeof",
  "true",
  "false",
  "null",
];

module.exports = grammar({
  name: "tlc",

  extras: ($) => [/\s/, $.comment],

  word: ($) => $.identifier,

  rules: {
    source_file: ($) => repeat($._token),

    _token: ($) =>
      choice(
        $.function_definition,
        $.type_definition,
        $.string,
        $.character,
        $.number,
        $.builtin_type,
        $.keyword,
        $.identifier,
        $.operator,
        $.punctuation,
      ),

    function_definition: ($) =>
      seq("func", field("name", $.identifier)),

    type_definition: ($) => seq("type", field("name", $.identifier)),

    comment: ($) =>
      token(
        choice(
          seq("//", /.*/),
          seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/"),
        ),
      ),

    string: ($) =>
      token(seq('"', repeat(choice(/[^"\\\n]/, /\\./)), '"')),

    character: ($) =>
      token(seq("'", choice(/[^'\\\n]/, /\\./), "'")),

    number: ($) => token(/[0-9][0-9a-zA-Z_]*/),

    builtin_type: ($) => choice(...BUILTIN_TYPES),

    keyword: ($) => choice(...KEYWORDS),

    identifier: ($) => /[A-Za-z_][A-Za-z0-9_]*/,

    operator: ($) =>
      choice(
        ":=",
        "==",
        "!=",
        "<=",
        ">=",
        "&&",
        "||",
        "<<",
        ">>",
        "->",
        "++",
        "--",
        "+=",
        "-=",
        "*=",
        "/=",
        "+",
        "-",
        "*",
        "/",
        "%",
        "=",
        "<",
        ">",
        "!",
        "&",
        "|",
        "^",
        "~",
        "?",
      ),

    punctuation: ($) => choice("(", ")", "{", "}", "[", "]", ",", ";", ".", ":"),
  },
});
