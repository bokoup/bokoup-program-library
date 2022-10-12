INSERT INTO create_promo (
    signature,
    promo_owner,
    mint,
    metadata,
    authority,
    promo,
    platform,
    admin_settings,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
ON CONFLICT ON CONSTRAINT create_promo_pkey DO UPDATE 
    SET
        promo_owner = EXCLUDED.promo_owner,
        mint = EXCLUDED.mint,
        metadata = EXCLUDED.metadata,
        authority = EXCLUDED.authority,
        promo = EXCLUDED.promo,
        platform = EXCLUDED.platform,
        admin_settings = EXCLUDED.admin_settings,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > create_promo.slot
RETURNING created_at = modified_at