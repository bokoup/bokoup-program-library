INSERT INTO burn_delegated_promo_token (
    signature,
    payer,
    promo_group,
    mint,
    authority,
    promo,
    platform,
    admin_settings,
    token_account,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
ON CONFLICT ON CONSTRAINT burn_delegated_promo_token_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        promo_group = EXCLUDED.promo_group,
        mint = EXCLUDED.mint,
        authority = EXCLUDED.authority,
        promo = EXCLUDED.promo,
        platform = EXCLUDED.platform,
        admin_settings = EXCLUDED.admin_settings,
        token_account = EXCLUDED.token_account,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > burn_delegated_promo_token.slot
RETURNING created_at = modified_at