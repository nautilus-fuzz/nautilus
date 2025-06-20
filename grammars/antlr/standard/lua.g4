grammar lua;

start
    : program
    ;

program
    : statementlist retstat
    ;

statementlist
    : statement statementlist
    | /* empty */
    ;

statement
    : ';'
    | varlist '=' exprlist
    | 'local' attrlist '=' exprlist
    | 'local' attrlist
    | 'do' program 'end'
    | function
    | functioncall
    | conditional
    | loop
    | '::' labelname '::'
    ;

labelname
    : 'label_a'
    | 'label_b'
    ;

retstat
    : 'return'
    | 'return' ';'
    | 'return' exprlist
    | 'return' exprlist ';'
    | /* empty */
    ;

function
    : funcdef '(' function_args ')' program 'end'
    ;

funcdef
    : 'function' funcnamelist
    | 'function' funcnamelist ':' name
    | 'local' 'function' name
    ;

lambda
    : 'function' '(' function_args ')' program 'end'
    ;

function_args
    : /* empty */
    | function_arglist
    ;

function_arglist
    : name ',' function_arglist
    | name
    | '.' '.' '.'
    ;

args
    : '(' exprlistopt ')'
    | tableconstructor
    | literalstring
    ;

conditional
    : 'if' expr 'then' program 'end'
    | 'if' expr 'then' program elseif 'else' program 'end'
    | 'if' expr 'then' program elseif 'end'
    ;

elseif
    : 'elseif' expr 'then' program elseif
    | /* empty */
    ;

loop
    : 'while' expr 'do' program 'end'
    | 'for' name '=' expr ',' exprlist 'do' program 'end'
    | 'for' namelist 'in' exprlist 'do' program 'end'
    | 'repeat' program 'until' expr
    ;

exprlist
    : expr ',' exprlist
    | expr
    ;

exprlistopt
    : exprlist
    | /* empty */
    ;

expr
    : 'nil'
    | 'false'
    | 'true'
    | numeral
    | literalstring
    | tableconstructor
    | expr binop expr
    | unop expr
    | lambda
    | functioncall
    | prefix_expr
    | '.' '.' '.'
    ;

prefix_expr
    : name
    | prefix_expr '[' expr ']'
    | prefix_expr . name
    | prefix_expr args
    | prefix_expr ':' name args
    | '(' expr ')'
    ;

functioncall
    : prefix_expr args
    | prefix_expr ':' name args
    ;

binop
    : '+'
    | '-'
    | '*'
    | '/'
    | '//'
    | '^'
    | '%'
    | '&'
    | '~'
    | '|'
    | '>>'
    | '<<'
    | '.' '.'
    | '<'
    | '<='
    | '>'
    | '>='
    | '=='
    | '~='
    | 'and'
    | 'or'
    ;

unop
    : '-'
    | 'not'
    | '#'
    | '~'
    ;

tableconstructor
    : '{' fieldlist '}'
    | '{}'
    ;

fieldlist
    : field fieldsep fieldlist
    | field fieldsepopt
    ;

field
    : '[' expr ']' '=' expr
    | name '=' expr
    | expr
    ;

fieldsep
    : ','
    | ';'
    ;

fieldsepopt
    : fieldsep
    | /* empty */
    ;

funcnamelist
    : name '.' funcnamelist
    | name
    ;

namelist
    : name ',' namelist
    | name
    ;

name
    : 'var_' letter
    ;

varlist
    : var ',' varlist
    | var
    ;

var
    : name
    | prefix_expr '[' expr ']'
    | prefix_expr . name
    ;

char
    : letter
    | decimaldigit
    ;

attrlist
    : name attr ',' namelist
    | name attr
    ;

attr
    : '<' name '>'
    | /* empty */
    ;

literalstring
    : shortstring
    | longstring
    ;

shortstring
    : '"' string '"'
    | '\''string'\''
    ;

longstring
    : '[' '[' string ']' ']'
    | '[=[' string ']=]'
    | '[==[' string ']==]'
    ;

string
    : /* empty */
    | strchr string
    ;

strchr
    : decimaldigit
    | letter
    | '/'
    | '//'
    | '.'
    | '$'
    | escapesequence
    ;

escapesequence
    : '\\a'
    | '\\b'
    | '\\f'
    | '\\n'
    | '\\r'
    | '\\t'
    | '\\v'
    | '\\z'
    | '\\\''
    | '\\"'
    | '\\x' hexdigit hexdigit
    | '\\u{' hexdigits '}'
    ;

exponent
    : 'e'
    | 'E'
    ;

bin_exponent
    : 'p'
    | 'P'
    ;

sign
    : '-'
    | /* empty */
    ;

numeral
    : decimal
    | '0' 'x' hexadecimal
    | '0' 'X' hexadecimal
    ;

decimal
    : decimaldigits
    | decimaldigits exponent sign decimaldigits
    | decimaldigits '.' decimaldigits
    | '.' decimaldigits
    | '.' decimaldigits exponent sign decimaldigits
    | decimaldigits '.' decimaldigits exponent sign decimaldigits
    ;

hexadecimal
    : hexdigits
    | hexdigits bin_exponent sign hexdigits
    | hexdigits '.' hexdigits
    | hexdigits '.' hexdigits bin_exponent sign hexdigits
    ;

decimaldigits
    : decimaldigit decimaldigits
    | decimaldigit
    ;

hexdigits
    : hexdigit hexdigits
    | hexdigit
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

hexdigit
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
    | decimaldigit
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

