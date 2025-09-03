# TODO

## Perf

- WHIR univariate skip
- Opti recursion bytecode
- inverse folding ordering in WHIR to enable Packing during sumcheck (more generally, TODO packing everywhere)
- one can "move out" the variable of the eq(.) polynomials out of the sumcheck computation in WHIR (as done in the PIOP)
- Extension field: dim 5/6
- Structured AIR: often no all the columns use both up/down -> only handle the used ones to speed up the PIOP zerocheck
- use RowMAjorMatrix instead of Vec<Vec> for witness
- Fill Precompile tables during bytecode execution
- Use Univariate Skip to commit to tables with k.2^n rows (k small)
- increase density of multi commitments -> we can almost reduce by 2x commitment costs
- avoid field embedding in the initial sumcheck of logup*, when table / values are in base field
- opti logup* GKR when the indexes are not a power of 2 (which is the case in the execution table)
- incremental merkle paths in whir-p3
- Experiment to increase degree, and reduce commitments, in Poseidon arithmetization
- Avoid embedding overhead on the flag, len, and index columns in the AIR table for dot products
- reduce to only 2 logup*, one vectorized, one not
- Batched logup*: when computing the eq() factor we can opti if the points contain boolean factor
- Recursion program: batch the initial leaf evals in a single multilineae evaluation (+ only 1 statement on the point per vm multilinear eval (instead of `n_vars` ones))
- Lev's trick to skip some low-level modular reduction
- Sumcheckcheck, case z = 0, no need to fold, only keep first half of the values (done in PR 33 by Lambda) (and also in WHIR?)
- Custom AVX2 / AVX512 / Neon implem in Plonky3 for all of the finite field operations (done for degree 4 extension, but not degree 5)
- leanISA compiler: inlined function
- batch the logup* in basefield. We should have only 2 logup* in memory (not 3)
- the 2 executions of the program, before generating the validity proof, can be merged, using some kind of placeholders
- XMSS aggregation program has 40% of unused memory!! -> TODO improve the compiler to reduce memory fragmentation

## Not Perf

- Whir batching: handle the case where the second polynomial is too small compared to the first one
- bounddary condition on dot_product table: first flag = 1
- verify correctness of the Grand Product check
- Proof size: replace all equality checks in the verifier algo by value deduction
- WIR recursion: batch the multilinear_eval calls on initial merkle leaves
- KoalaBear extension of degree 5: only AVX2 has been tested and benchmarked. TODO: AVX512 / Neon
- KoalaBear extension of degree 6: in order to use the (proven) Johnson bound in WHIR