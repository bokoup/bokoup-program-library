INSERT INTO create_promo (
    signature,
    payer,
    promo_group,
    mint,
    metadata,
    authority,
    promo,
    platform,
    admin_settings,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
ON CONFLICT ON CONSTRAINT create_promo_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        promo_group = EXCLUDED.promo_group,
        mint = EXCLUDED.mint,
        metadata = EXCLUDED.metadata,
        authority = EXCLUDED.authority,
        promo = EXCLUDED.promo,
        platform = EXCLUDED.platform,
        admin_settings = EXCLUDED.admin_settings,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > create_promo.slot
RETURNING created_at = modified_at