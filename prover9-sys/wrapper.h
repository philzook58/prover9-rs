#include "term.h"
#include "parse.h"
#include "discrimw.h"
#include "literals.h"
#include "termorder.h"
#include "topform.h"
#include "unify.h"
#include "parautil.h"
#include "demod.h"
#include "flatdemod.h"
#include "resolve.h"
#include "subsume.h"
#include "paramod.h"

void prover9_init(void);
int prover9_term_is_variable(Term term);
int prover9_term_is_constant(Term term);
int prover9_term_is_complex(Term term);
int prover9_term_var_num(Term term);
int prover9_term_sym_num(Term term);
int prover9_term_arity(Term term);
const char *prover9_term_symbol(Term term);
Term prover9_term_arg(Term term, int index);
void prover9_term_set_arg(Term term, int index, Term arg);
int prover9_max_vars(void);
int prover9_match(Term pattern, Context context, Term target, Trail *trail);
int prover9_literal_sign(Literals literal);
Term prover9_literal_atom(Literals literal);
Literals prover9_topform_literals(Topform topform);
Term prover9_context_binding_term(Context context, int varnum);
Context prover9_context_binding_context(Context context, int varnum);
int prover9_can_demodulate(Topform clause, Topform demodulator, int direction,
                           int lex_order_vars);
Topform prover9_para_pos(Topform from_clause, int from_lit_num, int from_side,
                         Topform into_clause, int into_lit_num,
                         const int *into_path, int into_path_len,
                         int allow_into_vars);
Topform prover9_factor2(Topform clause, int lit1_num, int lit2_num,
                        int renumber_vars);
Mindex prover9_rewrite_index_new(void);
void prover9_rewrite_index_destroy(Mindex idx);
int prover9_rewrite_rule_type(Topform clause);
int prover9_rewrite_prepare_rule(Topform clause, int allow_flips);
void prover9_rewrite_index_insert(Topform rule, int rule_type, Mindex idx);
void prover9_rewrite_index_remove(Topform rule, int rule_type, Mindex idx);
Term prover9_rewrite_term(Term term, Mindex idx, int lex_order_vars);
Topform prover9_rewrite_clause(Topform clause, Mindex idx, int lex_order_vars);
