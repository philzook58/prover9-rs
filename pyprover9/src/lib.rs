use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyModule};

fn to_py_err(err: prover9::Error) -> PyErr {
    PyValueError::new_err(format!("{err:?}"))
}

macro_rules! py_enum_display {
    ($name:ident) => {
        #[pymethods]
        impl $name {
            fn __repr__(&self) -> String {
                format!("{self:?}")
            }

            fn __str__(&self) -> String {
                format!("{self:?}")
            }
        }
    };
}

#[pyclass(name = "OrderMethod", module = "prover9", eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PyOrderMethod {
    Lrpo,
    Lpo,
    Rpo,
    Kbo,
}

impl From<PyOrderMethod> for prover9::OrderMethod {
    fn from(value: PyOrderMethod) -> Self {
        match value {
            PyOrderMethod::Lrpo => Self::Lrpo,
            PyOrderMethod::Lpo => Self::Lpo,
            PyOrderMethod::Rpo => Self::Rpo,
            PyOrderMethod::Kbo => Self::Kbo,
        }
    }
}

#[pyclass(name = "TermOrdering", module = "prover9", eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PyTermOrdering {
    NotComparable,
    SameAs,
    LessThan,
    GreaterThan,
    LessThanOrSameAs,
    GreaterThanOrSameAs,
    NotLessThan,
    NotGreaterThan,
}

impl From<prover9::TermOrdering> for PyTermOrdering {
    fn from(value: prover9::TermOrdering) -> Self {
        match value {
            prover9::TermOrdering::NotComparable => Self::NotComparable,
            prover9::TermOrdering::SameAs => Self::SameAs,
            prover9::TermOrdering::LessThan => Self::LessThan,
            prover9::TermOrdering::GreaterThan => Self::GreaterThan,
            prover9::TermOrdering::LessThanOrSameAs => Self::LessThanOrSameAs,
            prover9::TermOrdering::GreaterThanOrSameAs => Self::GreaterThanOrSameAs,
            prover9::TermOrdering::NotLessThan => Self::NotLessThan,
            prover9::TermOrdering::NotGreaterThan => Self::NotGreaterThan,
        }
    }
}

#[pyclass(name = "DemodDirection", module = "prover9", eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PyDemodDirection {
    LeftToRight,
    RightToLeft,
}

impl From<PyDemodDirection> for prover9::DemodDirection {
    fn from(value: PyDemodDirection) -> Self {
        match value {
            PyDemodDirection::LeftToRight => Self::LeftToRight,
            PyDemodDirection::RightToLeft => Self::RightToLeft,
        }
    }
}

#[pyclass(name = "EqualitySide", module = "prover9", eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PyEqualitySide {
    Left,
    Right,
}

impl From<PyEqualitySide> for prover9::EqualitySide {
    fn from(value: PyEqualitySide) -> Self {
        match value {
            PyEqualitySide::Left => Self::Left,
            PyEqualitySide::Right => Self::Right,
        }
    }
}

#[pyclass(name = "RewriteRuleKind", module = "prover9", eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PyRewriteRuleKind {
    Oriented,
    LexDependentLeftToRight,
    LexDependentRightToLeft,
    LexDependentBoth,
}

impl From<prover9::RewriteRuleKind> for PyRewriteRuleKind {
    fn from(value: prover9::RewriteRuleKind) -> Self {
        match value {
            prover9::RewriteRuleKind::Oriented => Self::Oriented,
            prover9::RewriteRuleKind::LexDependentLeftToRight => Self::LexDependentLeftToRight,
            prover9::RewriteRuleKind::LexDependentRightToLeft => Self::LexDependentRightToLeft,
            prover9::RewriteRuleKind::LexDependentBoth => Self::LexDependentBoth,
        }
    }
}

py_enum_display!(PyOrderMethod);
py_enum_display!(PyTermOrdering);
py_enum_display!(PyDemodDirection);
py_enum_display!(PyEqualitySide);
py_enum_display!(PyRewriteRuleKind);

#[pyclass(name = "Term", module = "prover9", unsendable)]
struct PyTerm {
    inner: prover9::Term,
}

impl From<prover9::Term> for PyTerm {
    fn from(inner: prover9::Term) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyTerm {
    #[new]
    fn new(input: &str) -> PyResult<Self> {
        prover9::Term::parse(input)
            .map(Into::into)
            .map_err(to_py_err)
    }

    #[staticmethod]
    fn parse(input: &str) -> PyResult<Self> {
        Self::new(input)
    }

    #[staticmethod]
    fn var(var: usize) -> PyResult<Self> {
        prover9::Term::var(var).map(Into::into).map_err(to_py_err)
    }

    #[staticmethod]
    fn atom(name: &str) -> PyResult<Self> {
        prover9::Term::atom(name).map(Into::into).map_err(to_py_err)
    }

    #[staticmethod]
    fn app(name: &str, children: Vec<Py<PyTerm>>, py: Python<'_>) -> PyResult<Self> {
        let children = children
            .into_iter()
            .map(|child| child.borrow(py).inner.clone())
            .collect();
        prover9::Term::app_vec(name, children)
            .map(Into::into)
            .map_err(to_py_err)
    }

    fn children(&self) -> Vec<PyTerm> {
        self.inner
            .children()
            .into_iter()
            .map(|child| child.to_owned().into())
            .collect()
    }

    fn is_variable(&self) -> bool {
        self.inner.as_ref().is_variable()
    }

    fn var_num(&self) -> Option<usize> {
        self.inner.as_ref().var_num()
    }

    fn symbol(&self) -> Option<String> {
        self.inner.as_ref().symbol().map(str::to_owned)
    }

    fn arity(&self) -> usize {
        self.inner.as_ref().arity()
    }

    fn is_ground(&self) -> bool {
        self.inner.is_ground()
    }

    fn depth(&self) -> usize {
        self.inner.depth()
    }

    fn symbol_count(&self) -> usize {
        self.inner.symbol_count()
    }

    fn occurs_in(&self, other: &PyTerm) -> bool {
        self.inner.occurs_in(&other.inner)
    }

    fn unify(&self, other: &PyTerm) -> Option<PyUnification> {
        self.inner.unify(&other.inner).map(Into::into)
    }

    fn matches(&self, target: &PyTerm) -> Option<Vec<PySubstitution>> {
        self.inner
            .matches(&target.inner)
            .map(|items| items.into_iter().map(Into::into).collect())
    }

    fn order(&self, other: &PyTerm) -> PyTermOrdering {
        self.inner.order(&other.inner).into()
    }

    fn lrpo_gt(&self, other: &PyTerm) -> bool {
        self.inner.lrpo_gt(&other.inner)
    }

    fn kbo_gt(&self, other: &PyTerm) -> bool {
        self.inner.kbo_gt(&other.inner)
    }

    fn greater_than_current_ordering(&self, other: &PyTerm) -> bool {
        self.inner.greater_than_current_ordering(&other.inner)
    }

    fn __copy__(&self) -> Self {
        self.inner.clone().into()
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> Self {
        self.inner.clone().into()
    }

    fn __richcmp__(&self, other: &PyTerm, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.inner == other.inner,
            CompareOp::Ne => self.inner != other.inner,
            _ => false,
        }
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Term({})", self.inner)
    }
}

#[pyclass(name = "Literal", module = "prover9", unsendable)]
struct PyLiteral {
    inner: prover9::Literal,
}

impl From<prover9::Literal> for PyLiteral {
    fn from(inner: prover9::Literal) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyLiteral {
    #[staticmethod]
    fn positive(atom: &PyTerm) -> Self {
        prover9::Literal::positive(atom.inner.clone()).into()
    }

    #[staticmethod]
    fn negative(atom: &PyTerm) -> Self {
        prover9::Literal::negative(atom.inner.clone()).into()
    }

    fn is_positive(&self) -> bool {
        self.inner.is_positive()
    }

    fn atom(&self) -> PyTerm {
        self.inner.atom().to_owned().into()
    }

    fn is_positive_equality(&self) -> bool {
        self.inner.is_positive_equality()
    }

    fn is_negative_equality(&self) -> bool {
        self.inner.is_negative_equality()
    }

    fn equality_sides(&self) -> Option<(PyTerm, PyTerm)> {
        self.inner
            .equality_sides()
            .map(|(left, right)| (left.into(), right.into()))
    }

    fn to_term(&self) -> PyTerm {
        self.inner.to_term().into()
    }

    fn __copy__(&self) -> Self {
        self.inner.clone().into()
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> Self {
        self.inner.clone().into()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Literal({})", self.inner)
    }
}

#[pyclass(name = "LiteralList", module = "prover9", unsendable)]
struct PyLiteralList {
    inner: prover9::LiteralList,
}

impl From<prover9::LiteralList> for PyLiteralList {
    fn from(inner: prover9::LiteralList) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyLiteralList {
    #[new]
    fn new(literals: Vec<Py<PyLiteral>>, py: Python<'_>) -> Self {
        let literals = literals
            .into_iter()
            .map(|literal| literal.borrow(py).inner.clone())
            .collect();
        prover9::LiteralList::new(literals).into()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn positive_count(&self) -> usize {
        self.inner.positive_count()
    }

    fn negative_count(&self) -> usize {
        self.inner.negative_count()
    }

    fn get(&self, index: usize) -> Option<PyLiteral> {
        self.inner.get(index).map(Into::into)
    }

    fn to_term(&self) -> Option<PyTerm> {
        self.inner.to_term().map(Into::into)
    }

    fn __copy__(&self) -> Self {
        self.inner.clone().into()
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> Self {
        self.inner.clone().into()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.inner)
    }
}

#[pyclass(name = "Clause", module = "prover9", unsendable)]
struct PyClause {
    inner: prover9::Clause,
}

impl From<prover9::Clause> for PyClause {
    fn from(inner: prover9::Clause) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyClause {
    #[new]
    fn new(term: &PyTerm) -> Self {
        prover9::Clause::from_term(term.inner.clone()).into()
    }

    #[staticmethod]
    fn from_term(term: &PyTerm) -> Self {
        Self::new(term)
    }

    fn to_term(&self) -> PyTerm {
        self.inner.to_term().into()
    }

    fn literals(&self) -> PyLiteralList {
        self.inner.literals().into()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn positive_count(&self) -> usize {
        self.inner.positive_count()
    }

    fn negative_count(&self) -> usize {
        self.inner.negative_count()
    }

    fn is_unit(&self) -> bool {
        self.inner.is_unit()
    }

    fn is_horn(&self) -> bool {
        self.inner.is_horn()
    }

    fn is_positive(&self) -> bool {
        self.inner.is_positive()
    }

    fn is_negative(&self) -> bool {
        self.inner.is_negative()
    }

    fn is_mixed(&self) -> bool {
        self.inner.is_mixed()
    }

    fn is_ground(&self) -> bool {
        self.inner.is_ground()
    }

    fn depth(&self) -> usize {
        self.inner.depth()
    }

    fn symbol_count(&self) -> usize {
        self.inner.symbol_count()
    }

    fn is_positive_equality_unit(&self) -> bool {
        self.inner.is_positive_equality_unit()
    }

    fn is_negative_equality_unit(&self) -> bool {
        self.inner.is_negative_equality_unit()
    }

    fn contains_positive_equality(&self) -> bool {
        self.inner.contains_positive_equality()
    }

    fn contains_equality(&self) -> bool {
        self.inner.contains_equality()
    }

    fn only_equality_literals(&self) -> bool {
        self.inner.only_equality_literals()
    }

    fn is_tautology(&self) -> bool {
        self.inner.is_tautology()
    }

    fn maximal_literal_indices(&self) -> Vec<usize> {
        self.inner.maximal_literal_indices()
    }

    fn maximal_signed_literal_indices(&self) -> Vec<usize> {
        self.inner.maximal_signed_literal_indices()
    }

    fn number_of_maximal_literals(&self) -> usize {
        self.inner.number_of_maximal_literals()
    }

    fn paramodulate_into(
        &self,
        from_literal_index: usize,
        from_side: PyEqualitySide,
        into_clause: &PyClause,
        into_literal_index: usize,
        into_path: Vec<usize>,
        allow_into_vars: bool,
    ) -> Option<Self> {
        self.inner
            .paramodulate_into(
                from_literal_index,
                from_side.into(),
                &into_clause.inner,
                into_literal_index,
                &into_path,
                allow_into_vars,
            )
            .map(Into::into)
    }

    fn resolve_with(
        &self,
        self_literal_index: usize,
        other: &PyClause,
        other_literal_index: usize,
    ) -> Option<Self> {
        self.inner
            .resolve_with(self_literal_index, &other.inner, other_literal_index)
            .map(Into::into)
    }

    fn resolve_with_flipped_other(
        &self,
        self_literal_index: usize,
        other: &PyClause,
        other_literal_index: usize,
    ) -> Option<Self> {
        self.inner
            .resolve_with_flipped_other(self_literal_index, &other.inner, other_literal_index)
            .map(Into::into)
    }

    fn resolve_with_xx(&self, literal_index: usize) -> Option<Self> {
        self.inner.resolve_with_xx(literal_index).map(Into::into)
    }

    fn factor_with(&self, first_literal_index: usize, second_literal_index: usize) -> Option<Self> {
        self.inner
            .factor_with(first_literal_index, second_literal_index)
            .map(Into::into)
    }

    fn merge_duplicate_literals(&self) -> Self {
        self.inner.merge_duplicate_literals().into()
    }

    fn simplify_basic(&self) -> Option<Self> {
        self.inner.simplify_basic().map(Into::into)
    }

    fn renumber_variables(&mut self, max_vars: usize) {
        self.inner.renumber_variables(max_vars);
    }

    fn demodulate_once_with(
        &self,
        demodulator: &PyClause,
        direction: PyDemodDirection,
    ) -> Option<Self> {
        self.inner
            .demodulate_once_with(&demodulator.inner, direction.into())
            .map(Into::into)
    }

    fn subsumes(&self, other: &PyClause) -> bool {
        self.inner.subsumes(&other.inner)
    }

    fn subsumes_bt(&self, other: &PyClause) -> bool {
        self.inner.subsumes_bt(&other.inner)
    }

    fn __copy__(&self) -> Self {
        self.inner.clone().into()
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> Self {
        self.inner.clone().into()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Clause({})", self.inner)
    }
}

#[pyclass(name = "Substitution", module = "prover9", unsendable)]
struct PySubstitution {
    inner: prover9::Substitution,
}

impl From<prover9::Substitution> for PySubstitution {
    fn from(inner: prover9::Substitution) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PySubstitution {
    #[getter]
    fn var(&self) -> usize {
        self.inner.var
    }

    #[getter]
    fn term(&self) -> PyTerm {
        self.inner.term.clone().into()
    }

    fn __repr__(&self) -> String {
        format!("Substitution(var={}, term={})", self.inner.var, self.inner.term)
    }
}

#[pyclass(name = "Unification", module = "prover9", unsendable)]
struct PyUnification {
    inner: prover9::Unification,
}

impl From<prover9::Unification> for PyUnification {
    fn from(inner: prover9::Unification) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyUnification {
    #[getter]
    fn left(&self) -> PyTerm {
        self.inner.left.clone().into()
    }

    #[getter]
    fn right(&self) -> PyTerm {
        self.inner.right.clone().into()
    }

    #[getter]
    fn left_substitution(&self) -> Vec<PySubstitution> {
        self.inner
            .left_substitution
            .clone()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    #[getter]
    fn right_substitution(&self) -> Vec<PySubstitution> {
        self.inner
            .right_substitution
            .clone()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn __repr__(&self) -> String {
        format!("Unification(left={}, right={})", self.inner.left, self.inner.right)
    }
}

#[pyclass(name = "CnfProblem", module = "prover9", unsendable)]
struct PyCnfProblem {
    inner: prover9::CnfProblem,
}

impl From<prover9::CnfProblem> for PyCnfProblem {
    fn from(inner: prover9::CnfProblem) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyCnfProblem {
    #[new]
    fn new(input: &str) -> PyResult<Self> {
        prover9::CnfProblem::parse_tptp_cnf(input)
            .map(Into::into)
            .map_err(to_py_err)
    }

    #[staticmethod]
    fn parse_tptp_cnf(input: &str) -> PyResult<Self> {
        Self::new(input)
    }

    fn clauses(&self) -> Vec<PyClause> {
        self.inner
            .clauses()
            .iter()
            .cloned()
            .map(Into::into)
            .collect()
    }

    fn prove_unsat(&self) -> Option<PyClause> {
        self.inner.prove_unsat().map(Into::into)
    }

    fn __copy__(&self) -> Self {
        self.inner.clone().into()
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> Self {
        self.inner.clone().into()
    }

    fn __repr__(&self) -> String {
        format!("CnfProblem(clauses={})", self.inner.clauses().len())
    }
}

#[pyclass(name = "RewriteSet", module = "prover9", unsendable)]
struct PyRewriteSet {
    inner: prover9::RewriteSet,
}

impl From<prover9::RewriteSet> for PyRewriteSet {
    fn from(inner: prover9::RewriteSet) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyRewriteSet {
    #[new]
    fn new() -> Self {
        prover9::RewriteSet::new().into()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn rules(&self) -> Vec<PyClause> {
        self.inner.rules().iter().cloned().map(Into::into).collect()
    }

    fn insert_rule(&mut self, rule: &PyClause) -> PyResult<PyRewriteRuleKind> {
        self.inner
            .insert_rule(&rule.inner)
            .map(Into::into)
            .map_err(to_py_err)
    }

    fn rewrite_term(&self, term: &PyTerm) -> PyTerm {
        self.inner.rewrite_term(&term.inner).into()
    }

    fn rewrite_clause(&self, clause: &PyClause) -> PyClause {
        self.inner.rewrite_clause(&clause.inner).into()
    }

    fn __repr__(&self) -> String {
        format!("RewriteSet(rule_count={})", self.inner.len())
    }
}

#[pyclass(name = "WildDiscrimIndex", module = "prover9", unsendable)]
struct PyWildDiscrimIndex {
    inner: prover9::WildDiscrimIndex,
}

impl From<prover9::WildDiscrimIndex> for PyWildDiscrimIndex {
    fn from(inner: prover9::WildDiscrimIndex) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyWildDiscrimIndex {
    #[new]
    fn new() -> Self {
        prover9::WildDiscrimIndex::new().into()
    }

    fn insert(&mut self, term: &PyTerm) {
        self.inner.insert(&term.inner);
    }

    fn retrieve_generalizations(&self, query: &PyTerm) -> Vec<PyTerm> {
        self.inner
            .retrieve_generalizations(&query.inner)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn __repr__(&self) -> String {
        "WildDiscrimIndex()".to_owned()
    }
}

#[pyfunction]
fn parse_tptp_cnf(input: &str) -> PyResult<PyCnfProblem> {
    PyCnfProblem::parse_tptp_cnf(input)
}

#[pyfunction]
fn set_order_method(method: PyOrderMethod) {
    prover9::set_order_method(method.into());
}

#[pyfunction]
fn set_symbol_precedence(name: &str, arity: usize, precedence: i32) -> PyResult<()> {
    prover9::set_symbol_precedence(name, arity, precedence).map_err(to_py_err)
}

#[pymodule]
fn _prover9(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyOrderMethod>()?;
    m.add_class::<PyTermOrdering>()?;
    m.add_class::<PyDemodDirection>()?;
    m.add_class::<PyEqualitySide>()?;
    m.add_class::<PyRewriteRuleKind>()?;
    m.add_class::<PyTerm>()?;
    m.add_class::<PyLiteral>()?;
    m.add_class::<PyLiteralList>()?;
    m.add_class::<PyClause>()?;
    m.add_class::<PySubstitution>()?;
    m.add_class::<PyUnification>()?;
    m.add_class::<PyCnfProblem>()?;
    m.add_class::<PyRewriteSet>()?;
    m.add_class::<PyWildDiscrimIndex>()?;
    m.add_function(wrap_pyfunction!(parse_tptp_cnf, m)?)?;
    m.add_function(wrap_pyfunction!(set_order_method, m)?)?;
    m.add_function(wrap_pyfunction!(set_symbol_precedence, m)?)?;
    Ok(())
}
