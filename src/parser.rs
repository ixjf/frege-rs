#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Subscript(pub Option<u64>);

impl PartialEq<u64> for Subscript {
    fn eq(&self, rhs: &u64) -> bool {
        match self.0 {
            Some(ref lhs) => lhs == rhs,
            None => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SimpleStatementLetter(pub char, pub Subscript);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SingularTerm(pub char, pub Subscript);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Variable(pub char, pub Subscript);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Degree(pub u64);

impl PartialEq<u64> for Degree {
    fn eq(&self, rhs: &u64) -> bool {
        self.0 == *rhs
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PredicateLetter(pub char, pub Subscript, pub Degree);

#[derive(Debug, Clone)]
pub enum ParseTree {
    StatementSet(Vec<Statement>),
    Argument(Vec<Statement>, Statement),
}

#[derive(Debug, Clone)]
pub enum Term {
    SingularTerm(SingularTerm),
    Variable(Variable),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Simple(SimpleStatementLetter),
    Singular(PredicateLetter, Vec<SingularTerm>),
    LogicalConjunction(Box<Statement>, Box<Statement>),
    LogicalNegation(Box<Statement>),
    LogicalDisjunction(Box<Statement>, Box<Statement>),
    LogicalConditional(Box<Statement>, Box<Statement>),
    Existential(Variable, Predicate),
    Universal(Variable, Predicate),
}

#[derive(Debug, Clone)]
pub enum Predicate {
    Simple(PredicateLetter, Vec<Term>),
    Conjunctive(Box<Predicate>, Box<Predicate>),
    Negative(Box<Predicate>),
    Disjunctive(Box<Predicate>, Box<Predicate>),
    Conditional(Box<Predicate>, Box<Predicate>),
}

mod pest_parser {
    use pest::Parser;

    #[derive(Parser)]
    #[grammar = "GRAMMAR.pest"]
    pub struct GeneratedParser;
}

use self::pest_parser::Rule;
use pest::error::Error as pest_error;
use pest::error::ErrorVariant as pest_error_variant;
use pest::iterators::{Pair, Pairs};
use pest::Span;

pub struct Error {
    pub decorated_message: String,
    pub position: (usize, usize),
}

impl Error {
    pub(in parser) fn new_from_custom_error(span: Span, decorated_message: &str) -> Self {
        let e: pest_error<Rule> = pest_error::new_from_span(
            pest_error_variant::CustomError {
                message: decorated_message.to_owned(),
            },
            span.clone(),
        );

        Error {
            position: span.start_pos().line_col(),
            decorated_message: format!("{}", e),
        }
    }

    pub(in parser) fn new_from_parsing_error(e: pest_error<Rule>) -> Error {
        use pest::error::LineColLocation;

        let position = match e.line_col {
            LineColLocation::Pos((line, col)) => (line, col),
            _ => unreachable!(), // is this actually unreachable? it's not documented
        };

        Error {
            position,
            decorated_message: format!("{}", e),
        }
    }
}

pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn parse<'a>(&self, input: &'a str) -> Result<ParseTree, Error> {
        use pest::Parser;

        match pest_parser::GeneratedParser::parse(Rule::input, input) {
            Ok(p) => self.into_ast(p),
            Err(e) => Err(Error::new_from_parsing_error(e)),
        }
    }

    fn into_ast(&self, mut pairs: Pairs<'_, Rule>) -> Result<ParseTree, Error> {
        let inner = pairs.next().unwrap().into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::statement_set => self.statement_set_into_ast(inner),
            Rule::argument => self.argument_into_ast(inner),
            _ => unreachable!("should never reach here"),
        }
    }

    fn statement_set_into_ast(&self, pair: Pair<Rule>) -> Result<ParseTree, Error> {
        assert!(pair.as_rule() == Rule::statement_set);

        let mut statements = Vec::new();

        for st_pair in pair.into_inner() {
            match self.statement_into_ast(st_pair) {
                Ok(st) => statements.push(st),
                Err(e) => return Err(e),
            }
        }

        Ok(ParseTree::StatementSet(statements))
    }

    fn statement_into_ast(&self, pair: Pair<Rule>) -> Result<Statement, Error> {
        assert!(pair.as_rule() == Rule::statement);

        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::complex_statement => self.complex_statement_into_ast(inner),
            Rule::simple_statement => self.simple_statement_into_ast(inner),
            _ => unreachable!("should never reach here"),
        }
    }

    fn complex_statement_into_ast(&self, pair: Pair<Rule>) -> Result<Statement, Error> {
        assert!(pair.as_rule() == Rule::complex_statement);

        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::logical_conjunction => self.logical_conjunction_into_ast(inner),
            Rule::logical_negation => self.logical_negation_into_ast(inner),
            Rule::logical_disjunction => self.logical_disjunction_into_ast(inner),
            Rule::logical_conditional => self.logical_conditional_into_ast(inner),
            Rule::existential_statement => self.existential_statement_into_ast(inner),
            Rule::universal_statement => self.universal_statement_into_ast(inner),
            _ => unreachable!("should never reach here"),
        }
    }

    fn logical_conjunction_into_ast(&self, pair: Pair<Rule>) -> Result<Statement, Error> {
        assert!(pair.as_rule() == Rule::logical_conjunction);

        let mut inner = pair.into_inner();

        let lstatement = self.statement_into_ast(inner.next().unwrap())?;
        let rstatement = self.statement_into_ast(inner.next().unwrap())?;

        Ok(Statement::LogicalConjunction(
            Box::new(lstatement),
            Box::new(rstatement),
        ))
    }

    fn logical_negation_into_ast(&self, pair: Pair<Rule>) -> Result<Statement, Error> {
        assert!(pair.as_rule() == Rule::logical_negation);

        let mut inner = pair.into_inner();

        let rstatement = self.statement_into_ast(inner.next().unwrap())?;

        Ok(Statement::LogicalNegation(Box::new(rstatement)))
    }

    fn logical_disjunction_into_ast(&self, pair: Pair<Rule>) -> Result<Statement, Error> {
        assert!(pair.as_rule() == Rule::logical_disjunction);

        let mut inner = pair.into_inner();

        let lstatement = self.statement_into_ast(inner.next().unwrap())?;
        let rstatement = self.statement_into_ast(inner.next().unwrap())?;

        Ok(Statement::LogicalDisjunction(
            Box::new(lstatement),
            Box::new(rstatement),
        ))
    }

    fn logical_conditional_into_ast(&self, pair: Pair<Rule>) -> Result<Statement, Error> {
        assert!(pair.as_rule() == Rule::logical_conditional);

        let mut inner = pair.into_inner();

        let lstatement = self.statement_into_ast(inner.next().unwrap())?;
        let rstatement = self.statement_into_ast(inner.next().unwrap())?;

        Ok(Statement::LogicalConditional(
            Box::new(lstatement),
            Box::new(rstatement),
        ))
    }

    fn existential_statement_into_ast(&self, pair: Pair<Rule>) -> Result<Statement, Error> {
        assert!(pair.as_rule() == Rule::existential_statement);

        let mut inner = pair.into_inner();

        let variable = self.variable_into_ast(inner.next().unwrap());

        let mut stack = Vec::new();
        stack.push(variable.clone());
        let predicate = self.predicate_into_ast(inner.next().unwrap(), &mut stack)?;

        Ok(Statement::Existential(variable, predicate))
    }

    fn universal_statement_into_ast(&self, pair: Pair<Rule>) -> Result<Statement, Error> {
        assert!(pair.as_rule() == Rule::universal_statement);

        let mut inner = pair.into_inner();

        let variable = self.variable_into_ast(inner.next().unwrap());

        let mut stack = Vec::new();
        stack.push(variable.clone());
        let predicate = self.predicate_into_ast(inner.next().unwrap(), &mut stack)?;

        Ok(Statement::Universal(variable, predicate))
    }

    fn simple_statement_into_ast(&self, pair: Pair<Rule>) -> Result<Statement, Error> {
        assert!(pair.as_rule() == Rule::simple_statement);

        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::singular_statement => self.singular_statement_into_ast(inner),
            Rule::simple_statement_letter => {
                let mut inner_inner = inner.into_inner();
                let letter = inner_inner.next().unwrap().as_str().chars().next().unwrap();

                let subscript = match inner_inner.peek() {
                    Some(_) => self.subscript_into_ast(inner_inner.next().unwrap()),
                    None => Subscript(None),
                };

                Ok(Statement::Simple(SimpleStatementLetter(letter, subscript)))
            }
            _ => unreachable!("should never reach here"),
        }
    }

    fn singular_statement_into_ast(&self, pair: Pair<Rule>) -> Result<Statement, Error> {
        assert!(pair.as_rule() == Rule::singular_statement);

        let mut inner = pair.clone().into_inner();

        let predicate_letter = self.predicate_letter_into_ast(inner.next().unwrap());

        let terms = inner
            .map(|x| match x.as_rule() {
                Rule::singular_term => self.singular_term_into_ast(x),
                _ => unreachable!("should never reach here"),
            })
            .collect::<Vec<SingularTerm>>();

        if predicate_letter.2 != terms.len() as u64 {
            return Err(Error::new_from_custom_error(
                pair.as_span(),
                "degree doesn't match number of terms specified",
            ));
        }

        Ok(Statement::Singular(predicate_letter, terms))
    }

    fn singular_term_into_ast(&self, pair: Pair<Rule>) -> SingularTerm {
        assert!(pair.as_rule() == Rule::singular_term);

        let mut inner = pair.into_inner();

        let alpha = inner.next().unwrap().as_str().chars().next().unwrap();

        let subscript = match inner.peek() {
            Some(_) => self.subscript_into_ast(inner.next().unwrap()),
            None => Subscript(None),
        };

        SingularTerm(alpha, subscript)
    }

    fn predicate_letter_into_ast(&self, pair: Pair<Rule>) -> PredicateLetter {
        assert!(pair.as_rule() == Rule::predicate_letter);

        let mut inner = pair.into_inner();

        let predicate_letter_alpha = inner.next().unwrap().as_str().chars().next().unwrap();

        let predicate_letter_subscript = match inner.peek().unwrap().as_rule() {
            Rule::subscript_number => self.subscript_into_ast(inner.next().unwrap()),
            _ => Subscript(None),
        };

        let superscript_number = Degree(
            inner
                .next()
                .unwrap()
                .as_str()
                .chars()
                .map(|x| match x {
                    '\u{2070}' => '0',
                    '\u{00B9}' => '1',
                    '\u{00B2}' => '2',
                    '\u{00B3}' => '3',
                    '\u{2074}' => '4',
                    '\u{2075}' => '5',
                    '\u{2076}' => '6',
                    '\u{2077}' => '7',
                    '\u{2078}' => '8',
                    '\u{2079}' => '9',
                    _ => unreachable!("should never reach here"),
                })
                .collect::<String>()
                .parse()
                .unwrap(),
        );

        PredicateLetter(
            predicate_letter_alpha,
            predicate_letter_subscript,
            superscript_number,
        )
    }

    fn subscript_into_ast(&self, pair: Pair<Rule>) -> Subscript {
        assert!(pair.as_rule() == Rule::subscript_number);

        Subscript(Some(
            pair.as_str()
                .chars()
                .map(|x| match x {
                    '\u{2080}' => '0',
                    '\u{2081}' => '1',
                    '\u{2082}' => '2',
                    '\u{2083}' => '3',
                    '\u{2084}' => '4',
                    '\u{2085}' => '5',
                    '\u{2086}' => '6',
                    '\u{2087}' => '7',
                    '\u{2088}' => '8',
                    '\u{2089}' => '9',
                    _ => unreachable!("should never reach here"),
                })
                .collect::<String>()
                .parse::<u64>()
                .unwrap(),
        ))
    }

    fn predicate_into_ast(
        &self,
        pair: Pair<Rule>,
        mut stack: &mut Vec<Variable>,
    ) -> Result<Predicate, Error> {
        assert!(pair.as_rule() == Rule::predicate);

        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::compound_predicate => self.compound_predicate_into_ast(inner, &mut stack),
            Rule::simple_predicate => self.simple_predicate_into_ast(inner, &mut stack),
            _ => unreachable!("should never reach here"),
        }
    }

    fn compound_predicate_into_ast(
        &self,
        pair: Pair<Rule>,
        mut stack: &mut Vec<Variable>,
    ) -> Result<Predicate, Error> {
        assert!(pair.as_rule() == Rule::compound_predicate);

        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::conjunctive_predicate => self.conjunctive_predicate_into_ast(inner, &mut stack),
            Rule::negative_predicate => self.negative_predicate_into_ast(inner, &mut stack),
            Rule::disjunctive_predicate => self.disjunctive_predicate_into_ast(inner, &mut stack),
            Rule::conditional_predicate => self.conditional_predicate_into_ast(inner, &mut stack),
            _ => unreachable!("should never reach here"),
        }
    }

    fn conjunctive_predicate_into_ast(
        &self,
        pair: Pair<Rule>,
        stack: &mut Vec<Variable>,
    ) -> Result<Predicate, Error> {
        assert!(pair.as_rule() == Rule::conjunctive_predicate);

        let mut inner = pair.into_inner();

        let lpredicate = self.predicate_into_ast(inner.next().unwrap(), &mut stack.clone())?;
        let rpredicate = self.predicate_into_ast(inner.next().unwrap(), &mut stack.clone())?;

        Ok(Predicate::Conjunctive(
            Box::new(lpredicate),
            Box::new(rpredicate),
        ))
    }

    fn negative_predicate_into_ast(
        &self,
        pair: Pair<Rule>,
        mut stack: &mut Vec<Variable>,
    ) -> Result<Predicate, Error> {
        assert!(pair.as_rule() == Rule::negative_predicate);

        let mut inner = pair.into_inner();

        let rpredicate = self.predicate_into_ast(inner.next().unwrap(), &mut stack)?;

        Ok(Predicate::Negative(Box::new(rpredicate)))
    }

    fn disjunctive_predicate_into_ast(
        &self,
        pair: Pair<Rule>,
        stack: &mut Vec<Variable>,
    ) -> Result<Predicate, Error> {
        assert!(pair.as_rule() == Rule::disjunctive_predicate);

        let mut inner = pair.into_inner();

        let lpredicate = self.predicate_into_ast(inner.next().unwrap(), &mut stack.clone())?;
        let rpredicate = self.predicate_into_ast(inner.next().unwrap(), &mut stack.clone())?;

        Ok(Predicate::Disjunctive(
            Box::new(lpredicate),
            Box::new(rpredicate),
        ))
    }

    fn conditional_predicate_into_ast(
        &self,
        pair: Pair<Rule>,
        stack: &mut Vec<Variable>,
    ) -> Result<Predicate, Error> {
        assert!(pair.as_rule() == Rule::conditional_predicate);

        let mut inner = pair.into_inner();

        let lpredicate = self.predicate_into_ast(inner.next().unwrap(), &mut stack.clone())?;
        let rpredicate = self.predicate_into_ast(inner.next().unwrap(), &mut stack.clone())?;

        Ok(Predicate::Conditional(
            Box::new(lpredicate),
            Box::new(rpredicate),
        ))
    }

    fn simple_predicate_into_ast(
        &self,
        pair: Pair<Rule>,
        stack: &mut Vec<Variable>,
    ) -> Result<Predicate, Error> {
        assert!(pair.as_rule() == Rule::simple_predicate);

        let mut inner = pair.clone().into_inner();

        let predicate_letter = self.predicate_letter_into_ast(inner.next().unwrap());

        let terms = inner
            .map(|x| match x.as_rule() {
                Rule::singular_term => Term::SingularTerm(self.singular_term_into_ast(x)),
                Rule::variable => Term::Variable(self.variable_into_ast(x)),
                _ => unreachable!("should never reach here"),
            })
            .collect::<Vec<Term>>();

        if predicate_letter.2 != terms.len() as u64 {
            return Err(Error::new_from_custom_error(
                pair.as_span(),
                "degree doesn't match number of terms specified",
            ));
        }

        if !terms.iter().all(|x| match x {
            Term::Variable(var) => stack.contains(var),
            _ => true,
        }) {
            return Err(Error::new_from_custom_error(
                pair.as_span(),
                "predicate binds to variable that isn't in scope",
            ));
        }

        Ok(Predicate::Simple(predicate_letter, terms))
    }

    fn variable_into_ast(&self, pair: Pair<Rule>) -> Variable {
        assert!(pair.as_rule() == Rule::variable);

        let mut inner = pair.into_inner();

        let alpha = inner.next().unwrap().as_str().chars().next().unwrap();

        let subscript = match inner.peek() {
            Some(_) => self.subscript_into_ast(inner.next().unwrap()),
            None => Subscript(None),
        };

        Variable(alpha, subscript)
    }

    fn argument_into_ast(&self, pair: Pair<Rule>) -> Result<ParseTree, Error> {
        assert!(pair.as_rule() == Rule::argument);

        let mut statements = Vec::new();

        for st_pair in pair.into_inner() {
            match st_pair.as_rule() {
                Rule::premise | Rule::conclusion => {
                    match self.statement_into_ast(st_pair.into_inner().next().unwrap()) {
                        Ok(st) => statements.push(st),
                        Err(e) => return Err(e),
                    }
                }
                _ => unreachable!("should never reach here"),
            }
        }

        // The grammar guarantees us that the conclusion comes last.
        // That means that the conclusion will be at the back of the vector
        let conclusion = statements.pop().unwrap();

        Ok(ParseTree::Argument(statements, conclusion))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_error_provides_correct_position_info() {
        let e =
            Error::new_from_custom_error(Span::new("Hello world!", 0, 4).unwrap(), "missing comma");
        assert!(e.position.0 == 1 && e.position.1 == 1);
    }

    #[test]
    fn parses_statement_set() {
        let parser = Parser::new();

        match parser.parse("{A, B, C, D, F, G}") {
            Ok(parse_tree) => match parse_tree {
                ParseTree::StatementSet(_) => {}
                _ => assert!(false),
            },
            _ => assert!(false),
        };
    }

    #[test]
    fn parses_argument() {
        let parser = Parser::new();

        match parser.parse("A, B, C, D, F .:. G") {
            Ok(parse_tree) => match parse_tree {
                ParseTree::Argument(_, _) => {}
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn parses_logical_conjunction() {
        let parser = Parser::new();

        match parser.parse("{(A & B)}") {
            Ok(parse_tree) => match parse_tree {
                ParseTree::StatementSet(mut statements) => {
                    assert!(statements.len() == 1);
                    match statements.pop().unwrap() {
                        Statement::LogicalConjunction(a, b) => match (*a, *b) {
                            (Statement::Simple(_), Statement::Simple(_)) => {}
                            _ => assert!(false),
                        },
                        _ => assert!(false),
                    }
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn parses_logical_negation() {
        let parser = Parser::new();

        match parser.parse("{~A}") {
            Ok(parse_tree) => match parse_tree {
                ParseTree::StatementSet(mut statements) => {
                    assert!(statements.len() == 1);
                    match statements.pop().unwrap() {
                        Statement::LogicalNegation(a) => match *a {
                            Statement::Simple(_) => {}
                            _ => assert!(false),
                        },
                        _ => assert!(false),
                    }
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn parses_logical_disjunction() {
        let parser = Parser::new();

        match parser.parse("{(A ∨ B)}") {
            Ok(parse_tree) => match parse_tree {
                ParseTree::StatementSet(mut statements) => {
                    assert!(statements.len() == 1);
                    match statements.pop().unwrap() {
                        Statement::LogicalDisjunction(a, b) => match (*a, *b) {
                            (Statement::Simple(_), Statement::Simple(_)) => {}
                            _ => assert!(false),
                        },
                        _ => assert!(false),
                    }
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn parses_logical_conditional() {
        let parser = Parser::new();

        match parser.parse("{(A ⊃ B)}") {
            Ok(parse_tree) => match parse_tree {
                ParseTree::StatementSet(mut statements) => {
                    assert!(statements.len() == 1);
                    match statements.pop().unwrap() {
                        Statement::LogicalConditional(a, b) => match (*a, *b) {
                            (Statement::Simple(_), Statement::Simple(_)) => {}
                            _ => assert!(false),
                        },
                        _ => assert!(false),
                    }
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn parses_existential_statement() {
        let parser = Parser::new();

        match parser.parse("{∃z(A¹z & B¹z)}") {
            Ok(parse_tree) => match parse_tree {
                ParseTree::StatementSet(mut statements) => {
                    assert!(statements.len() == 1);
                    match statements.pop().unwrap() {
                        Statement::Existential(a, b) => match (a, b) {
                            (Variable(_, _), Predicate::Conjunctive(_, _)) => {}
                            _ => assert!(false),
                        },
                        _ => assert!(false),
                    }
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn understands_that_degree_means_number_of_terms() {
        let parser = Parser::new();

        match parser.parse("{∃zA¹zs}") {
            Ok(_) => assert!(false),
            _ => {}
        }
    }

    #[test]
    fn keeps_track_of_variable_stack() {
        let parser = Parser::new();

        match parser.parse("{∃zA¹y}") {
            Ok(_) => assert!(false),
            _ => {}
        }
    }

    #[test]
    fn parses_universal_statement() {
        let parser = Parser::new();

        match parser.parse("{∀z(A¹z & B¹z)}") {
            Ok(parse_tree) => match parse_tree {
                ParseTree::StatementSet(mut statements) => {
                    assert!(statements.len() == 1);
                    match statements.pop().unwrap() {
                        Statement::Universal(a, b) => match (a, b) {
                            (Variable(_, _), Predicate::Conjunctive(_, _)) => {}
                            _ => assert!(false),
                        },
                        _ => assert!(false),
                    }
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn parses_simple_statement_with_subscript() {
        let parser = Parser::new();

        match parser.parse("{A₂}") {
            Ok(parse_tree) => match parse_tree {
                ParseTree::StatementSet(mut statements) => {
                    assert!(statements.len() == 1);
                    match statements.pop().unwrap() {
                        Statement::Simple(st_letter) => {
                            assert!(st_letter.0 == 'A');
                            assert!(st_letter.1 == Subscript(Some(2)));
                        }
                        _ => assert!(false),
                    }
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn parses_singular_statement() {
        let parser = Parser::new();

        match parser.parse("{A₂¹b}") {
            Ok(parse_tree) => match parse_tree {
                ParseTree::StatementSet(mut statements) => {
                    assert!(statements.len() == 1);
                    match statements.pop().unwrap() {
                        Statement::Singular(predicate_letter, mut terms) => {
                            assert!(predicate_letter.0 == 'A');
                            assert!(predicate_letter.1 == 2);
                            assert!(predicate_letter.2 == 1);
                            assert!(terms.len() == 1);
                            assert!(terms.pop().unwrap() == SingularTerm('b', Subscript(None)));
                        }
                        _ => assert!(false),
                    }
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn singular_statement_doesnt_allow_variables() {
        let parser = Parser::new();

        match parser.parse("{A₂¹x}") {
            Ok(_) => assert!(false),
            _ => {}
        }
    }
}