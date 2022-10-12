INSERT INTO burn_delegated_promo_token (
    signature,
    payer,
    mint,
    authority,
    promo,
    platform,
    admin_settings,
    token_account,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
ON CONFLICT ON CONSTRAINT burn_delegated_promo_token_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        mint = EXCLUDED.mint,
        authority = EXCLUDED.authority,
        promo = EXCLUDED.promo,
        platform = EXCLUDED.platform,
        admin_settings = EXCLUDED.admin_settings,
        token_account = EXCLUDED.token_account,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > burn_delegated_promo_token.slot
RETURNING created_at = modified_at