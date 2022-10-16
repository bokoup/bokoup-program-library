INSERT INTO promo (
    id,
    owner,
    mint,
    metadata,
    mint_count,
    burn_count,
    max_mint,
    max_burn,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
ON CONFLICT ON CONSTRAINT promo_pkey DO UPDATE 
    SET
        owner = EXCLUDED.owner,
        mint = EXCLUDED.mint,
        metadata = EXCLUDED.metadata,
        mint_count = EXCLUDED.mint_count,
        burn_count = EXCLUDED.burn_count,
        max_mint = EXCLUDED.max_mint,
        max_burn = EXCLUDED.max_burn,
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > promo.slot
        OR (
            EXCLUDED.slot = promo.slot
            AND EXCLUDED.write_version > promo.write_version
        )
RETURNING created_at = modified_at