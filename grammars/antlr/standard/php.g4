grammar php;

start
    : '<?php' top_statement_list '?>' EOF
    ;

reserved_non_modifiers
    : 'include'
    | 'include_once'
    | 'eval'
    | 'require'
    | 'require_once'
    | 'or'
    | 'xor'
    | 'and'
    | 'instanceof'
    | 'new'
    | 'clone'
    | 'exit'
    | 'if'
    | 'elseif'
    | 'else'
    | 'endif'
    | 'echo'
    | 'do'
    | 'while'
    | 'endwhile'
    | 'for'
    | 'endfor'
    | 'foreach'
    | 'endforeach'
    | 'declare'
    | 'enddeclare'
    | 'as'
    | 'try'
    | 'catch'
    | 'finally'
    | 'throw'
    | 'use'
    | 'insteadof'
    | 'global'
    | 'var'
    | 'unset'
    | 'isset'
    | 'empty'
    | 'continue'
    | 'goto'
    | 'function'
    | 'const'
    | 'return'
    | 'print'
    | 'yield'
    | 'list'
    | 'switch'
    | 'endswitch'
    | 'case'
    | 'default'
    | 'break'
    | 'array'
    | 'callable'
    | 'extends'
    | 'implements'
    | 'namespace'
    | 'trait'
    | 'interface'
    | t_class
    | '__class__'
    | '__trait__'
    | '__function__'
    | '__method__'
    | '__line__'
    | '__file__'
    | '__dir__'
    | '__namespace__'
    ;

semi_reserved
    : reserved_non_modifiers
    | 'static'
    | 'abstract'
    | 'final'
    | 'private'
    | 'protected'
    | 'public'
    ;

identifier
    : t_string
    | semi_reserved
    ;

top_statement_list
    : top_statement_list top_statement
    | /* empty */
    ;

namespace_name
    : t_string
    | namespace_name t_NS_separator t_string
    ;

name
    : namespace_name
    | 'namespace' t_NS_separator namespace_name
    | t_NS_separator namespace_name
    ;

top_statement
    : statement
    | function_declaration_statement
    | class_declaration_statement
    | trait_declaration_statement
    | interface_declaration_statement
    | '__halt_compiler()' ';'
    | 'namespace' namespace_name ';'
    | 'namespace' namespace_name '{' top_statement_list '}'
    | 'namespace' '{' top_statement_list '}'
    | 'use' mixed_group_use_declaration ';'
    | 'use' use_type group_use_declaration ';'
    | 'use' use_declarations ';'
    | 'use' use_type use_declarations ';'
    | 'const' const_list ';'
    ;

use_type
    : 'function'
    | 'const'
    ;

group_use_declaration
    : namespace_name t_NS_separator '{' unprefixed_use_declarations possible_comma '}'
    | t_NS_separator namespace_name t_NS_separator '{' unprefixed_use_declarations possible_comma '}'
    ;

mixed_group_use_declaration
    : namespace_name t_NS_separator '{' inline_use_declarations possible_comma '}'
    | t_NS_separator namespace_name t_NS_separator '{' inline_use_declarations possible_comma '}'
    ;

possible_comma
    : ','
    | /* empty */
    ;

inline_use_declarations
    : inline_use_declarations ',' inline_use_declaration
    | inline_use_declaration
    ;

unprefixed_use_declarations
    : unprefixed_use_declarations ',' unprefixed_use_declaration
    | unprefixed_use_declaration
    ;

use_declarations
    : use_declarations ',' use_declaration
    | use_declaration
    ;

inline_use_declaration
    : unprefixed_use_declaration
    | use_type unprefixed_use_declaration
    ;

unprefixed_use_declaration
    : namespace_name
    | namespace_name 'as' t_string
    ;

use_declaration
    : unprefixed_use_declaration
    | t_NS_separator unprefixed_use_declaration
    ;

const_list
    : const_list ',' const_decl
    | const_decl
    ;

inner_statement_list
    : inner_statement_list inner_statement
    | 
    ;

inner_statement
    : statement
    | function_declaration_statement
    | class_declaration_statement
    | trait_declaration_statement
    | interface_declaration_statement
    | '__halt_compiler()' ';'
    ;

statement
    : '{' inner_statement_list '}'
    | if_stmt
    | alt_if_stmt
    | 'while' '(' expr ')' while_statement
    | 'do' statement 'while' '(' expr ')' ';'
    | 'for' '(' for_exprs ';' for_exprs ';' for_exprs ')' for_statement
    | 'switch' '(' expr ')' switch_case_list
    | 'break' optional_expr ';'
    | 'continue' optional_expr ';'
    | 'return' optional_expr ';'
    | 'global' global_var_list ';'
    | 'static' static_var_list ';'
    | 'echo' echo_expr_list ';'
    | expr ';'
    | 'unset' '(' unset_variables possible_comma ')' ';'
    | 'foreach' '(' expr 'as' foreach_variable ')' foreach_statement
    | 'foreach' '(' expr 'as' foreach_variable '=' '>' foreach_variable ')' foreach_statement
    | 'declare' '(' const_list ')'
    | declare_statement
    | ';'
    | 'try' '{' inner_statement_list '}' catch_list finally_statement
    | 'throw' expr ';'
    | 'goto' t_string ';'
    | t_string ':'
    ;

catch_list
    : /* empty */
    | catch_list 'catch' '(' catch_name_list t_variable ')' '{' inner_statement_list '}'
    ;

catch_name_list
    : name
    | catch_name_list '|' name
    ;

finally_statement
    : /* empty */
    | 'finally' '{' inner_statement_list '}'
    ;

unset_variables
    : unset_variable
    | unset_variables ',' unset_variable
    ;

unset_variable
    : variable
    ;

function_declaration_statement
    : 'function' returns_ref t_string '(' parameter_list ')' return_type '{' inner_statement_list '}'
    ;

IS_reference
    : '&'
    ;

IS_variadic
    : '...'
    ;

class_declaration_statement
    : class_modifiers t_class t_string extends_from implements_list '{' class_statement_list '}'
    | t_class t_string extends_from implements_list '{' class_statement_list '}'
    ;

class_modifiers
    : class_modifier
    | class_modifiers class_modifier
    ;

class_modifier
    : 'abstract'
    | 'final'
    ;

trait_declaration_statement
    : 'trait' t_string '{' class_statement_list '}'
    ;

interface_declaration_statement
    : 'interface' t_string interface_extends_list '{' class_statement_list '}'
    ;

extends_from
    : /* empty */
    | 'extends' name
    ;

interface_extends_list
    : /* empty */
    | 'extends' name_list
    ;

implements_list
    : /* empty */
    | 'implements' name_list
    ;

foreach_variable
    : variable
    | '&' variable
    | 'list' '(' array_pair_list ')'
    | '[' array_pair_list ']'
    ;

for_statement
    : statement
    | ':' inner_statement_list 'endfor' ';'
    ;

foreach_statement
    : statement
    | ':' inner_statement_list 'endforeach' ';'
    ;

declare_statement
    : ':' inner_statement_list 'enddeclare' ';'
    ;

switch_case_list
    : '{' case_list '}'
    | '{' ';' case_list '}'
    | ':' case_list 'endswitch' ';'
    | ':' ';' case_list 'endswitch' ';'
    ;

case_list
    : /* empty */
    | case_list 'case' expr case_separator inner_statement_list
    | case_list 'default' case_separator inner_statement_list
    ;

case_separator
    : ':'
    | ';'
    ;

while_statement
    : statement
    | ':' inner_statement_list 'endwhile' ';'
    ;

if_stmt_without_else
    : 'if' '(' expr ')' statement
    | if_stmt_without_else 'elseif' '(' expr ')' statement
    ;

if_stmt
    : if_stmt_without_else
    | if_stmt_without_else 'else' statement
    ;

alt_if_stmt_without_else
    : 'if' '(' expr ')' ':' inner_statement_list
    | alt_if_stmt_without_else 'elseif' '(' expr ')' ':' inner_statement_list
    ;

alt_if_stmt
    : alt_if_stmt_without_else 'endif' ';'
    | alt_if_stmt_without_else 'else' ':' inner_statement_list 'endif' ';'
    ;

parameter_list
    : non_empty_parameter_list
    | /* empty */
    ;

non_empty_parameter_list
    : parameter
    | non_empty_parameter_list ',' parameter
    ;

parameter
    : optional_type IS_reference? IS_variadic? t_variable
    | optional_type IS_reference? IS_variadic? t_variable '=' expr
    ;

optional_type
    : /* empty */
    | type_expr
    ;

type_expr
    : type
    | '?' type
    ;

type
    : 'array'
    | 'callable'
    | name
    ;

return_type
    : /* empty */
    | ':' type_expr
    ;

argument_list
    : '(' ')'
    | '(' non_empty_argument_list possible_comma ')'
    ;

non_empty_argument_list
    : argument
    | non_empty_argument_list ',' argument
    ;

argument
    : expr
    | '...' expr
    ;

global_var_list
    : global_var_list ',' global_var
    | global_var
    ;

global_var
    : simple_variable
    ;

static_var_list
    : static_var_list ',' static_var
    | static_var
    ;

static_var
    : t_variable
    | t_variable '=' expr
    ;

class_statement_list
    : class_statement_list class_statement
    | /* empty */
    ;

class_statement
    : variable_modifiers property_list ';'
    | method_modifiers 'const' class_const_list ';'
    | 'use' name_list trait_adaptations
    | method_modifiers 'function' returns_ref identifier '(' parameter_list ')' return_type method_body
    ;

name_list
    : name
    | name_list ',' name
    ;

trait_adaptations
    : ';'
    | '{' '}'
    | '{' trait_adaptation_list '}'
    ;

trait_adaptation_list
    : trait_adaptation
    | trait_adaptation_list trait_adaptation
    ;

trait_adaptation
    : trait_precedence ';'
    | trait_alias ';'
    ;

trait_precedence
    : absolute_trait_method_reference 'insteadof' name_list
    ;

trait_alias
    : trait_method_reference 'as' t_string
    | trait_method_reference 'as' reserved_non_modifiers
    | trait_method_reference 'as' member_modifier identifier
    | trait_method_reference 'as' member_modifier
    ;

trait_method_reference
    : identifier
    | absolute_trait_method_reference
    ;

absolute_trait_method_reference
    : name '::' identifier
    ;

method_body
    : ';'
    | '{' inner_statement_list '}'
    ;

variable_modifiers
    : non_empty_member_modifiers
    | 'var'
    ;

method_modifiers
    : /* empty */
    | non_empty_member_modifiers
    ;

non_empty_member_modifiers
    : member_modifier
    | non_empty_member_modifiers member_modifier
    ;

member_modifier
    : 'public'
    | 'protected'
    | 'private'
    | 'static'
    | 'abstract'
    | 'final'
    ;

property_list
    : property_list ',' property
    | property
    ;

property
    : t_variable
    | t_variable '=' expr
    ;

class_const_list
    : class_const_list ',' class_const_decl
    | class_const_decl
    ;

class_const_decl
    : identifier '=' expr
    ;

const_decl
    : t_string '=' expr
    ;

echo_expr_list
    : echo_expr_list ',' echo_expr
    | echo_expr
    ;

echo_expr
    : expr
    ;

for_exprs
    : /* empty */
    | non_empty_for_exprs
    ;

non_empty_for_exprs
    : non_empty_for_exprs ',' expr
    | expr
    ;

anonymous_class
    : t_class ctor_arguments extends_from implements_list '{' class_statement_list '}'
    ;

new_expr
    : 'new' class_name_reference ctor_arguments
    | 'new' anonymous_class
    ;

expr
    : variable 
    | 'list' '(' array_pair_list ')' '=' expr
    | '[' array_pair_list ']' '=' expr
    | variable '=' expr
    | variable '=' '&' variable
    | 'clone' expr
    | variable '+' '=' expr
    | variable '-' '=' expr
    | variable '*' '=' expr
    | variable '*' '*' '=' expr
    | variable '/' '=' expr
    | variable '.' '=' expr
    | variable '%' '=' expr
    | variable '&' '=' expr
    | variable '|' '=' expr
    | variable '^' '=' expr
    | variable '<' '<' '=' expr
    | variable '>' '>' '=' expr
    | variable '+' '+'
    | '+' '+' variable
    | variable '-' '-'
    | '-' '-' variable
    | expr '|' '|' expr
    | expr '&' '&' expr
    | expr 'or' expr
    | expr 'and' expr
    | expr 'xor' expr
    | expr '|' expr
    | expr '&' expr
    | expr '^' expr
    | expr '.' expr
    | expr '+' expr
    | expr '-' expr
    | expr '*' expr
    | expr '*' '*' expr
    | expr '/' expr
    | expr '%' expr
    | expr '<' '<' expr
    | expr '>' '>' expr
    | '+' expr
    | '-' expr
    | '!' expr
    | '~' expr
    | expr '=' '=' '=' expr
    | expr '!' '=' '=' expr
    | expr '=' '=' expr
    | expr '!' '=' expr
    | expr '<' expr
    | expr '<' '=' expr
    | expr '>' expr
    | expr '>' '=' expr
    | expr '<' '=' '>' expr
    | expr 'instanceof' class_name_reference
    | '(' expr ')'
    | new_expr
    | expr '?' expr ':' expr
    | expr '?' ':' expr
    | expr '?' '?' expr
    | internal_functions_IN_yacc
    | '(int)' expr
    | '(double)' expr
    | '(string)' expr
    | '(array)' expr
    | '(object)' expr
    | '(bool)' expr
    | '(unset)' expr
    | 'exit' exit_expr
    | '@' expr
    | scalar
    | '`' backticks_expr '`'
    | 'print' expr
    | 'yield'
    | 'yield' expr
    | 'yield' expr '=' '>' expr
    | 'yield from' expr
    | 'function' returns_ref '(' parameter_list ')' lexical_vars return_type '{' inner_statement_list '}'
    | 'static' 'function' returns_ref '(' parameter_list ')' lexical_vars return_type '{' inner_statement_list '}'
    ;

returns_ref
    : /* empty */
    | '&'
    ;

lexical_vars
    : /* empty */
    | 'use' '(' lexical_var_list ')'
    ;

lexical_var_list
    : lexical_var_list ',' lexical_var
    | lexical_var
    ;

lexical_var
    : t_variable
    | '&' t_variable
    ;

class_name
    : 'static'
    | name
    ;

class_name_reference
    : class_name
    | new_variable
    ;

exit_expr
    : /* empty */
    | '(' optional_expr ')'
    ;

backticks_expr
    : /* empty */
    | encaps_list
    ;

ctor_arguments
    : /* empty */
    | argument_list
    ;

dereferencable_scalar
    : 'array(' array_pair_list ')'
    | '[' array_pair_list ']'
    | t_constant_encapsed_string
    ;

scalar
    : LNUMBER
    | DNUMBER
    | '__line__'
    | '__file__'
    | '__dir__'
    | '__trait__'
    | '__method__'
    | '__function__'
    | '__namespace__'
    | '__class__'
    | '"' encaps_list '"'
    | dereferencable_scalar
    | constant
    ;

constant
    : name
    | class_name '::' identifier
    | variable_class_name '::' identifier
    | callable_variable '::' identifier
    | static_member '::' identifier
    | callable_variable '::' simple_variable '::' identifier
    ;

optional_expr
    : /* empty */
    | expr
    ;

variable_class_name
    : dereferencable
    ;

dereferencable
    : dereferencable '->' property_name
    | simple_variable
    | dereferencable '[' optional_expr ']'
    | dereferencable '{' expr '}'
    | name argument_list
    | '(' expr ')'
    | dereferencable_scalar
    ;

callable_variable
    : simple_variable
    | dereferencable '[' optional_expr ']'
    | callable_variable '[' optional_expr ']'
    | static_member '[' optional_expr ']'
    | callable_variable '::' simple_variable '[' optional_expr ']'


    | name '[' optional_expr ']'
    | class_name '::' identifier '[' optional_expr ']'
    | variable_class_name '::' identifier '[' optional_expr ']'
    | callable_variable '::' identifier '[' optional_expr ']'
    | static_member '::' identifier '[' optional_expr ']'
    | callable_variable '::' simple_variable '::' identifier '[' optional_expr ']'

   
    | dereferencable '{' expr '}'
    | callable_variable '{' expr '}'
    | static_member '{' expr '}'
    | callable_variable '::' simple_variable '{' expr '}'

    | dereferencable '->' property_name argument_list
    | callable_variable '->' property_name argument_list
    | static_member '->' property_name argument_list
    | callable_variable '::' simple_variable '->' property_name argument_list

    | name argument_list
    | class_name '::' member_name argument_list
    | variable_class_name '::' member_name argument_list
    | callable_variable '::' member_name argument_list
    | static_member '::' member_name argument_list
    | callable_variable '::' simple_variable '::' member_name argument_list

    | callable_variable argument_list
    | '(' expr ')' argument_list
    | dereferencable_scalar argument_list
    ;

variable
    : callable_variable
    | static_member
    | callable_variable '::' simple_variable
    | dereferencable '->' property_name
    | callable_variable '->' property_name
    | static_member '->' property_name
    | callable_variable '::' simple_variable '->' property_name
    ;

simple_variable
    : t_variable
    | '$' '{' expr '}'
    | '$' simple_variable
    ;

static_member
    : class_name '::' simple_variable
    | variable_class_name '::' simple_variable
    | static_member '::' simple_variable
    ;

new_variable
    : simple_variable
    | new_variable '[' optional_expr ']'
    | new_variable '{' expr '}'
    | new_variable '->' property_name
    | class_name '::' simple_variable
    | new_variable '::' simple_variable
    ;

member_name
    : identifier
    | '{' expr '}'
    | simple_variable
    ;

property_name
    : t_string
    | '{' expr '}'
    | simple_variable
    ;

array_pair_list
    : non_empty_array_pair_list
    ;

possible_array_pair
    : /* empty */
    | array_pair
    ;

non_empty_array_pair_list
    : non_empty_array_pair_list ',' possible_array_pair
    | possible_array_pair
    ;

array_pair
    : expr '=' '>' expr
    | expr
    | expr '=' '>' '&' variable
    | '&' variable
    | expr '=' '>' 'list' '(' array_pair_list ')'
    | 'list' '(' array_pair_list ')'
    ;

encaps_list
    : encaps_list encaps_var
    | encaps_var
    ;

encaps_var
    : t_variable
    | t_variable '[' encaps_var_offset ']'
    | t_variable '->' t_string
    | '{' '$' variable '}'
    ;

encaps_var_offset
    : t_string
    | t_variable
    ;

internal_functions_IN_yacc
    : 'isset' '(' isset_variables possible_comma ')'
    | 'empty' '(' expr ')'
    | 'include' expr
    | 'include_once' expr
    | 'eval' '(' expr ')'
    | 'require' expr
    | 'require_once' expr
    ;

isset_variables
    : isset_variable
    | isset_variables ',' isset_variable
    ;

isset_variable
    : expr
    ;

t_constant_encapsed_string
    : STR_1
    | STR_2
    ;

STR_1
    : '"' ('\'' | '\\"' | [a-zA-Z_]|[0-9]|[\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007\u0008\u0009\u000A\u000B\u000C\u000D\u000E\u000F\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017\u0018\u0019\u001A\u001B\u001C\u001D\u001E\u001F\u0020\u0021\u0023\u0024\u0025\u0026\u0028\u0029\u002A\u002B\u002C\u002D\u002E\u002F\u003A\u003B\u003C\u003D\u003E\u003F\u0040\u005B\u005C\u005D\u005E\u0060\u007B\u007C\u007D\u007E\u007F])* '"'
    ;

STR_2
    : '\'' ('"' |'\\\''| [a-zA-Z_]|[0-9]|[\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007\u0008\u0009\u000A\u000B\u000C\u000D\u000E\u000F\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017\u0018\u0019\u001A\u001B\u001C\u001D\u001E\u001F\u0020\u0021\u0023\u0024\u0025\u0026\u0028\u0029\u002A\u002B\u002C\u002D\u002E\u002F\u003A\u003B\u003C\u003D\u003E\u003F\u0040\u005B\u005C\u005D\u005E\u0060\u007B\u007C\u007D\u007E\u007F])* '\''
    ;

t_class
    : 'class'
    | 'Class'
    ;

t_variable
    : '$' t_string
    | '$' semi_reserved
    ;

LNUMBER
    : ([0-9]+) | ( '0x' [0-9a-fA-F]+)
    ;

DNUMBER
    : ([0-9]+.[0-9]+([eE][0-9]+)?[fFdD]?)|([0-9]+[eE][0-9]+[fFdD]?)
    ;

t_NS_separator
    : '\\'
    ;

t_string
    : T_STRING
    | letter
    | semi_reserved
    ;

T_STRING
    : [a-zA-Z_][a-zA-Z_0-9]*
    ;

letter
    : '_'
    | 'a'
    | 'b'
    | 'c'
    | 'd'
    | 'e'
    | 'f'
    | 'g'
    | 'h'
    | 'i'
    | 'j'
    | 'k'
    | 'l'
    | 'm'
    | 'n'
    | 'o'
    | 'p'
    | 'q'
    | 'r'
    | 's'
    | 't'
    | 'u'
    | 'v'
    | 'w'
    | 'x'
    | 'y'
    | 'z'
    | 'A'
    | 'B'
    | 'C'
    | 'D'
    | 'E'
    | 'F'
    | 'G'
    | 'H'
    | 'I'
    | 'J'
    | 'K'
    | 'L'
    | 'M'
    | 'N'
    | 'O'
    | 'P'
    | 'Q'
    | 'R'
    | 'S'
    | 'T'
    | 'U'
    | 'V'
    | 'W'
    | 'X'
    | 'Y'
    | 'Z'
    ;

WHITESPACE
    : (' ' | '\n' | '\t' | '\r')+ -> skip
     ;
