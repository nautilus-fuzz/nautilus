ctx.rule(u'START',u'{PROGRAM}')

ctx.rule(u'PROGRAM', u'{S_EXPRESSION}')
ctx.rule(u'S_EXPRESSION', u'{LIST}')
ctx.rule(u'S_EXPRESSION', u'{ATOM}')

ctx.rule(u'ATOM', u'{SYMBOL}')
ctx.rule(u'ATOM', u'{NUMBER}')

ctx.rule(u'LIST', u'(quote {S_EXPRESSION})')
ctx.rule(u'LIST', u'(lambda {BOUND_VAR} {BODY})')
ctx.rule(u'LIST', u'(and {S_EXPRESSION} {S_EXPRESSIONS})')
ctx.rule(u'LIST', u'(or {S_EXPRESSION} {S_EXPRESSIONS})')
ctx.rule(u'LIST', u'(begin {S_EXPRESSION} {S_EXPRESSIONS})')
ctx.rule(u'LIST', u'(rec {S_EXPRESSION} {S_EXPRESSIONS})')

ctx.rule(u'S_EXPRESSIONS', u'')
ctx.rule(u'S_EXPRESSIONS', u'{S_EXPRESSION} {S_EXPRESSIONS}')

ctx.rule(u'BODY', u'{S_EXPRESSION}')

ctx.rule(u'BOUND_VAR', u'{SYMBOL}')
ctx.rule(u'BOUND_VAR', u'({SYMBOL} {SYMBOLS})')

ctx.rule(u'SYMBOLS', u'{SYMBOL} {SYMBOLS}')
ctx.rule(u'SYMBOLS', u'')
ctx.regex(u'SYMBOL', "[a-z]+")

ctx.regex(u'NUMBER', "[0-9]+")

ctx.regex(u'ATOM', "#t")
ctx.regex(u'ATOM', "#f")
