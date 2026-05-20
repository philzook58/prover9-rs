use crate::{
    set_order_method, set_symbol_precedence, Clause, CnfProblem, DemodDirection, EqualitySide,
    Literal, LiteralList, OrderMethod, RewriteRuleKind, RewriteSet, Substitution, Term,
    TermOrdering, WildDiscrimIndex,
};
use std::fs;
use std::path::PathBuf;

#[test]
fn constructs_terms_and_lists_children() {
    let x = Term::var(0).unwrap();
    let leaf = Term::atom("a").unwrap();
    let term = Term::app("f", [x, leaf]).unwrap();

    assert_eq!(term.to_string(), "f(v0,a)");
    let children = term.children();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].to_string(), "v0");
    assert_eq!(children[1].to_string(), "a");
}

#[test]
fn unifies_terms_and_returns_substitutions() {
    let left = Term::app("f", [Term::var(0).unwrap(), Term::atom("a").unwrap()]).unwrap();
    let right = Term::app("f", [Term::atom("b").unwrap(), Term::var(1).unwrap()]).unwrap();

    let unification = left.unify(&right).expect("should unify");

    assert_eq!(unification.left.to_string(), "f(b,a)");
    assert_eq!(unification.right.to_string(), "f(b,a)");
    assert_eq!(
        unification.left_substitution,
        vec![Substitution {
            var: 0,
            term: Term::atom("b").unwrap()
        }]
    );
    assert_eq!(
        unification.right_substitution,
        vec![Substitution {
            var: 1,
            term: Term::atom("a").unwrap()
        }]
    );
}

#[test]
fn detects_unification_failure() {
    let left = Term::app("f", [Term::atom("a").unwrap()]).unwrap();
    let right = Term::app("g", [Term::atom("a").unwrap()]).unwrap();
    assert!(left.unify(&right).is_none());
}

#[test]
fn matches_terms_one_way_and_returns_substitutions() {
    let pattern = Term::app("f", [Term::var(0).unwrap(), Term::atom("a").unwrap()]).unwrap();
    let target = Term::app("f", [Term::atom("b").unwrap(), Term::atom("a").unwrap()]).unwrap();

    let substitution = pattern.matches(&target).unwrap();

    assert_eq!(
        substitution,
        vec![Substitution {
            var: 0,
            term: Term::atom("b").unwrap(),
        }]
    );
    assert!(target.matches(&pattern).is_none());
}

#[test]
fn matching_respects_repeated_variables() {
    let pattern = Term::app("f", [Term::var(0).unwrap(), Term::var(0).unwrap()]).unwrap();
    let good = Term::app("f", [Term::atom("a").unwrap(), Term::atom("a").unwrap()]).unwrap();
    let bad = Term::app("f", [Term::atom("a").unwrap(), Term::atom("b").unwrap()]).unwrap();

    assert!(pattern.matches(&good).is_some());
    assert!(pattern.matches(&bad).is_none());
}

#[test]
fn compares_terms_with_lrpo() {
    set_order_method(OrderMethod::Lrpo);
    set_symbol_precedence("a", 0, 1).unwrap();
    set_symbol_precedence("f", 1, 2).unwrap();

    let subterm = Term::atom("a").unwrap();
    let superterm = Term::app("f", [Term::atom("a").unwrap()]).unwrap();

    assert_eq!(superterm.order(&subterm), TermOrdering::GreaterThan);
    assert_eq!(subterm.order(&superterm), TermOrdering::LessThan);
}

#[test]
fn compares_identical_terms_as_same() {
    set_order_method(OrderMethod::Lrpo);

    let left = Term::app("f", [Term::atom("a").unwrap(), Term::var(0).unwrap()]).unwrap();
    let right = left.clone();

    assert_eq!(left.order(&right), TermOrdering::SameAs);
}

#[test]
fn lpo_examples_from_traat() {
    fn lpo(left: &Term, right: &Term) -> bool {
        left.order(right) == TermOrdering::GreaterThan
    }

    fn v(n: usize) -> Term {
        Term::var(n).unwrap()
    }

    fn c(name: &str) -> Term {
        Term::atom(name).unwrap()
    }

    fn u1(name: &str, a: Term) -> Term {
        Term::app(name, [a]).unwrap()
    }

    fn b2(name: &str, a: Term, b: Term) -> Term {
        Term::app(name, [a, b]).unwrap()
    }

    set_order_method(OrderMethod::Lpo);
    set_symbol_precedence("x", 0, 1).unwrap();
    set_symbol_precedence("y", 0, 2).unwrap();
    set_symbol_precedence("e", 0, 3).unwrap();
    set_symbol_precedence("+", 2, 4).unwrap();
    set_symbol_precedence("f", 2, 5).unwrap();
    set_symbol_precedence("i", 1, 6).unwrap();

    let x = c("x");
    let y = c("y");
    let e = c("e");

    assert!(!lpo(&x, &y));
    assert!(lpo(&y, &x));
    assert!(!lpo(&v(0), &x));
    assert!(!lpo(&x, &v(0)));
    assert!(lpo(&b2("+", x.clone(), y.clone()), &x));
    assert!(lpo(&b2("f", v(0), e.clone()), &v(0)));
    assert!(lpo(&u1("i", e.clone()), &e));
    assert!(lpo(
        &u1("i", b2("f", v(0), v(1))),
        &b2("f", u1("i", v(1)), u1("i", v(0)))
    ));
    assert!(lpo(
        &b2("f", b2("f", v(0), v(1)), v(2)),
        &b2("f", v(0), b2("f", v(1), v(2)))
    ));

    assert!(!lpo(&x, &b2("+", x.clone(), y)));
    assert!(!lpo(&v(0), &b2("f", v(0), e.clone())));
    assert!(!lpo(&e, &u1("i", e.clone())));
    assert!(!lpo(
        &b2("f", u1("i", v(1)), u1("i", v(0))),
        &u1("i", b2("f", v(0), v(1)))
    ));
    assert!(!lpo(
        &b2("f", v(0), b2("f", v(1), v(2))),
        &b2("f", b2("f", v(0), v(1)), v(2))
    ));
}

#[test]
fn parses_terms_and_reports_basic_properties() {
    let term = Term::parse("f(g(a),a)").unwrap();
    let ga = Term::parse("g(a)").unwrap();
    let a = Term::parse("a").unwrap();

    assert_eq!(term.to_string(), "f(g(a),a)");
    assert!(term.is_ground());
    assert_eq!(term.depth(), 2);
    assert_eq!(term.symbol_count(), 4);
    assert!(ga.occurs_in(&term));
    assert!(a.occurs_in(&term));
}

#[test]
fn direct_ordering_predicates_work() {
    set_symbol_precedence("a", 0, 1).unwrap();
    set_symbol_precedence("f", 1, 2).unwrap();
    set_order_method(OrderMethod::Lpo);

    let a = Term::atom("a").unwrap();
    let fa = Term::app("f", [Term::atom("a").unwrap()]).unwrap();

    assert!(fa.lrpo_gt(&a));
    assert!(fa.greater_than_current_ordering(&a));
}

#[test]
fn wild_discrimination_retrieves_actual_generalizations() {
    let mut index = WildDiscrimIndex::new();
    let repeated = Term::app("f", [Term::var(0).unwrap(), Term::var(0).unwrap()]).unwrap();
    let generalized = Term::app("f", [Term::var(0).unwrap(), Term::atom("a").unwrap()]).unwrap();
    index.insert(&repeated);
    index.insert(&generalized);

    let query = Term::app("f", [Term::atom("b").unwrap(), Term::atom("a").unwrap()]).unwrap();
    let retrieved = index.retrieve_generalizations(&query);
    let rendered: Vec<_> = retrieved.iter().map(ToString::to_string).collect();

    assert_eq!(rendered, vec!["f(v0,a)"]);
}

#[test]
fn literals_wrap_sign_and_atom() {
    let pos = Literal::positive(Term::app("p", [Term::atom("a").unwrap()]).unwrap());
    let neg = Literal::negative(Term::app("q", [Term::atom("b").unwrap()]).unwrap());

    assert!(pos.is_positive());
    assert_eq!(pos.atom().to_string(), "p(a)");
    assert_eq!(pos.to_string(), "p(a)");

    assert!(!neg.is_positive());
    assert_eq!(neg.atom().to_string(), "q(b)");
    assert_eq!(neg.to_string(), "-(q(b))");
}

#[test]
fn literal_lists_are_safe_owned_containers() {
    let lits = LiteralList::new(vec![
        Literal::positive(Term::app("p", [Term::atom("a").unwrap()]).unwrap()),
        Literal::negative(Term::app("q", [Term::atom("b").unwrap()]).unwrap()),
    ]);

    assert_eq!(lits.len(), 2);
    assert!(!lits.is_empty());
    assert_eq!(lits.positive_count(), 1);
    assert_eq!(lits.negative_count(), 1);
    assert_eq!(lits.get(0).unwrap().to_string(), "p(a)");
    assert_eq!(lits.get(1).unwrap().to_string(), "-(q(b))");
    assert_eq!(lits.to_term().unwrap().to_string(), "|(p(a),-(q(b)))");
}

#[test]
fn clauses_wrap_clause_only_topforms() {
    let clause = Clause::from_term(Term::parse("|(p(X),-(q(a)))").unwrap());

    assert!(!clause.is_empty());
    assert_eq!(clause.to_string(), "|(p(X),-(q(a)))");

    let lits = clause.literals();
    assert_eq!(lits.len(), 2);
    assert_eq!(lits.get(0).unwrap().to_string(), "p(X)");
    assert_eq!(lits.get(1).unwrap().to_string(), "-(q(a))");
}

#[test]
fn clauses_can_be_renumbered() {
    let mut clause = Clause::from_term(
        Term::app(
            "|",
            [
                Term::app("p", [Term::var(7).unwrap()]).unwrap(),
                Literal::negative(Term::app("q", [Term::var(9).unwrap()]).unwrap()).to_term(),
            ],
        )
        .unwrap(),
    );

    clause.renumber_variables(100);
    assert_eq!(clause.to_string(), "|(p(v0),-(q(v1)))");
}

#[test]
fn clause_shape_predicates_work() {
    let positive_unit = Clause::from_term(Term::parse("p(a)").unwrap());
    let mixed_horn = Clause::from_term(Term::parse("|(p(a),-(q(b)))").unwrap());
    let negative = Clause::from_term(
        Term::app(
            "|",
            [
                Literal::negative(Term::app("p", [Term::var(0).unwrap()]).unwrap()).to_term(),
                Literal::negative(Term::app("q", [Term::atom("a").unwrap()]).unwrap()).to_term(),
            ],
        )
        .unwrap(),
    );

    assert_eq!(positive_unit.len(), 1);
    assert_eq!(positive_unit.positive_count(), 1);
    assert_eq!(positive_unit.negative_count(), 0);
    assert!(positive_unit.is_unit());
    assert!(positive_unit.is_horn());
    assert!(positive_unit.is_positive());
    assert!(!positive_unit.is_negative());
    assert!(!positive_unit.is_mixed());
    assert!(positive_unit.is_ground());
    assert_eq!(positive_unit.depth(), 1);
    assert_eq!(positive_unit.symbol_count(), 2);

    assert_eq!(mixed_horn.len(), 2);
    assert_eq!(mixed_horn.positive_count(), 1);
    assert_eq!(mixed_horn.negative_count(), 1);
    assert!(!mixed_horn.is_unit());
    assert!(mixed_horn.is_horn());
    assert!(!mixed_horn.is_positive());
    assert!(!mixed_horn.is_negative());
    assert!(mixed_horn.is_mixed());
    assert!(mixed_horn.is_ground());

    assert!(!negative.is_positive());
    assert!(negative.is_negative());
    assert!(negative.is_horn());
    assert!(!negative.is_ground());
}

#[test]
fn equality_predicates_work() {
    let pos_eq_lit = Literal::positive(
        Term::app("=", [Term::atom("a").unwrap(), Term::atom("b").unwrap()]).unwrap(),
    );
    let neg_eq_lit = Literal::negative(
        Term::app("=", [Term::atom("a").unwrap(), Term::atom("b").unwrap()]).unwrap(),
    );
    let non_eq_lit = Literal::positive(Term::app("p", [Term::atom("a").unwrap()]).unwrap());

    assert!(pos_eq_lit.is_positive_equality());
    assert!(!pos_eq_lit.is_negative_equality());
    assert!(!neg_eq_lit.is_positive_equality());
    assert!(neg_eq_lit.is_negative_equality());
    assert!(!non_eq_lit.is_positive_equality());
    assert!(!non_eq_lit.is_negative_equality());

    let pos_eq_unit = Clause::from_term(Term::parse("=(a,b)").unwrap());
    let neg_eq_unit =
        Clause::from_term(Literal::negative(Term::parse("=(a,b)").unwrap()).to_term());
    let mixed = Clause::from_term(Term::parse("|(=(a,b),p(a))").unwrap());
    let only_eq = Clause::from_term(Term::parse("|(=(a,b),-(=(b,c)))").unwrap());
    let taut = Clause::from_term(Term::parse("|(p(a),-(p(a)))").unwrap());

    assert!(pos_eq_unit.is_positive_equality_unit());
    assert!(!pos_eq_unit.is_negative_equality_unit());
    assert!(neg_eq_unit.is_negative_equality_unit());
    assert!(!neg_eq_unit.is_positive_equality_unit());

    assert!(mixed.contains_equality());
    assert!(mixed.contains_positive_equality());
    assert!(!mixed.only_equality_literals());

    assert!(only_eq.contains_equality());
    assert!(only_eq.only_equality_literals());

    assert!(taut.is_tautology());
}

#[test]
fn equality_sides_are_exposed_minimally() {
    let pos_eq = Literal::positive(
        Term::app(
            "=",
            [
                Term::atom("a").unwrap(),
                Term::app("f", [Term::atom("b").unwrap()]).unwrap(),
            ],
        )
        .unwrap(),
    );
    let neg_eq = Literal::negative(
        Term::app("=", [Term::var(0).unwrap(), Term::atom("c").unwrap()]).unwrap(),
    );
    let non_eq = Literal::positive(Term::app("p", [Term::atom("a").unwrap()]).unwrap());

    let (lhs, rhs) = pos_eq.equality_sides().unwrap();
    assert_eq!(lhs.to_string(), "a");
    assert_eq!(rhs.to_string(), "f(b)");

    let (lhs, rhs) = neg_eq.equality_sides().unwrap();
    assert_eq!(lhs.to_string(), "v0");
    assert_eq!(rhs.to_string(), "c");

    assert!(non_eq.equality_sides().is_none());
}

#[test]
fn clauses_demodulate_once_with_ladr_demod1() {
    set_order_method(OrderMethod::Lrpo);
    set_symbol_precedence("a", 0, 1).unwrap();
    set_symbol_precedence("f", 1, 2).unwrap();
    set_symbol_precedence("p", 1, 3).unwrap();

    let clause = Clause::from_term(Term::parse("p(f(a))").unwrap());
    let rule = Clause::from_term(Term::parse("=(f(a),a)").unwrap());

    let rewritten = clause
        .demodulate_once_with(&rule, DemodDirection::LeftToRight)
        .unwrap();

    assert_eq!(rewritten.to_string(), "p(a)");
    assert_eq!(clause.to_string(), "p(f(a))");
}

#[test]
fn demodulation_returns_none_when_rule_does_not_apply() {
    set_order_method(OrderMethod::Lrpo);
    set_symbol_precedence("a", 0, 1).unwrap();
    set_symbol_precedence("f", 1, 2).unwrap();
    set_symbol_precedence("p", 1, 3).unwrap();

    let clause = Clause::from_term(Term::parse("p(a)").unwrap());
    let rule = Clause::from_term(Term::parse("=(f(a),a)").unwrap());

    assert!(clause
        .demodulate_once_with(&rule, DemodDirection::LeftToRight)
        .is_none());
}

#[test]
fn clauses_subsume_as_ladr_defines_it() {
    let general = Clause::from_term(Term::app("p", [Term::var(0).unwrap()]).unwrap());
    let specific = Clause::from_term(Term::app("p", [Term::atom("b").unwrap()]).unwrap());
    let unrelated = Clause::from_term(Term::app("q", [Term::atom("b").unwrap()]).unwrap());

    assert!(general.subsumes(&specific));
    assert!(!specific.subsumes(&general));
    assert!(!general.subsumes(&unrelated));
}

#[test]
fn backtracking_subsumption_wrapper_is_exposed() {
    let general = Clause::from_term(
        Term::app(
            "|",
            [
                Term::app("p", [Term::var(0).unwrap()]).unwrap(),
                Term::app("q", [Term::var(1).unwrap()]).unwrap(),
            ],
        )
        .unwrap(),
    );
    let specific = Clause::from_term(
        Term::app(
            "|",
            [
                Term::app("p", [Term::atom("b").unwrap()]).unwrap(),
                Term::app("q", [Term::atom("a").unwrap()]).unwrap(),
            ],
        )
        .unwrap(),
    );

    assert!(general.subsumes_bt(&specific));
    assert!(!specific.subsumes_bt(&general));
}

#[test]
fn maximal_literal_queries_work() {
    set_order_method(OrderMethod::Lrpo);
    set_symbol_precedence("a", 0, 1).unwrap();
    set_symbol_precedence("f", 1, 2).unwrap();
    set_symbol_precedence("p", 1, 3).unwrap();
    set_symbol_precedence("q", 1, 4).unwrap();

    let clause = Clause::from_term(
        Term::app(
            "|",
            [
                Term::app("p", [Term::atom("a").unwrap()]).unwrap(),
                Term::app("p", [Term::app("f", [Term::atom("a").unwrap()]).unwrap()]).unwrap(),
            ],
        )
        .unwrap(),
    );

    assert_eq!(clause.maximal_literal_indices(), vec![1]);
    assert_eq!(clause.number_of_maximal_literals(), 1);
    assert_eq!(clause.maximal_signed_literal_indices(), vec![1]);
}

#[test]
fn clauses_resolve_directly() {
    let left = Clause::from_term(
        Term::app(
            "|",
            [
                Term::app("p", [Term::atom("a").unwrap()]).unwrap(),
                Term::app("r", [Term::atom("b").unwrap()]).unwrap(),
            ],
        )
        .unwrap(),
    );
    let right = Clause::from_term(
        Term::app(
            "|",
            [
                Literal::negative(Term::app("p", [Term::atom("a").unwrap()]).unwrap()).to_term(),
                Term::app("s", [Term::atom("c").unwrap()]).unwrap(),
            ],
        )
        .unwrap(),
    );

    let resolvent = left.resolve_with(0, &right, 0).unwrap();

    assert_eq!(resolvent.to_string(), "|(r(b),s(c))");
    assert!(left.resolve_with(1, &right, 0).is_none());
}

#[test]
fn clauses_resolve_with_flipped_other_equality() {
    let left = Clause::from_term(
        Term::app(
            "|",
            [
                Term::app("=", [Term::atom("a").unwrap(), Term::atom("b").unwrap()]).unwrap(),
                Term::app("r", [Term::atom("x").unwrap()]).unwrap(),
            ],
        )
        .unwrap(),
    );
    let right = Clause::from_term(
        Term::app(
            "|",
            [
                Literal::negative(
                    Term::app("=", [Term::atom("b").unwrap(), Term::atom("a").unwrap()]).unwrap(),
                )
                .to_term(),
                Term::app("s", [Term::atom("y").unwrap()]).unwrap(),
            ],
        )
        .unwrap(),
    );

    assert!(left.resolve_with(0, &right, 0).is_none());
    let resolvent = left.resolve_with_flipped_other(0, &right, 0).unwrap();
    assert_eq!(resolvent.to_string(), "|(r(x),s(y))");
}

#[test]
fn clauses_paramodulate_at_explicit_position() {
    let from = Clause::from_term(Term::parse("=(a,b)").unwrap());
    let into = Clause::from_term(
        Term::app(
            "|",
            [
                Term::app("p", [Term::atom("a").unwrap()]).unwrap(),
                Term::app("r", [Term::atom("c").unwrap()]).unwrap(),
            ],
        )
        .unwrap(),
    );

    let paramodulant = from
        .paramodulate_into(0, EqualitySide::Left, &into, 0, &[1], false)
        .unwrap();

    assert_eq!(paramodulant.to_string(), "|(p(b),r(c))");
    assert!(from
        .paramodulate_into(0, EqualitySide::Left, &into, 1, &[1], false)
        .is_none());
}

#[test]
fn clauses_paramodulate_into_variables_when_requested() {
    let from = Clause::from_term(Term::parse("=(a,b)").unwrap());
    let into = Clause::from_term(Term::app("p", [Term::var(0).unwrap()]).unwrap());

    assert!(from
        .paramodulate_into(0, EqualitySide::Left, &into, 0, &[1], false)
        .is_none());

    let paramodulant = from
        .paramodulate_into(0, EqualitySide::Left, &into, 0, &[1], true)
        .unwrap();

    assert_eq!(paramodulant.to_string(), "p(b)");
}

#[test]
fn clauses_resolve_with_xx_when_negative_equality_unifies() {
    let clause = Clause::from_term(
        Term::app(
            "|",
            [
                Literal::negative(
                    Term::app("=", [Term::atom("a").unwrap(), Term::atom("a").unwrap()]).unwrap(),
                )
                .to_term(),
                Term::app("p", [Term::atom("b").unwrap()]).unwrap(),
            ],
        )
        .unwrap(),
    );

    let resolvent = clause.resolve_with_xx(0).unwrap();
    assert_eq!(resolvent.to_string(), "p(b)");
    assert!(clause.resolve_with_xx(1).is_none());
}

#[test]
fn clauses_factor_explicitly() {
    let clause = Clause::from_term(
        LiteralList::new(vec![
            Literal::positive(Term::app("p", [Term::var(0).unwrap()]).unwrap()),
            Literal::positive(Term::app("p", [Term::atom("a").unwrap()]).unwrap()),
            Literal::positive(Term::app("q", [Term::atom("b").unwrap()]).unwrap()),
        ])
        .to_term()
        .unwrap(),
    );

    let factor = clause.factor_with(0, 1).unwrap();
    assert_eq!(factor.to_string(), "|(p(a),q(b))");
    assert!(clause.factor_with(1, 2).is_none());
}

#[test]
fn clauses_merge_duplicate_literals_without_mutating_original() {
    let clause = Clause::from_term(
        LiteralList::new(vec![
            Literal::positive(Term::app("p", [Term::atom("a").unwrap()]).unwrap()),
            Literal::positive(Term::app("p", [Term::atom("a").unwrap()]).unwrap()),
            Literal::positive(Term::app("q", [Term::atom("b").unwrap()]).unwrap()),
        ])
        .to_term()
        .unwrap(),
    );

    let merged = clause.merge_duplicate_literals();
    assert_eq!(merged.to_string(), "|(p(a),q(b))");
    assert_eq!(clause.to_string(), "|(p(a),|(p(a),q(b)))");
}

#[test]
fn clauses_simplify_basic_merges_duplicates_only() {
    let clause = Clause::from_term(
        LiteralList::new(vec![
            Literal::positive(Term::app("p", [Term::var(0).unwrap()]).unwrap()),
            Literal::positive(Term::app("p", [Term::atom("a").unwrap()]).unwrap()),
            Literal::positive(Term::app("p", [Term::atom("a").unwrap()]).unwrap()),
            Literal::positive(Term::app("q", [Term::atom("b").unwrap()]).unwrap()),
        ])
        .to_term()
        .unwrap(),
    );

    let simplified = clause.simplify_basic().unwrap();
    assert_eq!(simplified.to_string(), "|(p(v0),|(p(a),q(b)))");
}

#[test]
fn clauses_simplify_basic_drops_tautologies() {
    let clause = Clause::from_term(Term::parse("|(p(a),-(p(a)))").unwrap());
    assert!(clause.simplify_basic().is_none());
}

#[test]
fn parses_tptp_cnf_problems() {
    let problem = CnfProblem::parse_tptp_cnf(
        r#"
            % comment
            cnf(one,axiom,( p(a) )).
            cnf(two,negated_conjecture,( ~p(a) | q(X) )).
        "#,
    )
    .unwrap();

    assert_eq!(problem.clauses().len(), 2);
    assert_eq!(problem.clauses()[0].to_string(), "p(a)");
    assert_eq!(problem.clauses()[1].to_string(), "|(-(p(a)),q(v0))");
}

#[test]
fn proves_pyres_examples() {
    let examples = pyres_examples_dir();
    for name in ["PUZ001-1.p", "PUZ002-1.p", "PUZ003-1.p"] {
        let input = fs::read_to_string(examples.join(name)).unwrap();
        let problem = CnfProblem::parse_tptp_cnf(&input).unwrap();
        let proof = problem.prove_unsat();
        assert!(proof.is_some(), "{name} should be refutable");
        assert!(
            proof.unwrap().is_empty(),
            "{name} should derive the empty clause"
        );
    }
}

#[test]
fn rewrite_sets_normalize_terms_with_indexed_rules() {
    set_order_method(OrderMethod::Lrpo);
    set_symbol_precedence("a", 0, 1).unwrap();
    set_symbol_precedence("f", 1, 2).unwrap();
    set_symbol_precedence("g", 1, 3).unwrap();

    let mut rewrites = RewriteSet::new();
    assert_eq!(
        rewrites
            .insert_rule(&Clause::from_term(Term::parse("=(f(a),a)").unwrap()))
            .unwrap(),
        RewriteRuleKind::Oriented
    );
    assert_eq!(
        rewrites
            .insert_rule(&Clause::from_term(Term::parse("=(g(a),f(a))").unwrap()))
            .unwrap(),
        RewriteRuleKind::Oriented
    );

    let original = Term::parse("g(a)").unwrap();
    let rewritten = rewrites.rewrite_term(&original);

    assert_eq!(rewritten.to_string(), "a");
    assert_eq!(original.to_string(), "g(a)");
}

#[test]
fn rewrite_sets_normalize_whole_clauses() {
    set_order_method(OrderMethod::Lrpo);
    set_symbol_precedence("a", 0, 1).unwrap();
    set_symbol_precedence("f", 1, 2).unwrap();
    set_symbol_precedence("p", 1, 3).unwrap();
    set_symbol_precedence("q", 1, 4).unwrap();

    let mut rewrites = RewriteSet::new();
    rewrites
        .insert_rule(&Clause::from_term(Term::parse("=(f(a),a)").unwrap()))
        .unwrap();

    let clause = Clause::from_term(Term::parse("|(p(f(a)),q(f(a)))").unwrap());
    let rewritten = rewrites.rewrite_clause(&clause);

    assert_eq!(rewritten.to_string(), "|(p(a),q(a))");
    assert_eq!(clause.to_string(), "|(p(f(a)),q(f(a)))");
}

#[test]
fn rewrite_sets_reject_non_demodulators() {
    let mut rewrites = RewriteSet::new();
    let bad_rule = Clause::from_term(Term::parse("|(p(a),q(a))").unwrap());

    let err = rewrites.insert_rule(&bad_rule).unwrap_err();
    assert_eq!(err, crate::Error::InvalidRewriteRule("|(p(a),q(a))".into()));
}

fn pyres_examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../eprover-rust/PyRes/EXAMPLES")
}
