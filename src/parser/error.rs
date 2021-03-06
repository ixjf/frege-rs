use super::parser::Rule;
use pest::error::Error as PestError;
use pest::error::ErrorVariant as PestErrorVariant;
use pest::Span;
use std::fmt;
use std::error::Error;

/// An error that may occur during parsing of the input, be it a syntax error
/// or a semantical error.
/// 
/// In addition to line and column, it provides a formatted message
/// underlining exactly where the error occurred and which error
/// it is.
#[derive(Debug)]
pub struct ParseError {
    /// (line, column)
    pub location: (usize, usize),
    decorated_message: String,
}

impl Error for ParseError {}

impl ParseError {
    pub(in crate::parser) fn new_from_custom_error(span: Span, decorated_message: &str) -> Self {
        let e: PestError<Rule> = PestError::new_from_span(
            PestErrorVariant::CustomError {
                message: decorated_message.to_owned(),
            },
            span.clone(),
        )
        .renamed_rules(ParseError::renamed_rules);

        ParseError {
            location: span.start_pos().line_col(),
            decorated_message: format!("{}", e),
        }
    }

    pub(in crate::parser) fn new_from_parsing_error(e: PestError<Rule>) -> ParseError {
        use pest::error::LineColLocation;

        let location = match e.line_col {
            LineColLocation::Pos((line, col)) => (line, col),
            _ => unreachable!(), // is this actually unreachable? it's not documented
        };

        ParseError {
            location,
            decorated_message: format!("{}", e.renamed_rules(ParseError::renamed_rules)),
        }
    }

    fn renamed_rules(r: &Rule) -> String {
        (match r {
            Rule::input => "input",
            Rule::statement_set => "statement set",
            Rule::argument => "argument",
            Rule::statement => "statement",
            Rule::grouper_opening => "grouper opening",
            Rule::grouper_closing => "grouper closing",
            Rule::statement_separator => "statement separator",
            Rule::statement_set_opening => "statement set opening",
            Rule::statement_set_closing => "statement set closing",
            Rule::conclusion_indicator => "conclusion indicator",
            Rule::premise => "premise",
            Rule::conclusion => "conclusion",
            Rule::formula => "formula",
            Rule::compound_formula => "compound formula",
            Rule::atomic_formula => "atomic formula",
            Rule::simple_predicate => "simple predicate",
            Rule::simple_statement => "simple statement",
            Rule::compound_formula_conjunction => "conjunction of formulas",
            Rule::compound_formula_negation => "negation of a formula",
            Rule::compound_formula_disjunction => "disjunction of formulas",
            Rule::compound_formula_conditional => "conditional formula",
            Rule::complex_statement => "complex statement",
            Rule::conjunction_connective => "conjunction connective",
            Rule::negation_connective => "negation connective",
            Rule::disjunction_connective => "disjunction connective",
            Rule::conditional_connective => "conditional connective",
            Rule::existential_statement => "existential statement",
            Rule::universal_statement => "universal statement",
            Rule::logical_conjunction => "logical conjunction",
            Rule::logical_negation => "logical negation",
            Rule::logical_disjunction => "logical disjunction",
            Rule::logical_conditional => "logical conditional",
            Rule::subscript_number => "subscript",
            Rule::simple_statement_letter_alpha => "simple statement letter",
            Rule::simple_statement_letter => "simple statement letter",
            Rule::singular_statement => "singular statement",
            Rule::singular_term_alpha => "singular term",
            Rule::singular_term => "singular term",
            Rule::variable_alpha => "variable",
            Rule::variable => "variable",
            Rule::superscript_number => "degree",
            Rule::predicate_letter_alpha => "predicate letter",
            Rule::predicate_letter => "predicate letter",
            Rule::existential_quantifier => "existential quantifier",
            Rule::universal_quantifier => "universal quantifier",
            Rule::EOI => "EOI",
            Rule::WHITESPACE => "white space",
        })
        .to_owned()
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.decorated_message)
    }
}
