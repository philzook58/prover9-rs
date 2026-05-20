#include "../wrapper.h"
#include <limits.h>
#include "top_input.h"

void prover9_init(void) {
  init_standard_ladr();
}

int prover9_term_is_variable(Term term) {
  return VARIABLE(term);
}

int prover9_term_is_constant(Term term) {
  return CONSTANT(term);
}

int prover9_term_is_complex(Term term) {
  return COMPLEX(term);
}

int prover9_term_var_num(Term term) {
  return VARNUM(term);
}

int prover9_term_sym_num(Term term) {
  return SYMNUM(term);
}

int prover9_term_arity(Term term) {
  return ARITY(term);
}

const char *prover9_term_symbol(Term term) {
  if (VARIABLE(term)) {
    return 0;
  }
  return sn_to_str(SYMNUM(term));
}

Term prover9_term_arg(Term term, int index) {
  return ARG(term, index);
}

void prover9_term_set_arg(Term term, int index, Term arg) {
  ARG(term, index) = arg;
}

int prover9_max_vars(void) {
  return MAX_VARS;
}

int prover9_match(Term pattern, Context context, Term target, Trail *trail) {
  return match(pattern, context, target, trail);
}

int prover9_literal_sign(Literals literal) {
  return literal->sign;
}

Term prover9_literal_atom(Literals literal) {
  return literal->atom;
}

Literals prover9_topform_literals(Topform topform) {
  return topform->literals;
}

Term prover9_context_binding_term(Context context, int varnum) {
  if (varnum < 0 || varnum >= MAX_VARS) {
    return 0;
  }
  return context->terms[varnum];
}

Context prover9_context_binding_context(Context context, int varnum) {
  if (varnum < 0 || varnum >= MAX_VARS) {
    return 0;
  }
  return context->contexts[varnum];
}

static int prover9_can_demodulate_term(Term current, Term atom, int direction,
                                       int lex_order_vars) {
  int arity;
  int i;

  if (VARIABLE(current)) {
    return 0;
  }

  arity = ARITY(current);
  for (i = 0; i < arity; i++) {
    if (prover9_can_demodulate_term(ARG(current, i), atom, direction,
                                    lex_order_vars)) {
      return 1;
    }
  }

  {
    Context context = get_context();
    Trail trail = NULL;
    int match_left = (direction == 1);
    Term pattern = ARG(atom, match_left ? 0 : 1);
    Term replacement = ARG(atom, match_left ? 1 : 0);
    int ok = 0;

    if (match(pattern, context, current, &trail)) {
      Term contractum = apply_demod(replacement, context, -1);
      ok = oriented_eq(atom) || term_greater(current, contractum, lex_order_vars);
      zap_term(contractum);
      undo_subst(trail);
    }

    free_context(context);
    return ok;
  }
}

int prover9_can_demodulate(Topform clause, Topform demodulator, int direction,
                           int lex_order_vars) {
  Literals literal;

  if (clause == NULL || demodulator == NULL || !pos_eq_unit(demodulator->literals)) {
    return 0;
  }

  for (literal = clause->literals; literal != NULL; literal = literal->next) {
    if (prover9_can_demodulate_term(literal->atom, demodulator->literals->atom,
                                    direction, lex_order_vars)) {
      return 1;
    }
  }

  return 0;
}

Topform prover9_para_pos(Topform from_clause, int from_lit_num, int from_side,
                         Topform into_clause, int into_lit_num,
                         const int *into_path, int into_path_len,
                         int allow_into_vars) {
  Context cf;
  Context ci;
  Trail tr = NULL;
  BOOL ok;
  Ilist from_pos = NULL;
  Ilist into_pos = NULL;
  Literals from_lit;
  Literals into_lit;
  Term alpha;
  Term into_term;
  Topform result = NULL;
  int i;

  if (from_clause == NULL || into_clause == NULL || from_side < 0 || from_side > 1 ||
      from_lit_num <= 0 || into_lit_num <= 0) {
    return NULL;
  }

  from_lit = ith_literal(from_clause->literals, from_lit_num);
  into_lit = ith_literal(into_clause->literals, into_lit_num);
  if (from_lit == NULL || into_lit == NULL || !eq_term(from_lit->atom)) {
    return NULL;
  }

  from_pos = ilist_append(from_pos, from_lit_num);
  from_pos = ilist_append(from_pos, from_side + 1);
  into_pos = ilist_append(into_pos, into_lit_num);
  for (i = 0; i < into_path_len; i++) {
    if (into_path[i] < 1) {
      zap_ilist(from_pos);
      zap_ilist(into_pos);
      return NULL;
    }
    into_pos = ilist_append(into_pos, into_path[i]);
  }

  alpha = ARG(from_lit->atom, from_side);
  into_term = term_at_pos(into_lit->atom, into_pos->next);
  if (into_term == NULL || (!allow_into_vars && VARIABLE(into_term))) {
    zap_ilist(from_pos);
    zap_ilist(into_pos);
    return NULL;
  }

  cf = get_context();
  ci = get_context();
  ok = unify(alpha, cf, into_term, ci, &tr);
  if (ok) {
    if (allow_into_vars) {
      result = para_pos2(from_clause, from_pos, into_clause, into_pos);
    }
    else {
      result = para_pos(from_clause, from_pos, into_clause, into_pos);
    }
    undo_subst(tr);
  }

  free_context(cf);
  free_context(ci);
  zap_ilist(from_pos);
  zap_ilist(into_pos);
  return result;
}

Topform prover9_factor2(Topform clause, int lit1_num, int lit2_num,
                        int renumber_vars) {
  Literals l1;
  Literals l2;
  Context subst;
  Trail tr = NULL;
  Topform result = NULL;
  Literals lit;

  if (clause == NULL || lit1_num <= 0 || lit2_num <= 0 || lit1_num == lit2_num) {
    return NULL;
  }

  l1 = ith_literal(clause->literals, lit1_num);
  l2 = ith_literal(clause->literals, lit2_num);
  if (l1 == NULL || l2 == NULL || l1->sign != l2->sign) {
    return NULL;
  }

  subst = get_context();
  if (unify(l1->atom, subst, l2->atom, subst, &tr)) {
    result = get_topform();
    result->justification = factor_just(clause, lit1_num, lit2_num);
    for (lit = clause->literals; lit; lit = lit->next) {
      if (lit != l2) {
        result->literals = append_literal(result->literals, apply_lit(lit, subst));
      }
    }
    if (renumber_vars) {
      renumber_variables(result, MAX_VARS);
    }
    undo_subst(tr);
    upward_clause_links(result);
    result->attributes = cat_att(
        result->attributes,
        inheritable_att_instances(clause->attributes, subst));
  }
  free_context(subst);
  return result;
}

Mindex prover9_rewrite_index_new(void) {
  return mindex_init(DISCRIM_BIND, ORDINARY_UNIF, 0);
}

void prover9_rewrite_index_destroy(Mindex idx) {
  if (idx != NULL) {
    mindex_destroy(idx);
  }
}

int prover9_rewrite_rule_type(Topform clause) {
  if (clause == NULL) {
    return NOT_DEMODULATOR;
  }
  return demodulator_type(clause, -1, 1);
}

void prover9_rewrite_index_insert(Topform rule, int rule_type, Mindex idx) {
  if (rule != NULL && idx != NULL && rule_type != NOT_DEMODULATOR) {
    idx_demodulator(rule, rule_type, INSERT, idx);
  }
}

Term prover9_rewrite_term(Term term, Mindex idx, int lex_order_vars) {
  Ilist just = NULL;
  Term result;

  if (term == NULL || idx == NULL) {
    return term;
  }

  result = demodulate(term, idx, &just, lex_order_vars);
  zap_ilist(just);
  return result;
}

Topform prover9_rewrite_clause(Topform clause, Mindex idx, int lex_order_vars) {
  Topform copy;
  int step_limit = INT_MAX;
  int increase_limit = INT_MAX;

  if (clause == NULL || idx == NULL) {
    return NULL;
  }

  copy = copy_clause(clause);
  fdemod_clause(copy, idx, &step_limit, &increase_limit, lex_order_vars);
  return copy;
}
