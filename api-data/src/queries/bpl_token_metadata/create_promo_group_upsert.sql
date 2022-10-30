INSERT INTO create_promo_group (
    signature,
    payer,
    promo_group,
    seed,
    lamports,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT ON CONSTRAINT create_promo_group_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        promo_group = EXCLUDED.promo_group,
        seed = EXCLUDED.seed,
        lamports = EXCLUDED.lamports,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > create_promo_group.slot
RETURNING created_at = modified_at