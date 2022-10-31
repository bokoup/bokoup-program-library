INSERT INTO delegate_promo_token (
    signature,
    payer,
    delegate,
    token_owner,
    authority,
    promo,
    token_account,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
ON CONFLICT ON CONSTRAINT delegate_promo_token_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        delegate = EXCLUDED.delegate,
        token_owner = EXCLUDED.token_owner,
        authority = EXCLUDED.authority,
        promo = EXCLUDED.promo,
        token_account = EXCLUDED.token_account,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > delegate_promo_token.slot
RETURNING created_at = modified_at