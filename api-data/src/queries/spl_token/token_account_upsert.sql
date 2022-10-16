INSERT INTO token_account (
    id,
    mint,
    owner,
    amount,
    delegate,
    state,
    is_native,
    delegated_amount,
    close_authority,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
ON CONFLICT ON CONSTRAINT token_account_pkey DO UPDATE 
    SET
        mint = EXCLUDED.mint, 
        owner = EXCLUDED.owner,
        amount = EXCLUDED.amount,
        delegate = EXCLUDED.delegate,
        state = EXCLUDED.state,
        is_native = EXCLUDED.is_native,
        delegated_amount = EXCLUDED.delegated_amount,
        close_authority = EXCLUDED.close_authority, 
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > token_account.slot
        OR (
            EXCLUDED.slot = token_account.slot
            AND EXCLUDED.write_version > token_account.write_version
        )
RETURNING created_at = modified_at