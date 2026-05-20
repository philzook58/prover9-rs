import prover9


def configure_ordering(symbol_specs: list[tuple[str, int, int]]) -> None:
    prover9.set_order_method(prover9.OrderMethod.Lrpo)
    for name, arity, precedence in symbol_specs:
        prover9.set_symbol_precedence(name, arity, precedence)


def rewrite_once(clause_text: str, rule_text: str) -> prover9.Clause | None:
    clause = prover9.Clause.from_term(prover9.Term(clause_text))
    rule = prover9.Clause.from_term(prover9.Term(rule_text))
    return clause.demodulate_once_with(rule, prover9.DemodDirection.LeftToRight)


def test_distribution_rule_rewrites_a_concrete_arithmetic_term() -> None:
    configure_ordering(
        [
            ("two", 0, 1),
            ("x", 0, 2),
            ("y", 0, 3),
            ("add", 2, 4),
            ("mul", 2, 5),
            ("goal", 1, 6),
        ]
    )

    rewritten = rewrite_once(
        "goal(mul(two,add(x,y)))",
        "=(mul(two,add(x,y)),add(mul(two,x),mul(two,y)))",
    )

    assert rewritten is not None
    assert str(rewritten) == "goal(add(mul(two,x),mul(two,y)))"


def test_differentiation_rules_expand_a_concrete_product_derivative() -> None:
    configure_ordering(
        [
            ("x", 0, 1),
            ("one", 0, 2),
            ("add", 2, 3),
            ("mul", 2, 4),
            ("diff", 2, 5),
            ("goal", 1, 6),
        ]
    )

    clause = prover9.Clause.from_term(prover9.Term("goal(diff(mul(x,x),x))"))
    rules = [
        "=(diff(mul(x,x),x),add(mul(diff(x,x),x),mul(x,diff(x,x))))",
        "=(diff(x,x),one)",
    ]

    for rule_text in rules:
        rule = prover9.Clause.from_term(prover9.Term(rule_text))
        while True:
            rewritten = clause.demodulate_once_with(
                rule, prover9.DemodDirection.LeftToRight
            )
            if rewritten is None:
                break
            clause = rewritten

    assert str(clause) == "goal(add(mul(one,x),mul(x,one)))"


def test_rewrite_rule_returns_none_when_it_does_not_match() -> None:
    configure_ordering(
        [
            ("two", 0, 1),
            ("x", 0, 2),
            ("y", 0, 3),
            ("add", 2, 4),
            ("mul", 2, 5),
            ("goal", 1, 6),
        ]
    )

    rewritten = rewrite_once(
        "goal(add(x,y))",
        "=(mul(two,add(x,y)),add(mul(two,x),mul(two,y)))",
    )

    assert rewritten is None
