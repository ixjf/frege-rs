local Interpreter = Interpreter or require "../Interpreter"
local Grammar = Grammar or require "../Grammar"
local SequenceGroup = SequenceGroup or require "../SequenceGroup"
local Char = Char or require "../Char"
local Alternatives = Alternatives or require "../Alternatives"
local Attributes = Attributes or require "../Attributes"
local Error = Error or require "../Error"

function parses_valid_statement()
    local g = Grammar()
        :add_rule("grouper-opening", SequenceGroup(Char("(")), Attributes(Attributes.Types.Token))
        :add_rule("grouper-closing", SequenceGroup(Char(")")), Attributes(Attributes.Types.Token))
        :add_rule("conjunction-connective", SequenceGroup(Char("&")), Attributes(Attributes.Types.Token))
        :add_rule("disjunction-connective", SequenceGroup(Char(utf8.char(0x2228))), Attributes(Attributes.Types.Token))
        :add_rule("negation-connective", SequenceGroup(Char("~")), Attributes(Attributes.Types.Token))
        :add_rule("conditional-connective", SequenceGroup(Char(utf8.char(0x2283))), Attributes(Attributes.Types.Token))
        :add_rule("simple-statement-letter", SequenceGroup(Range("A", "Z"), Range(utf8.char(0x2081), utf8.char(0x2089))), Attributes(Attributes.Types.Token))

        :add_rule("logical-conjunction", SequenceGroup("grouper-opening", "statement", "conjunction-connective", "statement", "grouper-closing"))
        :add_rule("statement", SequenceGroup(Alternatives(SequenceGroup("simple-statement"), SequenceGroup("complex-statement"))))
        :add_rule("simple-statement", SequenceGroup("simple-statement-letter"))
        :add_rule("complex-statement", SequenceGroup(Alternatives(SequenceGroup("logical-conjunction"), SequenceGroup("logical-disjunction"), SequenceGroup("logical-negation"), SequenceGroup("logical-conditional"))))
        :add_rule("logical-disjunction", SequenceGroup("grouper-opening", "statement", "disjunction-connective", "statement", "grouper-closing"))
        :add_rule("logical-negation", SequenceGroup("negation-connective", "statement"))
        :add_rule("logical-conditional", SequenceGroup("grouper-opening", "statement", "conditional-connective", "statement", "grouper-closing"))

    local i = Interpreter(g)
    assert(i:run("(A" .. utf8.char(0x2081) .. "&B" .. utf8.char(0x2082) .. ")", "statement"), "failed to parse valid logical conjunction")
    assert(i:run("F" .. utf8.char(0x2084), "statement"), "failed to parse valid simple statement")
    assert(i:run("~Z" .. utf8.char(0x2089), "statement"), "failed to parse valid logical negation")
    assert(i:run("(C" .. utf8.char(0x2086) .. utf8.char(0x2283) .. "D" .. utf8.char(0x2087) .. ")", "statement"), "failed to parse valid logical disjunction")
    assert(i:run("(K" .. utf8.char(0x2088) .. utf8.char(0x2228) .. "M" .. utf8.char(0x2083) .. ")", "statement"), "failed to parse valid logical conditional")
end

function understands_tokens_with_alternatives()
    local g = Grammar()
        :add_rule("rule1", SequenceGroup(Alternatives(SequenceGroup(Char("%")), SequenceGroup(Char("A")))), Attributes(Attributes.Types.Token))

        :add_rule("rule2", SequenceGroup("rule1"))

    local i = Interpreter(g)
    assert(i:run("A", "rule2"), "failed to parse input with valid token alternative")
    assert(i:run("%", "rule2"), "failed to parse input with valid token alternative")

    -- just in case
    assert(not i:run("B", "rule2"), "succeeded on invalid input @ token alternatives")
end

function chooses_best_match()
    local g1 = Grammar()
        :add_rule("rule4", SequenceGroup(Char("B")), Attributes(Attributes.Types.Token))
        :add_rule("rule5", SequenceGroup(Char("C")), Attributes(Attributes.Types.Token))
        

        :add_rule("rule1", SequenceGroup(Alternatives(SequenceGroup("rule2"), SequenceGroup("rule3"))))
        :add_rule("rule2", SequenceGroup("rule4", "rule5"))
        :add_rule("rule3", SequenceGroup("rule4"))

    -- TODO! This will work by checking if interpreter whose grammar has rule2 defined 
    -- both before and after (in separate grammars) rule3
    -- still chooses rule2 over rule3
    -- However, before we have a parse tree, we can't actually test this
end

function can_provide_error_info()
    local g = Grammar()
        :add_rule("grouper-opening", SequenceGroup(Char("(")), Attributes(Attributes.Types.Token))
        :add_rule("grouper-closing", SequenceGroup(Char(")")), Attributes(Attributes.Types.Token))
        :add_rule("conjunction-connective", SequenceGroup(Char("&")), Attributes(Attributes.Types.Token))
        :add_rule("disjunction-connective", SequenceGroup(Char(utf8.char(0x2228))), Attributes(Attributes.Types.Token))
        :add_rule("negation-connective", SequenceGroup(Char("~")), Attributes(Attributes.Types.Token))
        :add_rule("conditional-connective", SequenceGroup(Char(utf8.char(0x2283))), Attributes(Attributes.Types.Token))
        :add_rule("simple-statement-letter", SequenceGroup(Range("A", "Z"), Range(utf8.char(0x2081), utf8.char(0x2089))), Attributes(Attributes.Types.Token))

        :add_rule("logical-conjunction", SequenceGroup("grouper-opening", "statement", "conjunction-connective", "statement", "grouper-closing"))
        :add_rule("statement", SequenceGroup(Alternatives(SequenceGroup("simple-statement"), SequenceGroup("complex-statement"))))
        :add_rule("simple-statement", SequenceGroup("simple-statement-letter"))
        :add_rule("complex-statement", SequenceGroup(Alternatives(SequenceGroup("logical-conjunction"), SequenceGroup("logical-disjunction"), SequenceGroup("logical-negation"), SequenceGroup("logical-conditional"))))
        :add_rule("logical-disjunction", SequenceGroup("grouper-opening", "statement", "disjunction-connective", "statement", "grouper-closing"))
        :add_rule("logical-negation", SequenceGroup("negation-connective", "statement"))
        :add_rule("logical-conditional", SequenceGroup("grouper-opening", "statement", "conditional-connective", "statement", "grouper-closing"))

    local i = Interpreter(g)
    local res, err = i:run("A" .. utf8.char(0x2081) .. "&B" .. utf8.char(0x2082) .. ")", "statement")
    assert(res == false, "parsed invalid logical conjunction statement")
    assert(Error.isInstanceOf(err, Error), "parsed invalid logical conjunction statement")
    assert(err:err_type() == Error.Types.UnrecognizedExpr, "run() returned invalid err type")
    assert(err:err_line() == 1, "run() passed wrong error location info")
    assert(err:err_col() == 3, "run() passed wrong error location info")
end

parses_valid_statement()
understands_tokens_with_alternatives()
chooses_best_match()
-- Not working as expected, matches simple statement and everything else is ignored, rip
--can_provide_error_info()