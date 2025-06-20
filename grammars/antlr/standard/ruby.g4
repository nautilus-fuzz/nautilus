grammar ruby;

start
    : program EOF
    ;

program
    : compstmt
    ;

compstmt
    : stmt term_expr_rep term_opt
    ;

stmt
    : call 'do' block_var_opt compstmt 'end'
    | 'undef' fname
    | 'alias' fname fname
    | stmt 'if' expr
    | stmt 'while' expr
    | stmt 'unless' expr
    | stmt 'until' expr
    | 'begin' '{' compstmt '}'
    | 'end' '{' compstmt '}'
    | lhs '=' command 'do' block_var_opt compstmt 'end'
    | lhs '=' command
    | expr
    ;

expr
    : mlhs '=' mrhs
    | 'return' call_args
    | 'yield' call_args
    | expr 'and' expr
    | expr 'or' expr
    | 'not' expr
    | command
    | '!' command
    | arg
    ;

call
    : function
    | command
    ;

command
    : operation call_args
    | primary '.' operation call_args
    | primary ':' ':' operation call_args
    | 'super' call_args
    ;

function
    : operation call_args_brackets_opt
    | primary '.' operation '(' call_args_opt ')'
    | primary ':' ':' operation '(' call_args_opt ')'
    | primary '.' operation
    | primary ':' ':' operation
    | 'super' '(' call_args_opt ')'
    | 'super'
    ;

arg
    : lhs '=' arg
    | lhs op_asgn arg
    | arg op_bin arg
    | '+' arg
    | '-' arg
    | '!' arg
    | '~' arg
    | 'defined?' arg
    | primary
    ;

primary
    : '(' compstmt ')'
    | literal
    | variable
    | primary ':' ':' identifier
    | ':' ':' identifier
    | primary '[' args_opt ']'
    | '[' args_comma_opt ']'
    | '{' args_assocs_comma_opt '}'
    | 'return' call_args_brackets_opt
    | 'yield' call_args_brackets_opt
    | 'defined?' '(' arg ')'

    | operation call_args_brackets_opt function_body_opt
    | primary '.' operation '(' call_args_opt ')' function_body_opt
    | primary ':' ':' operation '(' call_args_opt ')' function_body_opt
    | primary '.' operation function_body_opt
    | primary ':' ':' operation function_body_opt
    | 'super' '(' call_args_opt ')' function_body_opt
    | 'super' function_body_opt

    | 'if' expr then compstmt elsif_rep else_opt 'end'
    | 'unless' expr then compstmt else_opt 'end'
    | 'while' expr do compstmt 'end'
    | 'until' expr do compstmt 'end'
    | 'case' compstmt when_rep else_opt 'end'
    | 'for' block_var 'in' expr do compstmt 'end'
    | 'begin' compstmt rescue_rep else_opt ensure_opt 'end'
    | 'class' identifier inheritance_opt compstmt 'end'
    | 'module' identifier compstmt 'end'
    | 'def' fname argdecl compstmt 'end'
    | 'def' singleton dots fname argdecl compstmt 'end'
    ;

// created as part of fixing mutually left-recursive rules [function, primary]
function_body_opt
    : '{' block_var_opt compstmt '}'
    | 
    ;

when_rep
    : 'when' mrhs then compstmt
    | 'when' mrhs then compstmt when_rep
    ;

then
    : term
    | 'then'
    | term 'then'
    ;

do
    : term
    | 'do'
    | term 'do'
    ;

block_var
    : lhs
    | mlhs
    ;

mlhs
    : mlhs_item ',' mlhs_item_opt '*' lhs
    | mlhs_item ',' mlhs_item_opt '*'
    | mlhs_item ',' mlhs_item_opt
    | '*' lhs
    ;

mlhs_item
    : lhs
    | '(' mlhs ')'
    ;

lhs
    : primary '[' args_opt ']'
    | primary '.' identifier
    | variable
    ;

mrhs
    : args arg_ast_opt
    | '*' arg
    ;

call_args
    : args
    | args assocs_opt arg_ast_opt arg_amp_opt
    | assocs arg_ast_opt arg_amp_opt
    | '*' arg arg_amp_opt
    | '&' arg
    | command
    ;

args
    : arg ',' args
    | arg
    ;

argdecl
    : '(' arglist ')'
    | arglist term
    ;

arglist
    : identifier identifier_rep identifier_ast_opt identifier_amp_opt
    | '*' identifier identifier_amp_opt
    | '&' identifier
    | /* empty */
    ;

singleton
    : variable
    | '(' expr ')'
    ;

assocs
    : assoc ',' assocs
    | assoc
    ;

assoc
    : arg '=' '>' arg
    ;

variable
    : varname
    | 'nil'
    | 'self'
    ;

literal
    : numeric
    | symbol
    | STRING
    | STRING2
    | HERE_DOC
    | REGEXP
    ;

term_opt
    : term
    | /* empty */
    ;

term
    : ';'
    | 
    ;

term_expr_rep
    : term expr term_expr_rep
    | /* empty */
    ;

block_var_opt
    : '|' block_var '|'
    | '|' '|'
    | /* empty */
    ;

arg_ast_opt
    : ',' '*' arg
    | /* empty */
    ;

arg_amp_opt
    : ',' '&' arg
    | /* empty */
    | ',' '&' arg
    | /* empty */
    ;

assocs_opt
    : ',' assocs
    | /* empty */
    ;

call_args_brackets_opt
    : '(' call_args_opt ')'
    | /* empty */
    ;

call_args_opt
    : call_args
    | /* empty */
    ;

args_opt
    : args
    | /* empty */
    ;

args_comma_opt
    : args comma_opt
    | /* empty */
    ;

comma_opt
    : ','
    | /* empty */
    ;

args_assocs_comma_opt
    : args
    | assocs comma_opt
    | /* empty */
    ;

elsif_rep
    : 'elsif ' expr then compstmt elsif_rep
    | /* empty */
    ;

else_opt
    : 'else' compstmt
    | /* empty */
    ;

inheritance_opt
    : '<' identifier
    | /* empty */
    ;

rescue_rep
    : 'rescue' args_opt do compstmt rescue_rep
    | /* empty */
    ;

ensure_opt
    : 'ensure' compstmt
    | /* empty */
    ;

dots
    : '.'
    | ':' ':'
    ;

mlhs_item_opt
    : mlhs_item_rep
    | /* empty */
    ;

mlhs_item_rep
    : mlhs_item ',' mlhs_item_rep
    | mlhs_item
    ;

identifier_ast_opt
    : ',' '*' identifier
    | /* empty */
    ;

identifier_amp_opt
    : ',' '&' identifier
    | /* empty */
    ;

identifier_rep
    : ',' identifier identifier_rep
    | /* empty */
    ;

varname
    : global
    | '@' identifier
    | identifier
    ;

op_asgn
    : '+' '='
    | '-' '='
    | '*' '='
    | '/' '='
    | '%' '='
    | '**='
    | '&' '='
    | '|' '='
    | '^' '='
    | '<<='
    | '>>='
    | '&&='
    | '||='
    ;

op_bin
    : '.' '.'
    | '.' '.' '.'
    | '+'
    | '-'
    | '*'
    | '/'
    | '%'
    | '*' '*'
    | '|'
    | '^'
    | '&'
    | '<=>'
    | '>'
    | '>' '='
    | '<'
    | '<' '='
    | '=' '='
    | '==='
    | '!' '='
    | '=' '~'
    | '!' '~'
    | '<' '<'
    | '>' '>'
    | '&' '&'
    | '|' '|'
    ;

fname_op
    : '.' '.'
    | '|'
    | '^'
    | '&'
    | '<=>'
    | '=' '='
    | '==='
    | '=' '~'
    | '>'
    | '>' '='
    | '<'
    | '<' '='
    | '+'
    | '-'
    | '*'
    | '/'
    | '%'
    | '*' '*'
    | '<' '<'
    | '>' '>'
    | '~'
    | '+' '@'
    | '-' '@'
    | '[' ']'
    | '[' ']' '='
    ;

symbol
    : ':' fname
    | ':' varname
    ;

fname
    : identifier
    | fname_op
    ;

operation
    : identifier
    | identifier '!'
    | identifier '?'
    ;

global
    : GLOBAL
    ;

STRING
    : '"' ('\\"' | [\u0000-\u0021\u0023-\u007F])* '"'
    | '\'' ('\\\'' | [\u0000-\u0026\u0028-\u007F])* '\''
    | '`' ('\\`' | [\u0000-\u005F\u0061-\u007F])* '`'
    ;

STRING2
    : '%' [Qqx] '{' ('\\{' | '\\}' | [\u0000-\u007A\u007C\u007E-\u007F])* '}'
    | '%' [Qqx] '[' ('\\[' | '\\]' | [\u0000-\u005A\u005C\u005E-\u007F])* ']'
    | '%' [Qqx] '(' ('\\(' | '\\)' | [\u0000-\u0027\u002A-\u007F])* ')'
    ;

HERE_DOC
    : '<<EOF\n' [\u0000-\u007F]*? '\nEOF'
    ;

REGEXP
    : '/' ('\\/'|[\u0000-\u002E\u0030-\u007F])* '/' [iop]
    | '%' 'r' '{' ('\\{' | '\\}' | [\u0000-\u007A\u007C\u007E-\u007F])* '}'
    | '%' 'r' '[' ('\\[' | '\\]' | [\u0000-\u005A\u005C\u005E-\u007F])* ']'
    ;

identifier
    : letter
    | IDENTIFIER
    ;

IDENTIFIER
    : [a-zA-Z_][a-zA-Z0-9_]*
    ;

numeric
    : hex
    | decimal
    ;

hex
    : hexd
    | HEX
    ;

HEX
    : '0' [xX] [0-9a-fA-F]+ ('.' [0-9a-fA-F]+)? ([pP] '-'? [0-9a-fA-F]+)?
    ;

GLOBAL
    : '$' '-'? [\u0000-\u007F]
    | '$' IDENTIFIER
    ;

decimal
    : decimaldigit
    | DECIMAL
    ;

DECIMAL
    : [+\-]? [0-9]+ ('.' [0-9]+)? ([eE] '-'? [0-9]+)?
    ;

decimaldigit
    : '0'
    | '1'
    | '2'
    | '3'
    | '4'
    | '5'
    | '6'
    | '7'
    | '8'
    | '9'
    ;

hexd
    : 'a'
    | 'b'
    | 'c'
    | 'd'
    | 'e'
    | 'f'
    | 'A'
    | 'B'
    | 'C'
    | 'D'
    | 'E'
    | 'F'
    | '0'
    | '1'
    | '2'
    | '3'
    | '4'
    | '5'
    | '6'
    | '7'
    | '8'
    | '9'
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

bracket
    : '('
    | ')'
    | '['
    | ']'
    | '{'
    | '}'
    ;

WHITESPACE
    : (' ' | '\n' | '\t' | '\r')+ -> skip
    ;
