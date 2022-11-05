INSERT INTO create_promo_group (
    signature,
    payer,
    seed,
    promo_group,
    lamports,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT ON CONSTRAINT create_promo_group_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        seed = EXCLUDED.seed,
        promo_group = EXCLUDED.promo_group,
        lamports = EXCLUDED.lamports,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > create_promo_group.slot
RETURNING created_at = modified_at