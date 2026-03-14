const PREC = {
  CALL: 6,
  UNARY: 5,
  MULTIPLICATIVE: 4,
  ADDITIVE: 3,
  COMPARISON: 2,
};

module.exports = grammar({
  name: "kira",

  extras: ($) => [
    /\s/,
    $.comment,
  ],

  rules: {
    source_file: ($) => seq(
      optional($.platforms_metadata),
      repeat($.import_statement),
      repeat($.function_definition),
    ),

    comment: () => token(seq("//", /.*/)),

    platforms_metadata: ($) => seq(
      "#platforms",
      "{",
      repeat($.platform_group),
      "}",
    ),

    platform_group: ($) => seq(
      field("name", $.identifier),
      "=",
      "[",
      optional(seq(
        $.identifier,
        repeat(seq(",", $.identifier)),
      )),
      "]",
      ";",
    ),

    import_statement: ($) => seq(
      "import",
      $.path,
      ";",
    ),

    function_definition: ($) => seq(
      repeat($.attribute),
      optional($.execution_mode),
      "func",
      field("name", $.identifier),
      field("parameters", $.parameter_list),
      optional(seq("->", field("return_type", $.type))),
      field("body", $.block),
    ),

    execution_mode: () => choice("native", "runtime", "script", "auto"),

    attribute: ($) => seq(
      "@",
      field("name", $.attribute_identifier),
      optional($.attribute_arguments),
    ),

    attribute_arguments: ($) => seq(
      "(",
      optional(seq(
        $.identifier,
        repeat(seq(",", $.identifier)),
      )),
      ")",
    ),

    parameter_list: ($) => seq(
      "(",
      optional(seq(
        $.parameter,
        repeat(seq(",", $.parameter)),
      )),
      ")",
    ),

    parameter: ($) => seq(
      field("name", $.identifier),
      ":",
      field("type", $.type),
    ),

    type: ($) => choice(
      $.builtin_type,
      $.identifier,
    ),

    builtin_type: () => choice("int", "float", "bool", "string"),

    block: ($) => seq(
      "{",
      repeat($.statement),
      "}",
    ),

    statement: ($) => choice(
      $.if_statement,
      $.let_statement,
      $.return_statement,
      $.expression_statement,
    ),

    if_statement: ($) => seq(
      "if",
      field("condition", $.expression),
      field("consequence", $.block),
      optional(seq("else", field("alternative", $.block))),
    ),

    let_statement: ($) => seq(
      "let",
      field("name", $.identifier),
      optional(seq(":", field("type", $.type))),
      "=",
      field("value", $.expression),
      ";",
    ),

    return_statement: ($) => seq(
      "return",
      field("value", $.expression),
      ";",
    ),

    expression_statement: ($) => seq(
      $.expression,
      ";",
    ),

    expression: ($) => choice(
      $.binary_expression,
      $.unary_expression,
      $.call_expression,
      $.cast_expression,
      $.path,
      $.literal,
      seq("(", $.expression, ")"),
    ),

    call_expression: ($) => prec(PREC.CALL, seq(
      field("function", choice($.path, seq("(", $.expression, ")"))),
      field("arguments", $.argument_list),
    )),

    argument_list: ($) => seq(
      "(",
      optional(seq(
        $.expression,
        repeat(seq(",", $.expression)),
      )),
      ")",
    ),

    cast_expression: ($) => prec(PREC.CALL, seq(
      field("target", $.builtin_type),
      "(",
      field("value", $.expression),
      ")",
    )),

    unary_expression: ($) => prec.right(PREC.UNARY, seq(
      field("operator", "-"),
      field("operand", $.expression),
    )),

    binary_expression: ($) => choice(
      ...[
        ["*", PREC.MULTIPLICATIVE],
        ["/", PREC.MULTIPLICATIVE],
        ["%", PREC.MULTIPLICATIVE],
        ["+", PREC.ADDITIVE],
        ["-", PREC.ADDITIVE],
        ["==", PREC.COMPARISON],
        ["!=", PREC.COMPARISON],
        ["<=", PREC.COMPARISON],
        [">=", PREC.COMPARISON],
      ].map(([operator, precedence]) =>
        prec.left(precedence, seq(
          field("left", $.expression),
          field("operator", operator),
          field("right", $.expression),
        )),
      ),
    ),

    path: ($) => seq(
      $.identifier,
      repeat(seq(".", $.member_identifier)),
    ),

    literal: ($) => choice(
      $.true,
      $.false,
      $.float_literal,
      $.integer_literal,
      $.string_literal,
    ),

    true: () => "true",
    false: () => "false",

    integer_literal: () => token(/[0-9]+/),
    float_literal: () => token(/[0-9]+\.[0-9]+/),

    string_literal: ($) => seq(
      "\"",
      repeat(choice(
        $.escape_sequence,
        token.immediate(prec(1, /[^"\\\n]+/)),
      )),
      "\"",
    ),

    escape_sequence: () => token.immediate(seq(
      "\\",
      choice('"', "\\", "n", "t", "r"),
    )),

    identifier: () => token(prec(-1, /[a-zA-Z_][a-zA-Z0-9_]*/)),
    member_identifier: () => token(prec(-1, /[a-zA-Z_][a-zA-Z0-9_]*/)),
    attribute_identifier: () => token(/[A-Z][a-zA-Z0-9_]*/),
  },
});
