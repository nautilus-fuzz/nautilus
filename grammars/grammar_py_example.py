
#ctx.rule(NONTERM: string, RHS: string|bytes) adds a rule NONTERM->RHS. We can use {NONTERM} in the RHS to request a recursion. 
ctx.rule("START","<document>{XML_CONTENT}</document>")
ctx.rule("XML_CONTENT","{XML}{XML_CONTENT}")
ctx.rule("XML_CONTENT","")

#ctx.script(NONTERM:string, RHS: [string]], func) adds a rule NONTERM->func(*RHS). 
# In contrast to normal `rule`, RHS is an array of nonterminals. 
# It's up to the function to combine the values returned for the NONTERMINALS with any fixed content used.
ctx.script("XML",["TAG","ATTR","XML_CONTENT"], lambda tag,attr,body: b"<%s %s>%s</%s>"%(tag,attr,body,tag) )
ctx.rule("ATTR","foo=bar")
ctx.rule("TAG","some_tag")
ctx.rule("TAG","other_tag")
ctx.regex("TAG","[a-z]+")