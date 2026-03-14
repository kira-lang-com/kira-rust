#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 158
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 87
#define ALIAS_COUNT 0
#define TOKEN_COUNT 48
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 16
#define MAX_ALIAS_SEQUENCE_LENGTH 8
#define PRODUCTION_ID_COUNT 20

enum ts_symbol_identifiers {
  sym_comment = 1,
  anon_sym_POUNDplatforms = 2,
  anon_sym_LBRACE = 3,
  anon_sym_RBRACE = 4,
  anon_sym_EQ = 5,
  anon_sym_LBRACK = 6,
  anon_sym_COMMA = 7,
  anon_sym_RBRACK = 8,
  anon_sym_SEMI = 9,
  anon_sym_import = 10,
  anon_sym_func = 11,
  anon_sym_DASH_GT = 12,
  anon_sym_native = 13,
  anon_sym_runtime = 14,
  anon_sym_script = 15,
  anon_sym_auto = 16,
  anon_sym_AT = 17,
  anon_sym_LPAREN = 18,
  anon_sym_RPAREN = 19,
  anon_sym_COLON = 20,
  anon_sym_int = 21,
  anon_sym_float = 22,
  anon_sym_bool = 23,
  anon_sym_string = 24,
  anon_sym_if = 25,
  anon_sym_else = 26,
  anon_sym_let = 27,
  anon_sym_return = 28,
  anon_sym_DASH = 29,
  anon_sym_STAR = 30,
  anon_sym_SLASH = 31,
  anon_sym_PERCENT = 32,
  anon_sym_PLUS = 33,
  anon_sym_EQ_EQ = 34,
  anon_sym_BANG_EQ = 35,
  anon_sym_LT_EQ = 36,
  anon_sym_GT_EQ = 37,
  anon_sym_DOT = 38,
  sym_true = 39,
  sym_false = 40,
  sym_integer_literal = 41,
  sym_float_literal = 42,
  anon_sym_DQUOTE = 43,
  aux_sym_string_literal_token1 = 44,
  sym_escape_sequence = 45,
  aux_sym_identifier_token1 = 46,
  sym_attribute_identifier = 47,
  sym_source_file = 48,
  sym_platforms_metadata = 49,
  sym_platform_group = 50,
  sym_import_statement = 51,
  sym_function_definition = 52,
  sym_execution_mode = 53,
  sym_attribute = 54,
  sym_attribute_arguments = 55,
  sym_parameter_list = 56,
  sym_parameter = 57,
  sym_type = 58,
  sym_builtin_type = 59,
  sym_block = 60,
  sym_statement = 61,
  sym_if_statement = 62,
  sym_let_statement = 63,
  sym_return_statement = 64,
  sym_expression_statement = 65,
  sym_expression = 66,
  sym_call_expression = 67,
  sym_argument_list = 68,
  sym_cast_expression = 69,
  sym_unary_expression = 70,
  sym_binary_expression = 71,
  sym_path = 72,
  sym_literal = 73,
  sym_string_literal = 74,
  sym_identifier = 75,
  sym_member_identifier = 76,
  aux_sym_source_file_repeat1 = 77,
  aux_sym_source_file_repeat2 = 78,
  aux_sym_platforms_metadata_repeat1 = 79,
  aux_sym_platform_group_repeat1 = 80,
  aux_sym_function_definition_repeat1 = 81,
  aux_sym_parameter_list_repeat1 = 82,
  aux_sym_block_repeat1 = 83,
  aux_sym_argument_list_repeat1 = 84,
  aux_sym_path_repeat1 = 85,
  aux_sym_string_literal_repeat1 = 86,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [sym_comment] = "comment",
  [anon_sym_POUNDplatforms] = "#platforms",
  [anon_sym_LBRACE] = "{",
  [anon_sym_RBRACE] = "}",
  [anon_sym_EQ] = "=",
  [anon_sym_LBRACK] = "[",
  [anon_sym_COMMA] = ",",
  [anon_sym_RBRACK] = "]",
  [anon_sym_SEMI] = ";",
  [anon_sym_import] = "import",
  [anon_sym_func] = "func",
  [anon_sym_DASH_GT] = "->",
  [anon_sym_native] = "native",
  [anon_sym_runtime] = "runtime",
  [anon_sym_script] = "script",
  [anon_sym_auto] = "auto",
  [anon_sym_AT] = "@",
  [anon_sym_LPAREN] = "(",
  [anon_sym_RPAREN] = ")",
  [anon_sym_COLON] = ":",
  [anon_sym_int] = "int",
  [anon_sym_float] = "float",
  [anon_sym_bool] = "bool",
  [anon_sym_string] = "string",
  [anon_sym_if] = "if",
  [anon_sym_else] = "else",
  [anon_sym_let] = "let",
  [anon_sym_return] = "return",
  [anon_sym_DASH] = "-",
  [anon_sym_STAR] = "*",
  [anon_sym_SLASH] = "/",
  [anon_sym_PERCENT] = "%",
  [anon_sym_PLUS] = "+",
  [anon_sym_EQ_EQ] = "==",
  [anon_sym_BANG_EQ] = "!=",
  [anon_sym_LT_EQ] = "<=",
  [anon_sym_GT_EQ] = ">=",
  [anon_sym_DOT] = ".",
  [sym_true] = "true",
  [sym_false] = "false",
  [sym_integer_literal] = "integer_literal",
  [sym_float_literal] = "float_literal",
  [anon_sym_DQUOTE] = "\"",
  [aux_sym_string_literal_token1] = "string_literal_token1",
  [sym_escape_sequence] = "escape_sequence",
  [aux_sym_identifier_token1] = "identifier_token1",
  [sym_attribute_identifier] = "attribute_identifier",
  [sym_source_file] = "source_file",
  [sym_platforms_metadata] = "platforms_metadata",
  [sym_platform_group] = "platform_group",
  [sym_import_statement] = "import_statement",
  [sym_function_definition] = "function_definition",
  [sym_execution_mode] = "execution_mode",
  [sym_attribute] = "attribute",
  [sym_attribute_arguments] = "attribute_arguments",
  [sym_parameter_list] = "parameter_list",
  [sym_parameter] = "parameter",
  [sym_type] = "type",
  [sym_builtin_type] = "builtin_type",
  [sym_block] = "block",
  [sym_statement] = "statement",
  [sym_if_statement] = "if_statement",
  [sym_let_statement] = "let_statement",
  [sym_return_statement] = "return_statement",
  [sym_expression_statement] = "expression_statement",
  [sym_expression] = "expression",
  [sym_call_expression] = "call_expression",
  [sym_argument_list] = "argument_list",
  [sym_cast_expression] = "cast_expression",
  [sym_unary_expression] = "unary_expression",
  [sym_binary_expression] = "binary_expression",
  [sym_path] = "path",
  [sym_literal] = "literal",
  [sym_string_literal] = "string_literal",
  [sym_identifier] = "identifier",
  [sym_member_identifier] = "member_identifier",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [aux_sym_source_file_repeat2] = "source_file_repeat2",
  [aux_sym_platforms_metadata_repeat1] = "platforms_metadata_repeat1",
  [aux_sym_platform_group_repeat1] = "platform_group_repeat1",
  [aux_sym_function_definition_repeat1] = "function_definition_repeat1",
  [aux_sym_parameter_list_repeat1] = "parameter_list_repeat1",
  [aux_sym_block_repeat1] = "block_repeat1",
  [aux_sym_argument_list_repeat1] = "argument_list_repeat1",
  [aux_sym_path_repeat1] = "path_repeat1",
  [aux_sym_string_literal_repeat1] = "string_literal_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [sym_comment] = sym_comment,
  [anon_sym_POUNDplatforms] = anon_sym_POUNDplatforms,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [anon_sym_EQ] = anon_sym_EQ,
  [anon_sym_LBRACK] = anon_sym_LBRACK,
  [anon_sym_COMMA] = anon_sym_COMMA,
  [anon_sym_RBRACK] = anon_sym_RBRACK,
  [anon_sym_SEMI] = anon_sym_SEMI,
  [anon_sym_import] = anon_sym_import,
  [anon_sym_func] = anon_sym_func,
  [anon_sym_DASH_GT] = anon_sym_DASH_GT,
  [anon_sym_native] = anon_sym_native,
  [anon_sym_runtime] = anon_sym_runtime,
  [anon_sym_script] = anon_sym_script,
  [anon_sym_auto] = anon_sym_auto,
  [anon_sym_AT] = anon_sym_AT,
  [anon_sym_LPAREN] = anon_sym_LPAREN,
  [anon_sym_RPAREN] = anon_sym_RPAREN,
  [anon_sym_COLON] = anon_sym_COLON,
  [anon_sym_int] = anon_sym_int,
  [anon_sym_float] = anon_sym_float,
  [anon_sym_bool] = anon_sym_bool,
  [anon_sym_string] = anon_sym_string,
  [anon_sym_if] = anon_sym_if,
  [anon_sym_else] = anon_sym_else,
  [anon_sym_let] = anon_sym_let,
  [anon_sym_return] = anon_sym_return,
  [anon_sym_DASH] = anon_sym_DASH,
  [anon_sym_STAR] = anon_sym_STAR,
  [anon_sym_SLASH] = anon_sym_SLASH,
  [anon_sym_PERCENT] = anon_sym_PERCENT,
  [anon_sym_PLUS] = anon_sym_PLUS,
  [anon_sym_EQ_EQ] = anon_sym_EQ_EQ,
  [anon_sym_BANG_EQ] = anon_sym_BANG_EQ,
  [anon_sym_LT_EQ] = anon_sym_LT_EQ,
  [anon_sym_GT_EQ] = anon_sym_GT_EQ,
  [anon_sym_DOT] = anon_sym_DOT,
  [sym_true] = sym_true,
  [sym_false] = sym_false,
  [sym_integer_literal] = sym_integer_literal,
  [sym_float_literal] = sym_float_literal,
  [anon_sym_DQUOTE] = anon_sym_DQUOTE,
  [aux_sym_string_literal_token1] = aux_sym_string_literal_token1,
  [sym_escape_sequence] = sym_escape_sequence,
  [aux_sym_identifier_token1] = aux_sym_identifier_token1,
  [sym_attribute_identifier] = sym_attribute_identifier,
  [sym_source_file] = sym_source_file,
  [sym_platforms_metadata] = sym_platforms_metadata,
  [sym_platform_group] = sym_platform_group,
  [sym_import_statement] = sym_import_statement,
  [sym_function_definition] = sym_function_definition,
  [sym_execution_mode] = sym_execution_mode,
  [sym_attribute] = sym_attribute,
  [sym_attribute_arguments] = sym_attribute_arguments,
  [sym_parameter_list] = sym_parameter_list,
  [sym_parameter] = sym_parameter,
  [sym_type] = sym_type,
  [sym_builtin_type] = sym_builtin_type,
  [sym_block] = sym_block,
  [sym_statement] = sym_statement,
  [sym_if_statement] = sym_if_statement,
  [sym_let_statement] = sym_let_statement,
  [sym_return_statement] = sym_return_statement,
  [sym_expression_statement] = sym_expression_statement,
  [sym_expression] = sym_expression,
  [sym_call_expression] = sym_call_expression,
  [sym_argument_list] = sym_argument_list,
  [sym_cast_expression] = sym_cast_expression,
  [sym_unary_expression] = sym_unary_expression,
  [sym_binary_expression] = sym_binary_expression,
  [sym_path] = sym_path,
  [sym_literal] = sym_literal,
  [sym_string_literal] = sym_string_literal,
  [sym_identifier] = sym_identifier,
  [sym_member_identifier] = sym_member_identifier,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [aux_sym_source_file_repeat2] = aux_sym_source_file_repeat2,
  [aux_sym_platforms_metadata_repeat1] = aux_sym_platforms_metadata_repeat1,
  [aux_sym_platform_group_repeat1] = aux_sym_platform_group_repeat1,
  [aux_sym_function_definition_repeat1] = aux_sym_function_definition_repeat1,
  [aux_sym_parameter_list_repeat1] = aux_sym_parameter_list_repeat1,
  [aux_sym_block_repeat1] = aux_sym_block_repeat1,
  [aux_sym_argument_list_repeat1] = aux_sym_argument_list_repeat1,
  [aux_sym_path_repeat1] = aux_sym_path_repeat1,
  [aux_sym_string_literal_repeat1] = aux_sym_string_literal_repeat1,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [sym_comment] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_POUNDplatforms] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COMMA] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SEMI] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_import] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_func] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_native] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_runtime] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_script] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_auto] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_AT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COLON] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_int] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_float] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_bool] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_string] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_if] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_else] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_let] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_return] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_STAR] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SLASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PERCENT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PLUS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BANG_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_GT_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DOT] = {
    .visible = true,
    .named = false,
  },
  [sym_true] = {
    .visible = true,
    .named = true,
  },
  [sym_false] = {
    .visible = true,
    .named = true,
  },
  [sym_integer_literal] = {
    .visible = true,
    .named = true,
  },
  [sym_float_literal] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_DQUOTE] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_string_literal_token1] = {
    .visible = false,
    .named = false,
  },
  [sym_escape_sequence] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_identifier_token1] = {
    .visible = false,
    .named = false,
  },
  [sym_attribute_identifier] = {
    .visible = true,
    .named = true,
  },
  [sym_source_file] = {
    .visible = true,
    .named = true,
  },
  [sym_platforms_metadata] = {
    .visible = true,
    .named = true,
  },
  [sym_platform_group] = {
    .visible = true,
    .named = true,
  },
  [sym_import_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_function_definition] = {
    .visible = true,
    .named = true,
  },
  [sym_execution_mode] = {
    .visible = true,
    .named = true,
  },
  [sym_attribute] = {
    .visible = true,
    .named = true,
  },
  [sym_attribute_arguments] = {
    .visible = true,
    .named = true,
  },
  [sym_parameter_list] = {
    .visible = true,
    .named = true,
  },
  [sym_parameter] = {
    .visible = true,
    .named = true,
  },
  [sym_type] = {
    .visible = true,
    .named = true,
  },
  [sym_builtin_type] = {
    .visible = true,
    .named = true,
  },
  [sym_block] = {
    .visible = true,
    .named = true,
  },
  [sym_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_if_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_let_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_return_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_expression_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_call_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_argument_list] = {
    .visible = true,
    .named = true,
  },
  [sym_cast_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_unary_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_binary_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_path] = {
    .visible = true,
    .named = true,
  },
  [sym_literal] = {
    .visible = true,
    .named = true,
  },
  [sym_string_literal] = {
    .visible = true,
    .named = true,
  },
  [sym_identifier] = {
    .visible = true,
    .named = true,
  },
  [sym_member_identifier] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_source_file_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_source_file_repeat2] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_platforms_metadata_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_platform_group_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_function_definition_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_parameter_list_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_argument_list_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_path_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_literal_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum ts_field_identifiers {
  field_alternative = 1,
  field_arguments = 2,
  field_body = 3,
  field_condition = 4,
  field_consequence = 5,
  field_function = 6,
  field_left = 7,
  field_name = 8,
  field_operand = 9,
  field_operator = 10,
  field_parameters = 11,
  field_return_type = 12,
  field_right = 13,
  field_target = 14,
  field_type = 15,
  field_value = 16,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_alternative] = "alternative",
  [field_arguments] = "arguments",
  [field_body] = "body",
  [field_condition] = "condition",
  [field_consequence] = "consequence",
  [field_function] = "function",
  [field_left] = "left",
  [field_name] = "name",
  [field_operand] = "operand",
  [field_operator] = "operator",
  [field_parameters] = "parameters",
  [field_return_type] = "return_type",
  [field_right] = "right",
  [field_target] = "target",
  [field_type] = "type",
  [field_value] = "value",
};

static const TSFieldMapSlice ts_field_map_slices[PRODUCTION_ID_COUNT] = {
  [1] = {.index = 0, .length = 1},
  [2] = {.index = 1, .length = 3},
  [3] = {.index = 4, .length = 3},
  [4] = {.index = 7, .length = 2},
  [5] = {.index = 9, .length = 2},
  [6] = {.index = 11, .length = 2},
  [7] = {.index = 13, .length = 4},
  [8] = {.index = 17, .length = 3},
  [9] = {.index = 20, .length = 1},
  [10] = {.index = 21, .length = 2},
  [11] = {.index = 23, .length = 1},
  [12] = {.index = 24, .length = 3},
  [13] = {.index = 27, .length = 4},
  [14] = {.index = 31, .length = 4},
  [15] = {.index = 35, .length = 2},
  [16] = {.index = 37, .length = 4},
  [17] = {.index = 41, .length = 3},
  [18] = {.index = 44, .length = 2},
  [19] = {.index = 46, .length = 3},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_name, 1},
  [1] =
    {field_body, 3},
    {field_name, 1},
    {field_parameters, 2},
  [4] =
    {field_body, 4},
    {field_name, 2},
    {field_parameters, 3},
  [7] =
    {field_name, 0},
    {field_type, 2},
  [9] =
    {field_operand, 1},
    {field_operator, 0},
  [11] =
    {field_arguments, 1},
    {field_function, 0},
  [13] =
    {field_body, 5},
    {field_name, 1},
    {field_parameters, 2},
    {field_return_type, 4},
  [17] =
    {field_body, 5},
    {field_name, 3},
    {field_parameters, 4},
  [20] =
    {field_name, 0},
  [21] =
    {field_condition, 1},
    {field_consequence, 2},
  [23] =
    {field_value, 1},
  [24] =
    {field_left, 0},
    {field_operator, 1},
    {field_right, 2},
  [27] =
    {field_body, 6},
    {field_name, 2},
    {field_parameters, 3},
    {field_return_type, 5},
  [31] =
    {field_arguments, 3},
    {field_function, 0},
    {field_function, 1},
    {field_function, 2},
  [35] =
    {field_target, 0},
    {field_value, 2},
  [37] =
    {field_body, 7},
    {field_name, 3},
    {field_parameters, 4},
    {field_return_type, 6},
  [41] =
    {field_alternative, 4},
    {field_condition, 1},
    {field_consequence, 2},
  [44] =
    {field_name, 1},
    {field_value, 3},
  [46] =
    {field_name, 1},
    {field_type, 3},
    {field_value, 5},
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static const uint16_t ts_non_terminal_alias_map[] = {
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 2,
  [4] = 4,
  [5] = 4,
  [6] = 4,
  [7] = 7,
  [8] = 2,
  [9] = 9,
  [10] = 10,
  [11] = 11,
  [12] = 12,
  [13] = 13,
  [14] = 14,
  [15] = 15,
  [16] = 16,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 23,
  [24] = 24,
  [25] = 25,
  [26] = 26,
  [27] = 27,
  [28] = 28,
  [29] = 29,
  [30] = 30,
  [31] = 31,
  [32] = 32,
  [33] = 33,
  [34] = 23,
  [35] = 22,
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 40,
  [41] = 41,
  [42] = 42,
  [43] = 43,
  [44] = 44,
  [45] = 45,
  [46] = 46,
  [47] = 47,
  [48] = 48,
  [49] = 49,
  [50] = 50,
  [51] = 51,
  [52] = 52,
  [53] = 53,
  [54] = 54,
  [55] = 55,
  [56] = 56,
  [57] = 57,
  [58] = 58,
  [59] = 59,
  [60] = 60,
  [61] = 61,
  [62] = 62,
  [63] = 63,
  [64] = 64,
  [65] = 65,
  [66] = 66,
  [67] = 67,
  [68] = 68,
  [69] = 69,
  [70] = 70,
  [71] = 71,
  [72] = 72,
  [73] = 73,
  [74] = 74,
  [75] = 75,
  [76] = 76,
  [77] = 77,
  [78] = 78,
  [79] = 79,
  [80] = 80,
  [81] = 81,
  [82] = 82,
  [83] = 83,
  [84] = 22,
  [85] = 85,
  [86] = 86,
  [87] = 87,
  [88] = 88,
  [89] = 23,
  [90] = 90,
  [91] = 91,
  [92] = 92,
  [93] = 93,
  [94] = 94,
  [95] = 95,
  [96] = 96,
  [97] = 97,
  [98] = 98,
  [99] = 99,
  [100] = 100,
  [101] = 101,
  [102] = 102,
  [103] = 103,
  [104] = 104,
  [105] = 105,
  [106] = 106,
  [107] = 107,
  [108] = 108,
  [109] = 109,
  [110] = 110,
  [111] = 111,
  [112] = 112,
  [113] = 113,
  [114] = 114,
  [115] = 115,
  [116] = 116,
  [117] = 117,
  [118] = 118,
  [119] = 119,
  [120] = 120,
  [121] = 121,
  [122] = 122,
  [123] = 123,
  [124] = 124,
  [125] = 125,
  [126] = 126,
  [127] = 127,
  [128] = 128,
  [129] = 129,
  [130] = 130,
  [131] = 131,
  [132] = 132,
  [133] = 133,
  [134] = 134,
  [135] = 135,
  [136] = 136,
  [137] = 137,
  [138] = 138,
  [139] = 139,
  [140] = 140,
  [141] = 141,
  [142] = 142,
  [143] = 143,
  [144] = 144,
  [145] = 145,
  [146] = 146,
  [147] = 147,
  [148] = 148,
  [149] = 149,
  [150] = 150,
  [151] = 151,
  [152] = 152,
  [153] = 153,
  [154] = 154,
  [155] = 155,
  [156] = 156,
  [157] = 157,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(54);
      ADVANCE_MAP(
        '!', 9,
        '"', 99,
        '#', 33,
        '%', 88,
        '(', 73,
        ')', 74,
        '*', 86,
        '+', 89,
        ',', 62,
        '-', 85,
        '.', 94,
        '/', 87,
        ':', 75,
        ';', 64,
        '<', 10,
        '=', 60,
        '>', 12,
        '@', 72,
        '[', 61,
        '\\', 50,
        ']', 63,
        'a', 159,
        'b', 136,
        'e', 125,
        'f', 106,
        'i', 118,
        'l', 115,
        'n', 104,
        'r', 116,
        's', 109,
        't', 142,
        '{', 57,
        '}', 58,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(52);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(97);
      if (lookahead == '_' ||
          ('c' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      if (('A' <= lookahead && lookahead <= 'Z')) ADVANCE(164);
      END_STATE();
    case 1:
      if (lookahead == '\n') SKIP(5);
      if (lookahead == '"') ADVANCE(99);
      if (lookahead == '/') ADVANCE(101);
      if (lookahead == '\\') ADVANCE(50);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(100);
      if (lookahead != 0) ADVANCE(102);
      END_STATE();
    case 2:
      ADVANCE_MAP(
        '!', 9,
        '"', 99,
        '%', 88,
        '(', 73,
        ')', 74,
        '*', 86,
        '+', 89,
        ',', 62,
        '-', 84,
        '.', 94,
        '/', 87,
        ';', 64,
        '<', 10,
        '=', 11,
        '>', 12,
        'b', 136,
        'f', 107,
        'i', 119,
        'l', 115,
        'r', 117,
        's', 158,
        't', 142,
        '{', 57,
        '}', 58,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(2);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(97);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 3:
      ADVANCE_MAP(
        '"', 99,
        '(', 73,
        ')', 74,
        ',', 62,
        '-', 84,
        '/', 8,
        ':', 75,
        '=', 59,
        'b', 136,
        'f', 107,
        'i', 133,
        's', 158,
        't', 142,
        '{', 57,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(3);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(97);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 4:
      ADVANCE_MAP(
        '"', 99,
        '(', 73,
        '-', 84,
        '/', 8,
        'b', 136,
        'e', 125,
        'f', 107,
        'i', 119,
        'l', 115,
        'r', 117,
        's', 158,
        't', 142,
        '}', 58,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(4);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(97);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 5:
      if (lookahead == '"') ADVANCE(99);
      if (lookahead == '/') ADVANCE(8);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(5);
      END_STATE();
    case 6:
      if (lookahead == ')') ADVANCE(74);
      if (lookahead == '/') ADVANCE(8);
      if (lookahead == ']') ADVANCE(63);
      if (lookahead == '}') ADVANCE(58);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(6);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 7:
      if (lookahead == '-') ADVANCE(13);
      if (lookahead == '/') ADVANCE(8);
      if (lookahead == 'b') ADVANCE(136);
      if (lookahead == 'f') ADVANCE(127);
      if (lookahead == 'i') ADVANCE(133);
      if (lookahead == 's') ADVANCE(158);
      if (lookahead == '{') ADVANCE(57);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(7);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 8:
      if (lookahead == '/') ADVANCE(55);
      END_STATE();
    case 9:
      if (lookahead == '=') ADVANCE(91);
      END_STATE();
    case 10:
      if (lookahead == '=') ADVANCE(92);
      END_STATE();
    case 11:
      if (lookahead == '=') ADVANCE(90);
      END_STATE();
    case 12:
      if (lookahead == '=') ADVANCE(93);
      END_STATE();
    case 13:
      if (lookahead == '>') ADVANCE(67);
      END_STATE();
    case 14:
      if (lookahead == 'a') ADVANCE(40);
      END_STATE();
    case 15:
      if (lookahead == 'a') ADVANCE(43);
      END_STATE();
    case 16:
      if (lookahead == 'c') ADVANCE(66);
      END_STATE();
    case 17:
      if (lookahead == 'c') ADVANCE(37);
      END_STATE();
    case 18:
      if (lookahead == 'e') ADVANCE(68);
      END_STATE();
    case 19:
      if (lookahead == 'e') ADVANCE(69);
      END_STATE();
    case 20:
      if (lookahead == 'f') ADVANCE(31);
      END_STATE();
    case 21:
      if (lookahead == 'i') ADVANCE(49);
      END_STATE();
    case 22:
      if (lookahead == 'i') ADVANCE(34);
      END_STATE();
    case 23:
      if (lookahead == 'i') ADVANCE(27);
      END_STATE();
    case 24:
      if (lookahead == 'l') ADVANCE(14);
      END_STATE();
    case 25:
      if (lookahead == 'm') ADVANCE(39);
      END_STATE();
    case 26:
      if (lookahead == 'm') ADVANCE(35);
      END_STATE();
    case 27:
      if (lookahead == 'm') ADVANCE(19);
      END_STATE();
    case 28:
      if (lookahead == 'n') ADVANCE(16);
      END_STATE();
    case 29:
      if (lookahead == 'n') ADVANCE(45);
      END_STATE();
    case 30:
      if (lookahead == 'o') ADVANCE(71);
      END_STATE();
    case 31:
      if (lookahead == 'o') ADVANCE(36);
      END_STATE();
    case 32:
      if (lookahead == 'o') ADVANCE(38);
      END_STATE();
    case 33:
      if (lookahead == 'p') ADVANCE(24);
      END_STATE();
    case 34:
      if (lookahead == 'p') ADVANCE(42);
      END_STATE();
    case 35:
      if (lookahead == 'p') ADVANCE(32);
      END_STATE();
    case 36:
      if (lookahead == 'r') ADVANCE(25);
      END_STATE();
    case 37:
      if (lookahead == 'r') ADVANCE(22);
      END_STATE();
    case 38:
      if (lookahead == 'r') ADVANCE(41);
      END_STATE();
    case 39:
      if (lookahead == 's') ADVANCE(56);
      END_STATE();
    case 40:
      if (lookahead == 't') ADVANCE(20);
      END_STATE();
    case 41:
      if (lookahead == 't') ADVANCE(65);
      END_STATE();
    case 42:
      if (lookahead == 't') ADVANCE(70);
      END_STATE();
    case 43:
      if (lookahead == 't') ADVANCE(21);
      END_STATE();
    case 44:
      if (lookahead == 't') ADVANCE(30);
      END_STATE();
    case 45:
      if (lookahead == 't') ADVANCE(23);
      END_STATE();
    case 46:
      if (lookahead == 'u') ADVANCE(28);
      END_STATE();
    case 47:
      if (lookahead == 'u') ADVANCE(44);
      END_STATE();
    case 48:
      if (lookahead == 'u') ADVANCE(29);
      END_STATE();
    case 49:
      if (lookahead == 'v') ADVANCE(18);
      END_STATE();
    case 50:
      if (lookahead == '"' ||
          lookahead == '\\' ||
          lookahead == 'n' ||
          lookahead == 'r' ||
          lookahead == 't') ADVANCE(103);
      END_STATE();
    case 51:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(98);
      END_STATE();
    case 52:
      if (eof) ADVANCE(54);
      ADVANCE_MAP(
        '!', 9,
        '"', 99,
        '#', 33,
        '%', 88,
        '(', 73,
        ')', 74,
        '*', 86,
        '+', 89,
        ',', 62,
        '-', 85,
        '.', 94,
        '/', 87,
        ':', 75,
        ';', 64,
        '<', 10,
        '=', 60,
        '>', 12,
        '@', 72,
        '[', 61,
        ']', 63,
        'a', 159,
        'b', 136,
        'e', 125,
        'f', 106,
        'i', 118,
        'l', 115,
        'n', 104,
        'r', 116,
        's', 109,
        't', 142,
        '{', 57,
        '}', 58,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(52);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(97);
      if (lookahead == '_' ||
          ('c' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      if (('A' <= lookahead && lookahead <= 'Z')) ADVANCE(164);
      END_STATE();
    case 53:
      if (eof) ADVANCE(54);
      ADVANCE_MAP(
        '!', 9,
        '#', 33,
        '%', 88,
        '(', 73,
        ')', 74,
        '*', 86,
        '+', 89,
        ',', 62,
        '-', 84,
        '.', 94,
        '/', 87,
        ':', 75,
        ';', 64,
        '<', 10,
        '=', 60,
        '>', 12,
        '@', 72,
        ']', 63,
        'a', 47,
        'f', 46,
        'i', 26,
        'n', 15,
        'r', 48,
        's', 17,
        '{', 57,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(53);
      if (('A' <= lookahead && lookahead <= 'Z')) ADVANCE(164);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(sym_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(55);
      END_STATE();
    case 56:
      ACCEPT_TOKEN(anon_sym_POUNDplatforms);
      END_STATE();
    case 57:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 58:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 59:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 60:
      ACCEPT_TOKEN(anon_sym_EQ);
      if (lookahead == '=') ADVANCE(90);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 64:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 65:
      ACCEPT_TOKEN(anon_sym_import);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(anon_sym_func);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(anon_sym_DASH_GT);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(anon_sym_native);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(anon_sym_runtime);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(anon_sym_script);
      END_STATE();
    case 71:
      ACCEPT_TOKEN(anon_sym_auto);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(anon_sym_AT);
      END_STATE();
    case 73:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      END_STATE();
    case 74:
      ACCEPT_TOKEN(anon_sym_RPAREN);
      END_STATE();
    case 75:
      ACCEPT_TOKEN(anon_sym_COLON);
      END_STATE();
    case 76:
      ACCEPT_TOKEN(anon_sym_int);
      END_STATE();
    case 77:
      ACCEPT_TOKEN(anon_sym_float);
      END_STATE();
    case 78:
      ACCEPT_TOKEN(anon_sym_bool);
      END_STATE();
    case 79:
      ACCEPT_TOKEN(anon_sym_string);
      END_STATE();
    case 80:
      ACCEPT_TOKEN(anon_sym_if);
      END_STATE();
    case 81:
      ACCEPT_TOKEN(anon_sym_else);
      END_STATE();
    case 82:
      ACCEPT_TOKEN(anon_sym_let);
      END_STATE();
    case 83:
      ACCEPT_TOKEN(anon_sym_return);
      END_STATE();
    case 84:
      ACCEPT_TOKEN(anon_sym_DASH);
      END_STATE();
    case 85:
      ACCEPT_TOKEN(anon_sym_DASH);
      if (lookahead == '>') ADVANCE(67);
      END_STATE();
    case 86:
      ACCEPT_TOKEN(anon_sym_STAR);
      END_STATE();
    case 87:
      ACCEPT_TOKEN(anon_sym_SLASH);
      if (lookahead == '/') ADVANCE(55);
      END_STATE();
    case 88:
      ACCEPT_TOKEN(anon_sym_PERCENT);
      END_STATE();
    case 89:
      ACCEPT_TOKEN(anon_sym_PLUS);
      END_STATE();
    case 90:
      ACCEPT_TOKEN(anon_sym_EQ_EQ);
      END_STATE();
    case 91:
      ACCEPT_TOKEN(anon_sym_BANG_EQ);
      END_STATE();
    case 92:
      ACCEPT_TOKEN(anon_sym_LT_EQ);
      END_STATE();
    case 93:
      ACCEPT_TOKEN(anon_sym_GT_EQ);
      END_STATE();
    case 94:
      ACCEPT_TOKEN(anon_sym_DOT);
      END_STATE();
    case 95:
      ACCEPT_TOKEN(sym_true);
      END_STATE();
    case 96:
      ACCEPT_TOKEN(sym_false);
      END_STATE();
    case 97:
      ACCEPT_TOKEN(sym_integer_literal);
      if (lookahead == '.') ADVANCE(51);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(97);
      END_STATE();
    case 98:
      ACCEPT_TOKEN(sym_float_literal);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(98);
      END_STATE();
    case 99:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 100:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '/') ADVANCE(101);
      if (lookahead == '\t' ||
          (0x0b <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(100);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(102);
      END_STATE();
    case 101:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '/') ADVANCE(102);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(102);
      END_STATE();
    case 102:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(102);
      END_STATE();
    case 103:
      ACCEPT_TOKEN(sym_escape_sequence);
      END_STATE();
    case 104:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'a') ADVANCE(151);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 105:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'a') ADVANCE(152);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 106:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'a') ADVANCE(128);
      if (lookahead == 'l') ADVANCE(139);
      if (lookahead == 'u') ADVANCE(130);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 107:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'a') ADVANCE(128);
      if (lookahead == 'l') ADVANCE(139);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 108:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'c') ADVANCE(66);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 109:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'c') ADVANCE(143);
      if (lookahead == 't') ADVANCE(144);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 110:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'e') ADVANCE(81);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 111:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'e') ADVANCE(95);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 112:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'e') ADVANCE(96);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 113:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'e') ADVANCE(68);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 114:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'e') ADVANCE(69);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 115:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'e') ADVANCE(150);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 116:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'e') ADVANCE(155);
      if (lookahead == 'u') ADVANCE(134);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 117:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'e') ADVANCE(155);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 118:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'f') ADVANCE(80);
      if (lookahead == 'm') ADVANCE(140);
      if (lookahead == 'n') ADVANCE(149);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 119:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'f') ADVANCE(80);
      if (lookahead == 'n') ADVANCE(149);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 120:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'g') ADVANCE(79);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 121:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'i') ADVANCE(162);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 122:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'i') ADVANCE(129);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 123:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'i') ADVANCE(141);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 124:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'i') ADVANCE(131);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 125:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'l') ADVANCE(147);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 126:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'l') ADVANCE(78);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 127:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'l') ADVANCE(139);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 128:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'l') ADVANCE(148);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 129:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'm') ADVANCE(114);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 130:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'n') ADVANCE(108);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 131:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'n') ADVANCE(120);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 132:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'n') ADVANCE(83);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 133:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'n') ADVANCE(149);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 134:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'n') ADVANCE(157);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 135:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'o') ADVANCE(71);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 136:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'o') ADVANCE(137);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 137:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'o') ADVANCE(126);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 138:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'o') ADVANCE(146);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 139:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'o') ADVANCE(105);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 140:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'p') ADVANCE(138);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 141:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'p') ADVANCE(154);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 142:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'r') ADVANCE(160);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 143:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'r') ADVANCE(123);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 144:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'r') ADVANCE(124);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 145:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'r') ADVANCE(132);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 146:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'r') ADVANCE(153);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 147:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 's') ADVANCE(110);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 148:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 's') ADVANCE(112);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 149:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 't') ADVANCE(76);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 150:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 't') ADVANCE(82);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 151:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 't') ADVANCE(121);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 152:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 't') ADVANCE(77);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 153:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 't') ADVANCE(65);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 154:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 't') ADVANCE(70);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 155:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 't') ADVANCE(161);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 156:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 't') ADVANCE(135);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 157:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 't') ADVANCE(122);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 158:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 't') ADVANCE(144);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 159:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'u') ADVANCE(156);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 160:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'u') ADVANCE(111);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 161:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'u') ADVANCE(145);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 162:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (lookahead == 'v') ADVANCE(113);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 163:
      ACCEPT_TOKEN(aux_sym_identifier_token1);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      END_STATE();
    case 164:
      ACCEPT_TOKEN(sym_attribute_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(164);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 53},
  [2] = {.lex_state = 2},
  [3] = {.lex_state = 2},
  [4] = {.lex_state = 2},
  [5] = {.lex_state = 2},
  [6] = {.lex_state = 2},
  [7] = {.lex_state = 2},
  [8] = {.lex_state = 2},
  [9] = {.lex_state = 3},
  [10] = {.lex_state = 3},
  [11] = {.lex_state = 3},
  [12] = {.lex_state = 3},
  [13] = {.lex_state = 3},
  [14] = {.lex_state = 3},
  [15] = {.lex_state = 3},
  [16] = {.lex_state = 3},
  [17] = {.lex_state = 3},
  [18] = {.lex_state = 3},
  [19] = {.lex_state = 3},
  [20] = {.lex_state = 3},
  [21] = {.lex_state = 53},
  [22] = {.lex_state = 4},
  [23] = {.lex_state = 4},
  [24] = {.lex_state = 4},
  [25] = {.lex_state = 2},
  [26] = {.lex_state = 2},
  [27] = {.lex_state = 2},
  [28] = {.lex_state = 2},
  [29] = {.lex_state = 2},
  [30] = {.lex_state = 2},
  [31] = {.lex_state = 2},
  [32] = {.lex_state = 2},
  [33] = {.lex_state = 2},
  [34] = {.lex_state = 2},
  [35] = {.lex_state = 2},
  [36] = {.lex_state = 2},
  [37] = {.lex_state = 53},
  [38] = {.lex_state = 53},
  [39] = {.lex_state = 53},
  [40] = {.lex_state = 2},
  [41] = {.lex_state = 2},
  [42] = {.lex_state = 2},
  [43] = {.lex_state = 2},
  [44] = {.lex_state = 2},
  [45] = {.lex_state = 2},
  [46] = {.lex_state = 2},
  [47] = {.lex_state = 2},
  [48] = {.lex_state = 2},
  [49] = {.lex_state = 2},
  [50] = {.lex_state = 2},
  [51] = {.lex_state = 2},
  [52] = {.lex_state = 2},
  [53] = {.lex_state = 2},
  [54] = {.lex_state = 2},
  [55] = {.lex_state = 2},
  [56] = {.lex_state = 2},
  [57] = {.lex_state = 53},
  [58] = {.lex_state = 53},
  [59] = {.lex_state = 53},
  [60] = {.lex_state = 2},
  [61] = {.lex_state = 53},
  [62] = {.lex_state = 2},
  [63] = {.lex_state = 2},
  [64] = {.lex_state = 2},
  [65] = {.lex_state = 2},
  [66] = {.lex_state = 2},
  [67] = {.lex_state = 53},
  [68] = {.lex_state = 2},
  [69] = {.lex_state = 2},
  [70] = {.lex_state = 2},
  [71] = {.lex_state = 53},
  [72] = {.lex_state = 7},
  [73] = {.lex_state = 53},
  [74] = {.lex_state = 53},
  [75] = {.lex_state = 53},
  [76] = {.lex_state = 7},
  [77] = {.lex_state = 7},
  [78] = {.lex_state = 53},
  [79] = {.lex_state = 7},
  [80] = {.lex_state = 53},
  [81] = {.lex_state = 7},
  [82] = {.lex_state = 53},
  [83] = {.lex_state = 53},
  [84] = {.lex_state = 53},
  [85] = {.lex_state = 53},
  [86] = {.lex_state = 53},
  [87] = {.lex_state = 53},
  [88] = {.lex_state = 53},
  [89] = {.lex_state = 53},
  [90] = {.lex_state = 53},
  [91] = {.lex_state = 53},
  [92] = {.lex_state = 53},
  [93] = {.lex_state = 53},
  [94] = {.lex_state = 6},
  [95] = {.lex_state = 3},
  [96] = {.lex_state = 6},
  [97] = {.lex_state = 6},
  [98] = {.lex_state = 3},
  [99] = {.lex_state = 6},
  [100] = {.lex_state = 0},
  [101] = {.lex_state = 1},
  [102] = {.lex_state = 1},
  [103] = {.lex_state = 1},
  [104] = {.lex_state = 7},
  [105] = {.lex_state = 0},
  [106] = {.lex_state = 0},
  [107] = {.lex_state = 7},
  [108] = {.lex_state = 6},
  [109] = {.lex_state = 0},
  [110] = {.lex_state = 6},
  [111] = {.lex_state = 0},
  [112] = {.lex_state = 0},
  [113] = {.lex_state = 7},
  [114] = {.lex_state = 0},
  [115] = {.lex_state = 0},
  [116] = {.lex_state = 0},
  [117] = {.lex_state = 6},
  [118] = {.lex_state = 0},
  [119] = {.lex_state = 6},
  [120] = {.lex_state = 0},
  [121] = {.lex_state = 0},
  [122] = {.lex_state = 7},
  [123] = {.lex_state = 0},
  [124] = {.lex_state = 0},
  [125] = {.lex_state = 7},
  [126] = {.lex_state = 6},
  [127] = {.lex_state = 6},
  [128] = {.lex_state = 0},
  [129] = {.lex_state = 6},
  [130] = {.lex_state = 6},
  [131] = {.lex_state = 0},
  [132] = {.lex_state = 7},
  [133] = {.lex_state = 6},
  [134] = {.lex_state = 6},
  [135] = {.lex_state = 6},
  [136] = {.lex_state = 0},
  [137] = {.lex_state = 3},
  [138] = {.lex_state = 0},
  [139] = {.lex_state = 6},
  [140] = {.lex_state = 0},
  [141] = {.lex_state = 6},
  [142] = {.lex_state = 0},
  [143] = {.lex_state = 0},
  [144] = {.lex_state = 53},
  [145] = {.lex_state = 53},
  [146] = {.lex_state = 3},
  [147] = {.lex_state = 0},
  [148] = {.lex_state = 53},
  [149] = {.lex_state = 0},
  [150] = {.lex_state = 53},
  [151] = {.lex_state = 3},
  [152] = {.lex_state = 0},
  [153] = {.lex_state = 0},
  [154] = {.lex_state = 0},
  [155] = {.lex_state = 0},
  [156] = {.lex_state = 0},
  [157] = {.lex_state = 0},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [sym_comment] = ACTIONS(3),
    [anon_sym_POUNDplatforms] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
    [anon_sym_LBRACK] = ACTIONS(1),
    [anon_sym_COMMA] = ACTIONS(1),
    [anon_sym_RBRACK] = ACTIONS(1),
    [anon_sym_SEMI] = ACTIONS(1),
    [anon_sym_import] = ACTIONS(1),
    [anon_sym_func] = ACTIONS(1),
    [anon_sym_DASH_GT] = ACTIONS(1),
    [anon_sym_native] = ACTIONS(1),
    [anon_sym_runtime] = ACTIONS(1),
    [anon_sym_script] = ACTIONS(1),
    [anon_sym_auto] = ACTIONS(1),
    [anon_sym_AT] = ACTIONS(1),
    [anon_sym_LPAREN] = ACTIONS(1),
    [anon_sym_RPAREN] = ACTIONS(1),
    [anon_sym_COLON] = ACTIONS(1),
    [anon_sym_int] = ACTIONS(1),
    [anon_sym_float] = ACTIONS(1),
    [anon_sym_bool] = ACTIONS(1),
    [anon_sym_string] = ACTIONS(1),
    [anon_sym_if] = ACTIONS(1),
    [anon_sym_else] = ACTIONS(1),
    [anon_sym_let] = ACTIONS(1),
    [anon_sym_return] = ACTIONS(1),
    [anon_sym_DASH] = ACTIONS(1),
    [anon_sym_STAR] = ACTIONS(1),
    [anon_sym_SLASH] = ACTIONS(1),
    [anon_sym_PERCENT] = ACTIONS(1),
    [anon_sym_PLUS] = ACTIONS(1),
    [anon_sym_EQ_EQ] = ACTIONS(1),
    [anon_sym_BANG_EQ] = ACTIONS(1),
    [anon_sym_LT_EQ] = ACTIONS(1),
    [anon_sym_GT_EQ] = ACTIONS(1),
    [anon_sym_DOT] = ACTIONS(1),
    [sym_true] = ACTIONS(1),
    [sym_false] = ACTIONS(1),
    [sym_integer_literal] = ACTIONS(1),
    [sym_float_literal] = ACTIONS(1),
    [anon_sym_DQUOTE] = ACTIONS(1),
    [sym_escape_sequence] = ACTIONS(1),
    [aux_sym_identifier_token1] = ACTIONS(1),
    [sym_attribute_identifier] = ACTIONS(1),
  },
  [1] = {
    [sym_source_file] = STATE(157),
    [sym_platforms_metadata] = STATE(37),
    [sym_import_statement] = STATE(38),
    [sym_function_definition] = STATE(58),
    [sym_execution_mode] = STATE(150),
    [sym_attribute] = STATE(71),
    [aux_sym_source_file_repeat1] = STATE(38),
    [aux_sym_source_file_repeat2] = STATE(58),
    [aux_sym_function_definition_repeat1] = STATE(71),
    [ts_builtin_sym_end] = ACTIONS(5),
    [sym_comment] = ACTIONS(3),
    [anon_sym_POUNDplatforms] = ACTIONS(7),
    [anon_sym_import] = ACTIONS(9),
    [anon_sym_func] = ACTIONS(11),
    [anon_sym_native] = ACTIONS(13),
    [anon_sym_runtime] = ACTIONS(13),
    [anon_sym_script] = ACTIONS(13),
    [anon_sym_auto] = ACTIONS(13),
    [anon_sym_AT] = ACTIONS(15),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 20,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(17), 1,
      anon_sym_RBRACE,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(25), 1,
      anon_sym_let,
    ACTIONS(27), 1,
      anon_sym_return,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(66), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    STATE(7), 2,
      sym_statement,
      aux_sym_block_repeat1,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(28), 4,
      sym_if_statement,
      sym_let_statement,
      sym_return_statement,
      sym_expression_statement,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [74] = 20,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(25), 1,
      anon_sym_let,
    ACTIONS(27), 1,
      anon_sym_return,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    ACTIONS(39), 1,
      anon_sym_RBRACE,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(66), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    STATE(7), 2,
      sym_statement,
      aux_sym_block_repeat1,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(28), 4,
      sym_if_statement,
      sym_let_statement,
      sym_return_statement,
      sym_expression_statement,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [148] = 20,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(25), 1,
      anon_sym_let,
    ACTIONS(27), 1,
      anon_sym_return,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    ACTIONS(41), 1,
      anon_sym_RBRACE,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(66), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    STATE(2), 2,
      sym_statement,
      aux_sym_block_repeat1,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(28), 4,
      sym_if_statement,
      sym_let_statement,
      sym_return_statement,
      sym_expression_statement,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [222] = 20,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(25), 1,
      anon_sym_let,
    ACTIONS(27), 1,
      anon_sym_return,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    ACTIONS(43), 1,
      anon_sym_RBRACE,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(66), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    STATE(3), 2,
      sym_statement,
      aux_sym_block_repeat1,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(28), 4,
      sym_if_statement,
      sym_let_statement,
      sym_return_statement,
      sym_expression_statement,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [296] = 20,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(25), 1,
      anon_sym_let,
    ACTIONS(27), 1,
      anon_sym_return,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    ACTIONS(45), 1,
      anon_sym_RBRACE,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(66), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    STATE(8), 2,
      sym_statement,
      aux_sym_block_repeat1,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(28), 4,
      sym_if_statement,
      sym_let_statement,
      sym_return_statement,
      sym_expression_statement,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [370] = 20,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(47), 1,
      anon_sym_RBRACE,
    ACTIONS(49), 1,
      anon_sym_LPAREN,
    ACTIONS(55), 1,
      anon_sym_if,
    ACTIONS(58), 1,
      anon_sym_let,
    ACTIONS(61), 1,
      anon_sym_return,
    ACTIONS(64), 1,
      anon_sym_DASH,
    ACTIONS(70), 1,
      sym_integer_literal,
    ACTIONS(73), 1,
      anon_sym_DQUOTE,
    ACTIONS(76), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(66), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    STATE(7), 2,
      sym_statement,
      aux_sym_block_repeat1,
    ACTIONS(67), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(52), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(28), 4,
      sym_if_statement,
      sym_let_statement,
      sym_return_statement,
      sym_expression_statement,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [444] = 20,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(23), 1,
      anon_sym_if,
    ACTIONS(25), 1,
      anon_sym_let,
    ACTIONS(27), 1,
      anon_sym_return,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    ACTIONS(79), 1,
      anon_sym_RBRACE,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(66), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    STATE(7), 2,
      sym_statement,
      aux_sym_block_repeat1,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(28), 4,
      sym_if_statement,
      sym_let_statement,
      sym_return_statement,
      sym_expression_statement,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [518] = 15,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    ACTIONS(81), 1,
      anon_sym_RPAREN,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(60), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [573] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(69), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [625] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(65), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [677] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(44), 1,
      sym_expression,
    STATE(49), 1,
      sym_string_literal,
    STATE(155), 1,
      sym_builtin_type,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [729] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(62), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [781] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(68), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [833] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(45), 1,
      sym_expression,
    STATE(49), 1,
      sym_string_literal,
    STATE(155), 1,
      sym_builtin_type,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [885] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(70), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [937] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(63), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [989] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(64), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [1041] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(49), 1,
      sym_string_literal,
    STATE(56), 1,
      sym_expression,
    STATE(155), 1,
      sym_builtin_type,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [1093] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_LPAREN,
    ACTIONS(29), 1,
      anon_sym_DASH,
    ACTIONS(33), 1,
      sym_integer_literal,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(40), 1,
      sym_path,
    STATE(43), 1,
      sym_expression,
    STATE(49), 1,
      sym_string_literal,
    STATE(155), 1,
      sym_builtin_type,
    ACTIONS(31), 3,
      sym_true,
      sym_false,
      sym_float_literal,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
    STATE(54), 5,
      sym_call_expression,
      sym_cast_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_literal,
  [1145] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(85), 2,
      anon_sym_EQ,
      anon_sym_SLASH,
    ACTIONS(83), 16,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COLON,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_DOT,
  [1171] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(89), 2,
      sym_integer_literal,
      aux_sym_identifier_token1,
    ACTIONS(87), 15,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
      anon_sym_if,
      anon_sym_else,
      anon_sym_let,
      anon_sym_return,
      anon_sym_DASH,
      sym_true,
      sym_false,
      sym_float_literal,
      anon_sym_DQUOTE,
  [1196] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(93), 2,
      sym_integer_literal,
      aux_sym_identifier_token1,
    ACTIONS(91), 15,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
      anon_sym_if,
      anon_sym_else,
      anon_sym_let,
      anon_sym_return,
      anon_sym_DASH,
      sym_true,
      sym_false,
      sym_float_literal,
      anon_sym_DQUOTE,
  [1221] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(97), 1,
      anon_sym_else,
    ACTIONS(99), 2,
      sym_integer_literal,
      aux_sym_identifier_token1,
    ACTIONS(95), 14,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
      anon_sym_if,
      anon_sym_let,
      anon_sym_return,
      anon_sym_DASH,
      sym_true,
      sym_false,
      sym_float_literal,
      anon_sym_DQUOTE,
  [1248] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(103), 2,
      sym_integer_literal,
      aux_sym_identifier_token1,
    ACTIONS(101), 14,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
      anon_sym_if,
      anon_sym_let,
      anon_sym_return,
      anon_sym_DASH,
      sym_true,
      sym_false,
      sym_float_literal,
      anon_sym_DQUOTE,
  [1272] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(109), 1,
      anon_sym_DOT,
    STATE(26), 1,
      aux_sym_path_repeat1,
    ACTIONS(105), 13,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1300] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(114), 1,
      anon_sym_SLASH,
    ACTIONS(116), 1,
      anon_sym_DOT,
    STATE(29), 1,
      aux_sym_path_repeat1,
    ACTIONS(112), 13,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1328] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(120), 2,
      sym_integer_literal,
      aux_sym_identifier_token1,
    ACTIONS(118), 14,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
      anon_sym_if,
      anon_sym_let,
      anon_sym_return,
      anon_sym_DASH,
      sym_true,
      sym_false,
      sym_float_literal,
      anon_sym_DQUOTE,
  [1352] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(116), 1,
      anon_sym_DOT,
    ACTIONS(124), 1,
      anon_sym_SLASH,
    STATE(26), 1,
      aux_sym_path_repeat1,
    ACTIONS(122), 13,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1380] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(128), 2,
      sym_integer_literal,
      aux_sym_identifier_token1,
    ACTIONS(126), 14,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
      anon_sym_if,
      anon_sym_let,
      anon_sym_return,
      anon_sym_DASH,
      sym_true,
      sym_false,
      sym_float_literal,
      anon_sym_DQUOTE,
  [1404] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(132), 2,
      sym_integer_literal,
      aux_sym_identifier_token1,
    ACTIONS(130), 14,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
      anon_sym_if,
      anon_sym_let,
      anon_sym_return,
      anon_sym_DASH,
      sym_true,
      sym_false,
      sym_float_literal,
      anon_sym_DQUOTE,
  [1428] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(136), 2,
      sym_integer_literal,
      aux_sym_identifier_token1,
    ACTIONS(134), 14,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
      anon_sym_if,
      anon_sym_let,
      anon_sym_return,
      anon_sym_DASH,
      sym_true,
      sym_false,
      sym_float_literal,
      anon_sym_DQUOTE,
  [1452] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(140), 2,
      sym_integer_literal,
      aux_sym_identifier_token1,
    ACTIONS(138), 14,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
      anon_sym_if,
      anon_sym_let,
      anon_sym_return,
      anon_sym_DASH,
      sym_true,
      sym_false,
      sym_float_literal,
      anon_sym_DQUOTE,
  [1476] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(93), 2,
      sym_integer_literal,
      aux_sym_identifier_token1,
    ACTIONS(91), 14,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
      anon_sym_if,
      anon_sym_let,
      anon_sym_return,
      anon_sym_DASH,
      sym_true,
      sym_false,
      sym_float_literal,
      anon_sym_DQUOTE,
  [1500] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(89), 2,
      sym_integer_literal,
      aux_sym_identifier_token1,
    ACTIONS(87), 14,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
      anon_sym_if,
      anon_sym_let,
      anon_sym_return,
      anon_sym_DASH,
      sym_true,
      sym_false,
      sym_float_literal,
      anon_sym_DQUOTE,
  [1524] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(144), 1,
      anon_sym_LPAREN,
    ACTIONS(146), 1,
      anon_sym_SLASH,
    STATE(50), 1,
      sym_argument_list,
    ACTIONS(142), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1551] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(9), 1,
      anon_sym_import,
    ACTIONS(11), 1,
      anon_sym_func,
    ACTIONS(15), 1,
      anon_sym_AT,
    ACTIONS(148), 1,
      ts_builtin_sym_end,
    STATE(150), 1,
      sym_execution_mode,
    STATE(39), 2,
      sym_import_statement,
      aux_sym_source_file_repeat1,
    STATE(57), 2,
      sym_function_definition,
      aux_sym_source_file_repeat2,
    STATE(71), 2,
      sym_attribute,
      aux_sym_function_definition_repeat1,
    ACTIONS(13), 4,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
  [1588] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(9), 1,
      anon_sym_import,
    ACTIONS(11), 1,
      anon_sym_func,
    ACTIONS(15), 1,
      anon_sym_AT,
    ACTIONS(148), 1,
      ts_builtin_sym_end,
    STATE(150), 1,
      sym_execution_mode,
    STATE(57), 2,
      sym_function_definition,
      aux_sym_source_file_repeat2,
    STATE(67), 2,
      sym_import_statement,
      aux_sym_source_file_repeat1,
    STATE(71), 2,
      sym_attribute,
      aux_sym_function_definition_repeat1,
    ACTIONS(13), 4,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
  [1625] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(9), 1,
      anon_sym_import,
    ACTIONS(11), 1,
      anon_sym_func,
    ACTIONS(15), 1,
      anon_sym_AT,
    ACTIONS(150), 1,
      ts_builtin_sym_end,
    STATE(150), 1,
      sym_execution_mode,
    STATE(61), 2,
      sym_function_definition,
      aux_sym_source_file_repeat2,
    STATE(67), 2,
      sym_import_statement,
      aux_sym_source_file_repeat1,
    STATE(71), 2,
      sym_attribute,
      aux_sym_function_definition_repeat1,
    ACTIONS(13), 4,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
  [1662] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(144), 1,
      anon_sym_LPAREN,
    ACTIONS(154), 1,
      anon_sym_SLASH,
    STATE(46), 1,
      sym_argument_list,
    ACTIONS(152), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1689] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(158), 1,
      anon_sym_SLASH,
    ACTIONS(156), 14,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_DOT,
  [1712] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(105), 14,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_DOT,
  [1735] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(162), 1,
      anon_sym_SLASH,
    ACTIONS(160), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1756] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(166), 1,
      anon_sym_SLASH,
    ACTIONS(164), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1777] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(170), 1,
      anon_sym_SLASH,
    ACTIONS(168), 2,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(160), 10,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1800] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(174), 1,
      anon_sym_SLASH,
    ACTIONS(172), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1821] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(178), 1,
      anon_sym_SLASH,
    ACTIONS(176), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1842] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(182), 1,
      anon_sym_SLASH,
    ACTIONS(180), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1863] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(186), 1,
      anon_sym_SLASH,
    ACTIONS(184), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1884] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(190), 1,
      anon_sym_SLASH,
    ACTIONS(188), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1905] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(194), 1,
      anon_sym_SLASH,
    ACTIONS(192), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1926] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(198), 1,
      anon_sym_SLASH,
    ACTIONS(196), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1947] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(202), 1,
      anon_sym_SLASH,
    ACTIONS(200), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1968] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      anon_sym_SLASH,
    ACTIONS(152), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [1989] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(206), 1,
      anon_sym_SLASH,
    ACTIONS(204), 12,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_PLUS,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [2010] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(170), 1,
      anon_sym_SLASH,
    ACTIONS(168), 2,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(208), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(160), 8,
      anon_sym_LBRACE,
      anon_sym_COMMA,
      anon_sym_SEMI,
      anon_sym_RPAREN,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [2035] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(11), 1,
      anon_sym_func,
    ACTIONS(15), 1,
      anon_sym_AT,
    ACTIONS(150), 1,
      ts_builtin_sym_end,
    STATE(150), 1,
      sym_execution_mode,
    STATE(59), 2,
      sym_function_definition,
      aux_sym_source_file_repeat2,
    STATE(71), 2,
      sym_attribute,
      aux_sym_function_definition_repeat1,
    ACTIONS(13), 4,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
  [2065] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(11), 1,
      anon_sym_func,
    ACTIONS(15), 1,
      anon_sym_AT,
    ACTIONS(148), 1,
      ts_builtin_sym_end,
    STATE(150), 1,
      sym_execution_mode,
    STATE(59), 2,
      sym_function_definition,
      aux_sym_source_file_repeat2,
    STATE(71), 2,
      sym_attribute,
      aux_sym_function_definition_repeat1,
    ACTIONS(13), 4,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
  [2095] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(210), 1,
      ts_builtin_sym_end,
    ACTIONS(212), 1,
      anon_sym_func,
    ACTIONS(218), 1,
      anon_sym_AT,
    STATE(150), 1,
      sym_execution_mode,
    STATE(59), 2,
      sym_function_definition,
      aux_sym_source_file_repeat2,
    STATE(71), 2,
      sym_attribute,
      aux_sym_function_definition_repeat1,
    ACTIONS(215), 4,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
  [2125] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(170), 1,
      anon_sym_SLASH,
    ACTIONS(221), 1,
      anon_sym_COMMA,
    ACTIONS(223), 1,
      anon_sym_RPAREN,
    STATE(114), 1,
      aux_sym_argument_list_repeat1,
    ACTIONS(168), 2,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(208), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(225), 4,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [2155] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(11), 1,
      anon_sym_func,
    ACTIONS(15), 1,
      anon_sym_AT,
    ACTIONS(227), 1,
      ts_builtin_sym_end,
    STATE(150), 1,
      sym_execution_mode,
    STATE(59), 2,
      sym_function_definition,
      aux_sym_source_file_repeat2,
    STATE(71), 2,
      sym_attribute,
      aux_sym_function_definition_repeat1,
    ACTIONS(13), 4,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
  [2185] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(170), 1,
      anon_sym_SLASH,
    ACTIONS(229), 1,
      anon_sym_LBRACE,
    STATE(24), 1,
      sym_block,
    ACTIONS(168), 2,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(208), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(225), 4,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [2212] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(170), 1,
      anon_sym_SLASH,
    ACTIONS(168), 2,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(208), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(231), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
    ACTIONS(225), 4,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [2237] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(170), 1,
      anon_sym_SLASH,
    ACTIONS(233), 1,
      anon_sym_SEMI,
    ACTIONS(168), 2,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(208), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(225), 4,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [2261] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(170), 1,
      anon_sym_SLASH,
    ACTIONS(235), 1,
      anon_sym_SEMI,
    ACTIONS(168), 2,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(208), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(225), 4,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [2285] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(170), 1,
      anon_sym_SLASH,
    ACTIONS(237), 1,
      anon_sym_SEMI,
    ACTIONS(168), 2,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(208), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(225), 4,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [2309] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(241), 1,
      anon_sym_import,
    STATE(67), 2,
      sym_import_statement,
      aux_sym_source_file_repeat1,
    ACTIONS(239), 7,
      ts_builtin_sym_end,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2329] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(170), 1,
      anon_sym_SLASH,
    ACTIONS(244), 1,
      anon_sym_RPAREN,
    ACTIONS(168), 2,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(208), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(225), 4,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [2353] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(170), 1,
      anon_sym_SLASH,
    ACTIONS(246), 1,
      anon_sym_RPAREN,
    ACTIONS(168), 2,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(208), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(225), 4,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [2377] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(170), 1,
      anon_sym_SLASH,
    ACTIONS(248), 1,
      anon_sym_SEMI,
    ACTIONS(168), 2,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(208), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(225), 4,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
  [2401] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(15), 1,
      anon_sym_AT,
    ACTIONS(250), 1,
      anon_sym_func,
    STATE(148), 1,
      sym_execution_mode,
    STATE(80), 2,
      sym_attribute,
      aux_sym_function_definition_repeat1,
    ACTIONS(13), 4,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
  [2424] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(140), 1,
      sym_type,
    STATE(98), 2,
      sym_builtin_type,
      sym_identifier,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
  [2444] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(252), 8,
      ts_builtin_sym_end,
      anon_sym_import,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2458] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(254), 8,
      ts_builtin_sym_end,
      anon_sym_import,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2472] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(256), 8,
      ts_builtin_sym_end,
      anon_sym_import,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2486] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(124), 1,
      sym_type,
    STATE(98), 2,
      sym_builtin_type,
      sym_identifier,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
  [2506] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(121), 1,
      sym_type,
    STATE(98), 2,
      sym_builtin_type,
      sym_identifier,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
  [2526] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(260), 1,
      anon_sym_LPAREN,
    STATE(91), 1,
      sym_attribute_arguments,
    ACTIONS(258), 6,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2544] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(146), 1,
      sym_type,
    STATE(98), 2,
      sym_builtin_type,
      sym_identifier,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
  [2564] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(264), 1,
      anon_sym_AT,
    STATE(80), 2,
      sym_attribute,
      aux_sym_function_definition_repeat1,
    ACTIONS(262), 5,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
  [2582] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(37), 1,
      aux_sym_identifier_token1,
    STATE(136), 1,
      sym_type,
    STATE(98), 2,
      sym_builtin_type,
      sym_identifier,
    ACTIONS(21), 4,
      anon_sym_int,
      anon_sym_float,
      anon_sym_bool,
      anon_sym_string,
  [2602] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(267), 7,
      ts_builtin_sym_end,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2615] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(269), 7,
      ts_builtin_sym_end,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2628] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(87), 7,
      ts_builtin_sym_end,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2641] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(271), 7,
      ts_builtin_sym_end,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2654] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(273), 7,
      ts_builtin_sym_end,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2667] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(275), 7,
      ts_builtin_sym_end,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2680] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(277), 7,
      ts_builtin_sym_end,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2693] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(91), 7,
      ts_builtin_sym_end,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2706] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(279), 6,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2718] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(281), 6,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2730] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(283), 6,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2742] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(285), 6,
      anon_sym_func,
      anon_sym_native,
      anon_sym_runtime,
      anon_sym_script,
      anon_sym_auto,
      anon_sym_AT,
  [2754] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(287), 1,
      anon_sym_RBRACE,
    ACTIONS(289), 1,
      aux_sym_identifier_token1,
    STATE(151), 1,
      sym_identifier,
    STATE(97), 2,
      sym_platform_group,
      aux_sym_platforms_metadata_repeat1,
  [2771] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(291), 5,
      anon_sym_LBRACE,
      anon_sym_EQ,
      anon_sym_COMMA,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [2782] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(289), 1,
      aux_sym_identifier_token1,
    ACTIONS(293), 1,
      anon_sym_RBRACE,
    STATE(151), 1,
      sym_identifier,
    STATE(94), 2,
      sym_platform_group,
      aux_sym_platforms_metadata_repeat1,
  [2799] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(295), 1,
      anon_sym_RBRACE,
    ACTIONS(297), 1,
      aux_sym_identifier_token1,
    STATE(151), 1,
      sym_identifier,
    STATE(97), 2,
      sym_platform_group,
      aux_sym_platforms_metadata_repeat1,
  [2816] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(300), 4,
      anon_sym_LBRACE,
      anon_sym_EQ,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [2826] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(289), 1,
      aux_sym_identifier_token1,
    ACTIONS(302), 1,
      anon_sym_RPAREN,
    STATE(112), 1,
      sym_parameter,
    STATE(143), 1,
      sym_identifier,
  [2842] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(304), 1,
      anon_sym_COMMA,
    STATE(100), 1,
      aux_sym_platform_group_repeat1,
    ACTIONS(307), 2,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
  [2856] = 4,
    ACTIONS(309), 1,
      sym_comment,
    ACTIONS(311), 1,
      anon_sym_DQUOTE,
    STATE(103), 1,
      aux_sym_string_literal_repeat1,
    ACTIONS(313), 2,
      aux_sym_string_literal_token1,
      sym_escape_sequence,
  [2870] = 4,
    ACTIONS(309), 1,
      sym_comment,
    ACTIONS(315), 1,
      anon_sym_DQUOTE,
    STATE(102), 1,
      aux_sym_string_literal_repeat1,
    ACTIONS(317), 2,
      aux_sym_string_literal_token1,
      sym_escape_sequence,
  [2884] = 4,
    ACTIONS(309), 1,
      sym_comment,
    ACTIONS(320), 1,
      anon_sym_DQUOTE,
    STATE(102), 1,
      aux_sym_string_literal_repeat1,
    ACTIONS(322), 2,
      aux_sym_string_literal_token1,
      sym_escape_sequence,
  [2898] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(324), 1,
      anon_sym_LBRACE,
    ACTIONS(326), 1,
      anon_sym_DASH_GT,
    STATE(85), 1,
      sym_block,
  [2911] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(307), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
  [2920] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(328), 1,
      anon_sym_COMMA,
    ACTIONS(330), 1,
      anon_sym_RPAREN,
    STATE(116), 1,
      aux_sym_parameter_list_repeat1,
  [2933] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(324), 1,
      anon_sym_LBRACE,
    ACTIONS(332), 1,
      anon_sym_DASH_GT,
    STATE(82), 1,
      sym_block,
  [2946] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(289), 1,
      aux_sym_identifier_token1,
    ACTIONS(334), 1,
      anon_sym_RPAREN,
    STATE(120), 1,
      sym_identifier,
  [2959] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(336), 1,
      anon_sym_COMMA,
    ACTIONS(338), 1,
      anon_sym_RBRACK,
    STATE(100), 1,
      aux_sym_platform_group_repeat1,
  [2972] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(289), 1,
      aux_sym_identifier_token1,
    STATE(27), 1,
      sym_identifier,
    STATE(149), 1,
      sym_path,
  [2985] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(336), 1,
      anon_sym_COMMA,
    ACTIONS(340), 1,
      anon_sym_RPAREN,
    STATE(100), 1,
      aux_sym_platform_group_repeat1,
  [2998] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(328), 1,
      anon_sym_COMMA,
    ACTIONS(342), 1,
      anon_sym_RPAREN,
    STATE(106), 1,
      aux_sym_parameter_list_repeat1,
  [3011] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(324), 1,
      anon_sym_LBRACE,
    ACTIONS(344), 1,
      anon_sym_DASH_GT,
    STATE(88), 1,
      sym_block,
  [3024] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(221), 1,
      anon_sym_COMMA,
    ACTIONS(346), 1,
      anon_sym_RPAREN,
    STATE(118), 1,
      aux_sym_argument_list_repeat1,
  [3037] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(336), 1,
      anon_sym_COMMA,
    ACTIONS(348), 1,
      anon_sym_RBRACK,
    STATE(109), 1,
      aux_sym_platform_group_repeat1,
  [3050] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(350), 1,
      anon_sym_COMMA,
    ACTIONS(353), 1,
      anon_sym_RPAREN,
    STATE(116), 1,
      aux_sym_parameter_list_repeat1,
  [3063] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(289), 1,
      aux_sym_identifier_token1,
    STATE(131), 1,
      sym_parameter,
    STATE(143), 1,
      sym_identifier,
  [3076] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(231), 1,
      anon_sym_RPAREN,
    ACTIONS(355), 1,
      anon_sym_COMMA,
    STATE(118), 1,
      aux_sym_argument_list_repeat1,
  [3089] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(289), 1,
      aux_sym_identifier_token1,
    ACTIONS(358), 1,
      anon_sym_RBRACK,
    STATE(115), 1,
      sym_identifier,
  [3102] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(336), 1,
      anon_sym_COMMA,
    ACTIONS(360), 1,
      anon_sym_RPAREN,
    STATE(111), 1,
      aux_sym_platform_group_repeat1,
  [3115] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(362), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [3123] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(364), 2,
      anon_sym_LBRACE,
      anon_sym_DASH_GT,
  [3131] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(366), 1,
      anon_sym_LBRACE,
    STATE(31), 1,
      sym_block,
  [3141] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(324), 1,
      anon_sym_LBRACE,
    STATE(83), 1,
      sym_block,
  [3151] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(368), 2,
      anon_sym_LBRACE,
      anon_sym_DASH_GT,
  [3159] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(370), 1,
      aux_sym_identifier_token1,
    STATE(42), 1,
      sym_member_identifier,
  [3169] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(289), 1,
      aux_sym_identifier_token1,
    STATE(138), 1,
      sym_identifier,
  [3179] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    STATE(104), 1,
      sym_parameter_list,
  [3189] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(374), 2,
      anon_sym_RBRACE,
      aux_sym_identifier_token1,
  [3197] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(289), 1,
      aux_sym_identifier_token1,
    STATE(137), 1,
      sym_identifier,
  [3207] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(353), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [3215] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(376), 2,
      anon_sym_LBRACE,
      anon_sym_DASH_GT,
  [3223] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(289), 1,
      aux_sym_identifier_token1,
    STATE(142), 1,
      sym_identifier,
  [3233] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(378), 2,
      anon_sym_RBRACE,
      aux_sym_identifier_token1,
  [3241] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(289), 1,
      aux_sym_identifier_token1,
    STATE(128), 1,
      sym_identifier,
  [3251] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(324), 1,
      anon_sym_LBRACE,
    STATE(86), 1,
      sym_block,
  [3261] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(380), 1,
      anon_sym_EQ,
    ACTIONS(382), 1,
      anon_sym_COLON,
  [3271] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    STATE(107), 1,
      sym_parameter_list,
  [3281] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(289), 1,
      aux_sym_identifier_token1,
    STATE(105), 1,
      sym_identifier,
  [3291] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(324), 1,
      anon_sym_LBRACE,
    STATE(87), 1,
      sym_block,
  [3301] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(384), 2,
      anon_sym_RBRACE,
      aux_sym_identifier_token1,
  [3309] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    STATE(113), 1,
      sym_parameter_list,
  [3319] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(386), 1,
      anon_sym_COLON,
  [3326] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(388), 1,
      anon_sym_func,
  [3333] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(390), 1,
      sym_attribute_identifier,
  [3340] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(392), 1,
      anon_sym_EQ,
  [3347] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(394), 1,
      anon_sym_SEMI,
  [3354] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(396), 1,
      anon_sym_func,
  [3361] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(398), 1,
      anon_sym_SEMI,
  [3368] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(250), 1,
      anon_sym_func,
  [3375] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(400), 1,
      anon_sym_EQ,
  [3382] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(402), 1,
      anon_sym_LBRACE,
  [3389] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(404), 1,
      anon_sym_LBRACK,
  [3396] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(406), 1,
      anon_sym_SEMI,
  [3403] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(408), 1,
      anon_sym_LPAREN,
  [3410] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(410), 1,
      anon_sym_SEMI,
  [3417] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(412), 1,
      ts_builtin_sym_end,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 74,
  [SMALL_STATE(4)] = 148,
  [SMALL_STATE(5)] = 222,
  [SMALL_STATE(6)] = 296,
  [SMALL_STATE(7)] = 370,
  [SMALL_STATE(8)] = 444,
  [SMALL_STATE(9)] = 518,
  [SMALL_STATE(10)] = 573,
  [SMALL_STATE(11)] = 625,
  [SMALL_STATE(12)] = 677,
  [SMALL_STATE(13)] = 729,
  [SMALL_STATE(14)] = 781,
  [SMALL_STATE(15)] = 833,
  [SMALL_STATE(16)] = 885,
  [SMALL_STATE(17)] = 937,
  [SMALL_STATE(18)] = 989,
  [SMALL_STATE(19)] = 1041,
  [SMALL_STATE(20)] = 1093,
  [SMALL_STATE(21)] = 1145,
  [SMALL_STATE(22)] = 1171,
  [SMALL_STATE(23)] = 1196,
  [SMALL_STATE(24)] = 1221,
  [SMALL_STATE(25)] = 1248,
  [SMALL_STATE(26)] = 1272,
  [SMALL_STATE(27)] = 1300,
  [SMALL_STATE(28)] = 1328,
  [SMALL_STATE(29)] = 1352,
  [SMALL_STATE(30)] = 1380,
  [SMALL_STATE(31)] = 1404,
  [SMALL_STATE(32)] = 1428,
  [SMALL_STATE(33)] = 1452,
  [SMALL_STATE(34)] = 1476,
  [SMALL_STATE(35)] = 1500,
  [SMALL_STATE(36)] = 1524,
  [SMALL_STATE(37)] = 1551,
  [SMALL_STATE(38)] = 1588,
  [SMALL_STATE(39)] = 1625,
  [SMALL_STATE(40)] = 1662,
  [SMALL_STATE(41)] = 1689,
  [SMALL_STATE(42)] = 1712,
  [SMALL_STATE(43)] = 1735,
  [SMALL_STATE(44)] = 1756,
  [SMALL_STATE(45)] = 1777,
  [SMALL_STATE(46)] = 1800,
  [SMALL_STATE(47)] = 1821,
  [SMALL_STATE(48)] = 1842,
  [SMALL_STATE(49)] = 1863,
  [SMALL_STATE(50)] = 1884,
  [SMALL_STATE(51)] = 1905,
  [SMALL_STATE(52)] = 1926,
  [SMALL_STATE(53)] = 1947,
  [SMALL_STATE(54)] = 1968,
  [SMALL_STATE(55)] = 1989,
  [SMALL_STATE(56)] = 2010,
  [SMALL_STATE(57)] = 2035,
  [SMALL_STATE(58)] = 2065,
  [SMALL_STATE(59)] = 2095,
  [SMALL_STATE(60)] = 2125,
  [SMALL_STATE(61)] = 2155,
  [SMALL_STATE(62)] = 2185,
  [SMALL_STATE(63)] = 2212,
  [SMALL_STATE(64)] = 2237,
  [SMALL_STATE(65)] = 2261,
  [SMALL_STATE(66)] = 2285,
  [SMALL_STATE(67)] = 2309,
  [SMALL_STATE(68)] = 2329,
  [SMALL_STATE(69)] = 2353,
  [SMALL_STATE(70)] = 2377,
  [SMALL_STATE(71)] = 2401,
  [SMALL_STATE(72)] = 2424,
  [SMALL_STATE(73)] = 2444,
  [SMALL_STATE(74)] = 2458,
  [SMALL_STATE(75)] = 2472,
  [SMALL_STATE(76)] = 2486,
  [SMALL_STATE(77)] = 2506,
  [SMALL_STATE(78)] = 2526,
  [SMALL_STATE(79)] = 2544,
  [SMALL_STATE(80)] = 2564,
  [SMALL_STATE(81)] = 2582,
  [SMALL_STATE(82)] = 2602,
  [SMALL_STATE(83)] = 2615,
  [SMALL_STATE(84)] = 2628,
  [SMALL_STATE(85)] = 2641,
  [SMALL_STATE(86)] = 2654,
  [SMALL_STATE(87)] = 2667,
  [SMALL_STATE(88)] = 2680,
  [SMALL_STATE(89)] = 2693,
  [SMALL_STATE(90)] = 2706,
  [SMALL_STATE(91)] = 2718,
  [SMALL_STATE(92)] = 2730,
  [SMALL_STATE(93)] = 2742,
  [SMALL_STATE(94)] = 2754,
  [SMALL_STATE(95)] = 2771,
  [SMALL_STATE(96)] = 2782,
  [SMALL_STATE(97)] = 2799,
  [SMALL_STATE(98)] = 2816,
  [SMALL_STATE(99)] = 2826,
  [SMALL_STATE(100)] = 2842,
  [SMALL_STATE(101)] = 2856,
  [SMALL_STATE(102)] = 2870,
  [SMALL_STATE(103)] = 2884,
  [SMALL_STATE(104)] = 2898,
  [SMALL_STATE(105)] = 2911,
  [SMALL_STATE(106)] = 2920,
  [SMALL_STATE(107)] = 2933,
  [SMALL_STATE(108)] = 2946,
  [SMALL_STATE(109)] = 2959,
  [SMALL_STATE(110)] = 2972,
  [SMALL_STATE(111)] = 2985,
  [SMALL_STATE(112)] = 2998,
  [SMALL_STATE(113)] = 3011,
  [SMALL_STATE(114)] = 3024,
  [SMALL_STATE(115)] = 3037,
  [SMALL_STATE(116)] = 3050,
  [SMALL_STATE(117)] = 3063,
  [SMALL_STATE(118)] = 3076,
  [SMALL_STATE(119)] = 3089,
  [SMALL_STATE(120)] = 3102,
  [SMALL_STATE(121)] = 3115,
  [SMALL_STATE(122)] = 3123,
  [SMALL_STATE(123)] = 3131,
  [SMALL_STATE(124)] = 3141,
  [SMALL_STATE(125)] = 3151,
  [SMALL_STATE(126)] = 3159,
  [SMALL_STATE(127)] = 3169,
  [SMALL_STATE(128)] = 3179,
  [SMALL_STATE(129)] = 3189,
  [SMALL_STATE(130)] = 3197,
  [SMALL_STATE(131)] = 3207,
  [SMALL_STATE(132)] = 3215,
  [SMALL_STATE(133)] = 3223,
  [SMALL_STATE(134)] = 3233,
  [SMALL_STATE(135)] = 3241,
  [SMALL_STATE(136)] = 3251,
  [SMALL_STATE(137)] = 3261,
  [SMALL_STATE(138)] = 3271,
  [SMALL_STATE(139)] = 3281,
  [SMALL_STATE(140)] = 3291,
  [SMALL_STATE(141)] = 3301,
  [SMALL_STATE(142)] = 3309,
  [SMALL_STATE(143)] = 3319,
  [SMALL_STATE(144)] = 3326,
  [SMALL_STATE(145)] = 3333,
  [SMALL_STATE(146)] = 3340,
  [SMALL_STATE(147)] = 3347,
  [SMALL_STATE(148)] = 3354,
  [SMALL_STATE(149)] = 3361,
  [SMALL_STATE(150)] = 3368,
  [SMALL_STATE(151)] = 3375,
  [SMALL_STATE(152)] = 3382,
  [SMALL_STATE(153)] = 3389,
  [SMALL_STATE(154)] = 3396,
  [SMALL_STATE(155)] = 3403,
  [SMALL_STATE(156)] = 3410,
  [SMALL_STATE(157)] = 3417,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(152),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(110),
  [11] = {.entry = {.count = 1, .reusable = true}}, SHIFT(127),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(144),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(145),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(84),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(10),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(95),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [25] = {.entry = {.count = 1, .reusable = true}}, SHIFT(130),
  [27] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [29] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [31] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [33] = {.entry = {.count = 1, .reusable = false}}, SHIFT(49),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(101),
  [37] = {.entry = {.count = 1, .reusable = false}}, SHIFT(21),
  [39] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [41] = {.entry = {.count = 1, .reusable = true}}, SHIFT(89),
  [43] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [45] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [47] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0),
  [49] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(10),
  [52] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(95),
  [55] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(13),
  [58] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(130),
  [61] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(11),
  [64] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(12),
  [67] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(49),
  [70] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(49),
  [73] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(101),
  [76] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(21),
  [79] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [81] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [83] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_identifier, 1, 0, 0),
  [85] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_identifier, 1, 0, 0),
  [87] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 3, 0, 0),
  [89] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_block, 3, 0, 0),
  [91] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 2, 0, 0),
  [93] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_block, 2, 0, 0),
  [95] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_statement, 3, 0, 10),
  [97] = {.entry = {.count = 1, .reusable = true}}, SHIFT(123),
  [99] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_statement, 3, 0, 10),
  [101] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_expression_statement, 2, 0, 0),
  [103] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_expression_statement, 2, 0, 0),
  [105] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_path_repeat1, 2, 0, 0),
  [107] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_path_repeat1, 2, 0, 0),
  [109] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_path_repeat1, 2, 0, 0), SHIFT_REPEAT(126),
  [112] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_path, 1, 0, 0),
  [114] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_path, 1, 0, 0),
  [116] = {.entry = {.count = 1, .reusable = true}}, SHIFT(126),
  [118] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_statement, 1, 0, 0),
  [120] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_statement, 1, 0, 0),
  [122] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_path, 2, 0, 0),
  [124] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_path, 2, 0, 0),
  [126] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_return_statement, 3, 0, 11),
  [128] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_return_statement, 3, 0, 11),
  [130] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_statement, 5, 0, 17),
  [132] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_statement, 5, 0, 17),
  [134] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_let_statement, 5, 0, 18),
  [136] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_let_statement, 5, 0, 18),
  [138] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_let_statement, 7, 0, 19),
  [140] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_let_statement, 7, 0, 19),
  [142] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_expression, 3, 0, 0),
  [144] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [146] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_expression, 3, 0, 0),
  [148] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [150] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 2, 0, 0),
  [152] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_expression, 1, 0, 0),
  [154] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_expression, 1, 0, 0),
  [156] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_member_identifier, 1, 0, 0),
  [158] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_member_identifier, 1, 0, 0),
  [160] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_binary_expression, 3, 0, 12),
  [162] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_binary_expression, 3, 0, 12),
  [164] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_unary_expression, 2, 0, 5),
  [166] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_unary_expression, 2, 0, 5),
  [168] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [170] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [172] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call_expression, 2, 0, 6),
  [174] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call_expression, 2, 0, 6),
  [176] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 3, 0, 0),
  [178] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 3, 0, 0),
  [180] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 2, 0, 0),
  [182] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 2, 0, 0),
  [184] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_literal, 1, 0, 0),
  [186] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_literal, 1, 0, 0),
  [188] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call_expression, 4, 0, 14),
  [190] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call_expression, 4, 0, 14),
  [192] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_cast_expression, 4, 0, 15),
  [194] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_cast_expression, 4, 0, 15),
  [196] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_argument_list, 4, 0, 0),
  [198] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_argument_list, 4, 0, 0),
  [200] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_argument_list, 3, 0, 0),
  [202] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_argument_list, 3, 0, 0),
  [204] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_argument_list, 2, 0, 0),
  [206] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_argument_list, 2, 0, 0),
  [208] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [210] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0),
  [212] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(127),
  [215] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(144),
  [218] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(145),
  [221] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [223] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [225] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [227] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 3, 0, 0),
  [229] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [231] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_argument_list_repeat1, 2, 0, 0),
  [233] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [235] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [237] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [239] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [241] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(110),
  [244] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [246] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [248] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [250] = {.entry = {.count = 1, .reusable = true}}, SHIFT(135),
  [252] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_platforms_metadata, 3, 0, 0),
  [254] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_import_statement, 3, 0, 0),
  [256] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_platforms_metadata, 4, 0, 0),
  [258] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_attribute, 2, 0, 1),
  [260] = {.entry = {.count = 1, .reusable = true}}, SHIFT(108),
  [262] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_function_definition_repeat1, 2, 0, 0),
  [264] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_function_definition_repeat1, 2, 0, 0), SHIFT_REPEAT(145),
  [267] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_function_definition, 4, 0, 2),
  [269] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_function_definition, 7, 0, 13),
  [271] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_function_definition, 5, 0, 3),
  [273] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_function_definition, 6, 0, 7),
  [275] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_function_definition, 8, 0, 16),
  [277] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_function_definition, 6, 0, 8),
  [279] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_attribute_arguments, 4, 0, 0),
  [281] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_attribute, 3, 0, 1),
  [283] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_attribute_arguments, 3, 0, 0),
  [285] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_attribute_arguments, 2, 0, 0),
  [287] = {.entry = {.count = 1, .reusable = true}}, SHIFT(75),
  [289] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [291] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_builtin_type, 1, 0, 0),
  [293] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [295] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_platforms_metadata_repeat1, 2, 0, 0),
  [297] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_platforms_metadata_repeat1, 2, 0, 0), SHIFT_REPEAT(21),
  [300] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type, 1, 0, 0),
  [302] = {.entry = {.count = 1, .reusable = true}}, SHIFT(125),
  [304] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_platform_group_repeat1, 2, 0, 0), SHIFT_REPEAT(139),
  [307] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_platform_group_repeat1, 2, 0, 0),
  [309] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [311] = {.entry = {.count = 1, .reusable = false}}, SHIFT(48),
  [313] = {.entry = {.count = 1, .reusable = true}}, SHIFT(103),
  [315] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_string_literal_repeat1, 2, 0, 0),
  [317] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_string_literal_repeat1, 2, 0, 0), SHIFT_REPEAT(102),
  [320] = {.entry = {.count = 1, .reusable = false}}, SHIFT(47),
  [322] = {.entry = {.count = 1, .reusable = true}}, SHIFT(102),
  [324] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [326] = {.entry = {.count = 1, .reusable = true}}, SHIFT(76),
  [328] = {.entry = {.count = 1, .reusable = true}}, SHIFT(117),
  [330] = {.entry = {.count = 1, .reusable = true}}, SHIFT(132),
  [332] = {.entry = {.count = 1, .reusable = true}}, SHIFT(81),
  [334] = {.entry = {.count = 1, .reusable = true}}, SHIFT(93),
  [336] = {.entry = {.count = 1, .reusable = true}}, SHIFT(139),
  [338] = {.entry = {.count = 1, .reusable = true}}, SHIFT(156),
  [340] = {.entry = {.count = 1, .reusable = true}}, SHIFT(90),
  [342] = {.entry = {.count = 1, .reusable = true}}, SHIFT(122),
  [344] = {.entry = {.count = 1, .reusable = true}}, SHIFT(72),
  [346] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [348] = {.entry = {.count = 1, .reusable = true}}, SHIFT(147),
  [350] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_parameter_list_repeat1, 2, 0, 0), SHIFT_REPEAT(117),
  [353] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_parameter_list_repeat1, 2, 0, 0),
  [355] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_argument_list_repeat1, 2, 0, 0), SHIFT_REPEAT(17),
  [358] = {.entry = {.count = 1, .reusable = true}}, SHIFT(154),
  [360] = {.entry = {.count = 1, .reusable = true}}, SHIFT(92),
  [362] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parameter, 3, 0, 4),
  [364] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parameter_list, 3, 0, 0),
  [366] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [368] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parameter_list, 2, 0, 0),
  [370] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [372] = {.entry = {.count = 1, .reusable = true}}, SHIFT(99),
  [374] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_platform_group, 7, 0, 9),
  [376] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parameter_list, 4, 0, 0),
  [378] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_platform_group, 5, 0, 9),
  [380] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [382] = {.entry = {.count = 1, .reusable = true}}, SHIFT(79),
  [384] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_platform_group, 6, 0, 9),
  [386] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [388] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execution_mode, 1, 0, 0),
  [390] = {.entry = {.count = 1, .reusable = true}}, SHIFT(78),
  [392] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [394] = {.entry = {.count = 1, .reusable = true}}, SHIFT(141),
  [396] = {.entry = {.count = 1, .reusable = true}}, SHIFT(133),
  [398] = {.entry = {.count = 1, .reusable = true}}, SHIFT(74),
  [400] = {.entry = {.count = 1, .reusable = true}}, SHIFT(153),
  [402] = {.entry = {.count = 1, .reusable = true}}, SHIFT(96),
  [404] = {.entry = {.count = 1, .reusable = true}}, SHIFT(119),
  [406] = {.entry = {.count = 1, .reusable = true}}, SHIFT(134),
  [408] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [410] = {.entry = {.count = 1, .reusable = true}}, SHIFT(129),
  [412] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef TREE_SITTER_HIDE_SYMBOLS
#define TS_PUBLIC
#elif defined(_WIN32)
#define TS_PUBLIC __declspec(dllexport)
#else
#define TS_PUBLIC __attribute__((visibility("default")))
#endif

TS_PUBLIC const TSLanguage *tree_sitter_kira(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .field_names = ts_field_names,
    .field_map_slices = ts_field_map_slices,
    .field_map_entries = ts_field_map_entries,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
