INSERT INTO promo (
    id,
    owner,
    mints,
    burns,
    max_mint,
    max_burn,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT ON CONSTRAINT promo_pkey DO UPDATE 
    SET
        owner = EXCLUDED.owner,
        mints = EXCLUDED.mints,
        burns = EXCLUDED.burns,
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