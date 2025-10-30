/*
Logup* (Lev Soukhanov)

https://eprint.iacr.org/2025/946.pdf

*/

use multilinear_toolkit::prelude::*;
use p3_field::{ExtensionField, PrimeField64};
use utils::ToUsize;

use p3_field::PrimeCharacteristicRing;
use tracing::{info_span, instrument};
use utils::{FSProver, FSVerifier};

use crate::{
    MIN_VARS_FOR_PACKING,
    quotient_gkr::{prove_gkr_quotient, verify_gkr_quotient},
};

#[derive(Debug)]
pub struct LogupStarStatements<EF> {
    pub on_indexes: Evaluation<EF>,
    pub on_table: Evaluation<EF>,
    pub on_pushforward: Vec<Evaluation<EF>>,
}

#[instrument(skip_all)]
pub fn prove_logup_star<EF>(
    prover_state: &mut FSProver<EF, impl FSChallenger<EF>>,
    table: &MleRef<'_, EF>,
    indexes: &[PF<EF>],
    claimed_value: EF,
    poly_eq_point: &[EF],
    pushforward: &[EF], // already commited
    max_index: Option<usize>,
) -> LogupStarStatements<EF>
where
    EF: ExtensionField<PF<EF>>,
    PF<EF>: PrimeField64,
{
    let table_length = table.unpacked_len();
    let indexes_length = indexes.len();
    let packing = log2_strict_usize(table_length) >= MIN_VARS_FOR_PACKING
        && log2_strict_usize(indexes_length) >= MIN_VARS_FOR_PACKING;
    let mut max_index = max_index.unwrap_or(table_length);
    if packing {
        max_index = max_index.div_ceil(packing_width::<EF>());
    }

    let (poly_eq_point_packed, pushforward_packed, table_packed) =
        info_span!("packing").in_scope(|| {
            (
                MleRef::Extension(poly_eq_point).pack_if(packing),
                MleRef::Extension(pushforward).pack_if(packing),
                table.pack_if(packing),
            )
        });

    let (sc_point, inner_evals, prod) =
        info_span!("logup_star sumcheck", table_length, indexes_length).in_scope(|| {
            let (sc_point, prod, table_folded, pushforward_folded) = run_product_sumcheck(
                &table_packed.by_ref(),
                &pushforward_packed.by_ref(),
                prover_state,
                claimed_value,
                table.n_vars(),
            );
            let inner_evals = vec![
                table_folded.as_extension().unwrap()[0],
                pushforward_folded.as_extension().unwrap()[0],
            ];
            (sc_point, inner_evals, prod)
        });

    let table_eval = inner_evals[0];
    prover_state.add_extension_scalar(table_eval);
    // delayed opening
    let on_table = Evaluation::new(sc_point.clone(), table_eval);

    let pushforwardt_eval = inner_evals[1];
    prover_state.add_extension_scalar(pushforwardt_eval);
    // delayed opening
    let mut on_pushforward = vec![Evaluation::new(sc_point, pushforwardt_eval)];

    // sanity check
    assert_eq!(prod, table_eval * pushforwardt_eval);

    let c = prover_state.sample();

    let (claim_left, _, eval_c_minux_indexes) = prove_gkr_quotient(
        prover_state,
        &poly_eq_point_packed.by_ref(),
        (c, indexes),
        None,
    );

    let increments = (0..table.unpacked_len())
        .into_par_iter()
        .map(PF::<EF>::from_usize)
        .collect::<Vec<_>>();
    let (claim_right, pushforward_final_eval, _) = prove_gkr_quotient(
        prover_state,
        &pushforward_packed.by_ref(),
        (c, &increments),
        Some(max_index),
    );

    let final_point_left = claim_left.point[1..].to_vec();
    let indexes_final_eval = c - eval_c_minux_indexes;
    prover_state.add_extension_scalar(indexes_final_eval);
    let on_indexes = Evaluation::new(final_point_left, indexes_final_eval);

    prover_state.add_extension_scalar(pushforward_final_eval);
    on_pushforward.push(Evaluation::new(
        claim_right.point[1..].to_vec(),
        pushforward_final_eval,
    ));

    // These statements remained to be proven
    LogupStarStatements {
        on_indexes,
        on_table,
        on_pushforward,
    }
}

pub fn verify_logup_star<EF>(
    verifier_state: &mut FSVerifier<EF, impl FSChallenger<EF>>,
    log_table_len: usize,
    log_indexes_len: usize,
    claims: &[Evaluation<EF>],
    alpha: EF, // batching challenge
) -> Result<LogupStarStatements<EF>, ProofError>
where
    EF: ExtensionField<PF<EF>>,
    PF<EF>: PrimeField64,
{
    let (sum, postponed) =
        sumcheck_verify(verifier_state, log_table_len, 2).map_err(|_| ProofError::InvalidProof)?;

    if sum
        != claims
            .iter()
            .zip(alpha.powers())
            .map(|(c, a)| c.value * a)
            .sum::<EF>()
    {
        return Err(ProofError::InvalidProof);
    }

    let table_eval = verifier_state.next_extension_scalar()?;
    let pushforward_eval = verifier_state.next_extension_scalar()?;

    let on_table = Evaluation::new(postponed.point.clone(), table_eval);
    let mut on_pushforward = vec![Evaluation::new(postponed.point, pushforward_eval)];

    if table_eval * pushforward_eval != postponed.value {
        return Err(ProofError::InvalidProof);
    }

    let random_challenge = verifier_state.sample(); // "c" in the paper

    let (quotient_left, claim_left) = verify_gkr_quotient(verifier_state, log_indexes_len + 1)?;
    let (quotient_right, claim_right) = verify_gkr_quotient(verifier_state, log_table_len + 1)?;

    if quotient_left != quotient_right {
        return Err(ProofError::InvalidProof);
    }

    let final_point_left = MultilinearPoint(claim_left.point[1..].to_vec());
    let index_openined_value = verifier_state.next_extension_scalar()?;
    let on_indexes = Evaluation::new(final_point_left.clone(), index_openined_value);
    if claim_left.value
        != claims
            .iter()
            .zip(alpha.powers())
            .map(|(claim, a)| final_point_left.eq_poly_outside(&claim.point) * a)
            .sum::<EF>()
            * (EF::ONE - claim_left.point[0])
            + (random_challenge - index_openined_value) * claim_left.point[0]
    {
        return Err(ProofError::InvalidProof);
    }

    let final_point_right = claim_right.point[1..].to_vec();
    let pushforward_opening_value = verifier_state.next_extension_scalar()?;
    on_pushforward.push(Evaluation::new(
        final_point_right.clone(),
        pushforward_opening_value,
    ));

    let big_endian_mle = final_point_right
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &p)| p * EF::TWO.exp_u64(i as u64))
        .sum::<EF>();

    if claim_right.value
        != pushforward_opening_value * (EF::ONE - claim_right.point[0])
            + (random_challenge - big_endian_mle) * claim_right.point[0]
    {
        return Err(ProofError::InvalidProof);
    }

    // these statements remained to be verified
    Ok(LogupStarStatements {
        on_indexes,
        on_table,
        on_pushforward,
    })
}

#[instrument(skip_all)]
pub fn compute_pushforward<F: PrimeField64, EF: ExtensionField<EF>>(
    indexes: &[F],
    table_length: usize,
    poly_eq_point: &[EF],
) -> Vec<EF> {
    assert_eq!(indexes.len(), poly_eq_point.len());
    // TODO there are a lot of fun optimizations here
    let mut pushforward = EF::zero_vec(table_length);
    for (index, value) in indexes.iter().zip(poly_eq_point) {
        let index_usize = index.to_usize();
        pushforward[index_usize] += *value;
    }
    pushforward
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use p3_field::PrimeCharacteristicRing;
    use p3_koala_bear::{KoalaBear, QuinticExtensionFieldKB};
    use rand::{Rng, SeedableRng, rngs::StdRng};
    use utils::{build_challenger, init_tracing};

    type F = KoalaBear;
    type EF = QuinticExtensionFieldKB;

    #[test]
    fn test_logup_star() {
        for log_table_len in [1, 10] {
            for log_indexes_len in 1..10 {
                test_logup_star_helper(log_table_len, log_indexes_len);
            }
        }

        test_logup_star_helper(15, 17);
    }

    fn test_logup_star_helper(log_table_len: usize, log_indexes_len: usize) {
        init_tracing();

        let table_length = 1 << log_table_len;

        let indexes_len = 1 << log_indexes_len;

        let mut rng = StdRng::seed_from_u64(0);

        let table = (0..table_length).map(|_| rng.random()).collect::<Vec<F>>();

        let mut indexes = vec![];
        let mut values = vec![];
        let max_index = table_length * 3 / 4;
        for _ in 0..indexes_len {
            let index = rng.random_range(0..max_index);
            indexes.push(F::from_usize(index));
            values.push(table[index]);
        }

        // Commit to the table
        let commited_table = table.clone(); // Phony commitment for the example
        // commit to the indexes
        let commited_indexes = indexes.clone(); // Phony commitment for the example

        let challenger = build_challenger();

        let point = MultilinearPoint(
            (0..log_indexes_len)
                .map(|_| rng.random())
                .collect::<Vec<EF>>(),
        );

        let mut prover_state = FSProver::new(challenger.clone());
        let eval = values.evaluate(&point);

        let time = std::time::Instant::now();
        let poly_eq_point = info_span!("eval_eq").in_scope(|| eval_eq(&point));
        let pushforward = compute_pushforward(&indexes, table_length, &poly_eq_point);
        let claim = Evaluation::new(point, eval);

        prove_logup_star(
            &mut prover_state,
            &MleRef::Base(&commited_table),
            &commited_indexes,
            claim.value,
            &poly_eq_point,
            &pushforward,
            Some(max_index),
        );
        println!("Proving logup_star took {} ms", time.elapsed().as_millis());

        let mut verifier_state = FSVerifier::new(prover_state.proof_data().to_vec(), challenger);
        let statements = verify_logup_star(
            &mut verifier_state,
            log_table_len,
            log_indexes_len,
            &[claim],
            EF::ONE,
        )
        .unwrap();

        assert_eq!(
            indexes.evaluate(&statements.on_indexes.point),
            statements.on_indexes.value
        );
        assert_eq!(
            table.evaluate(&statements.on_table.point),
            statements.on_table.value
        );
        for eval in &statements.on_pushforward {
            assert_eq!(pushforward.evaluate(&eval.point), eval.value);
        }

        {
            let n_muls = 16;
            let slice = (0..(table_length + indexes_len) / packing_width::<EF>())
                .map(|_| rng.random())
                .collect::<Vec<EFPacking<EF>>>();
            let time = Instant::now();
            let sum = slice
                .par_iter()
                .map(|x| (0..n_muls).map(|_| *x).product::<EFPacking<EF>>())
                .sum::<EFPacking<EF>>();
            assert!(sum != EFPacking::<EF>::ONE);
            println!(
                "Optimal time we can hope for: {} ms",
                time.elapsed().as_millis()
            );
        }
    }
}
