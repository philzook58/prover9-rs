use prover9_sys as sys;

use crate::term::Term;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Substitution {
    pub var: usize,
    pub term: Term,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unification {
    pub left: Term,
    pub right: Term,
    pub left_substitution: Vec<Substitution>,
    pub right_substitution: Vec<Substitution>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OrderMethod {
    Lrpo,
    Lpo,
    Rpo,
    Kbo,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TermOrdering {
    NotComparable,
    SameAs,
    LessThan,
    GreaterThan,
    LessThanOrSameAs,
    GreaterThanOrSameAs,
    NotLessThan,
    NotGreaterThan,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DemodDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EqualitySide {
    Left,
    Right,
}

impl OrderMethod {
    pub(crate) fn into_raw(self) -> sys::Order_method {
        match self {
            OrderMethod::Lrpo => sys::Order_method_LRPO_METHOD,
            OrderMethod::Lpo => sys::Order_method_LPO_METHOD,
            OrderMethod::Rpo => sys::Order_method_RPO_METHOD,
            OrderMethod::Kbo => sys::Order_method_KBO_METHOD,
        }
    }
}

impl TermOrdering {
    pub(crate) fn from_raw(raw: sys::Ordertype) -> Self {
        match raw {
            sys::Ordertype_NOT_COMPARABLE => Self::NotComparable,
            sys::Ordertype_SAME_AS => Self::SameAs,
            sys::Ordertype_LESS_THAN => Self::LessThan,
            sys::Ordertype_GREATER_THAN => Self::GreaterThan,
            sys::Ordertype_LESS_THAN_OR_SAME_AS => Self::LessThanOrSameAs,
            sys::Ordertype_GREATER_THAN_OR_SAME_AS => Self::GreaterThanOrSameAs,
            sys::Ordertype_NOT_LESS_THAN => Self::NotLessThan,
            sys::Ordertype_NOT_GREATER_THAN => Self::NotGreaterThan,
            _ => panic!("unknown Ordertype value: {raw}"),
        }
    }
}

impl DemodDirection {
    pub(crate) fn into_raw(self) -> i32 {
        match self {
            DemodDirection::LeftToRight => 1,
            DemodDirection::RightToLeft => 2,
        }
    }
}

impl EqualitySide {
    pub(crate) fn into_raw(self) -> i32 {
        match self {
            EqualitySide::Left => 0,
            EqualitySide::Right => 1,
        }
    }
}
