INSERT INTO mint (
    id,
    freeze_authority,
    mint_authority,
    is_initialized,
    supply,
    decimals,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT ON CONSTRAINT mint_pkey DO UPDATE 
    SET
        freeze_authority = EXCLUDED.freeze_authority,
        mint_authority = EXCLUDED.mint_authority,
        is_initialized = EXCLUDED.is_initialized,
        supply = EXCLUDED.supply ,
        decimals = EXCLUDED.decimals,
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > mint.slot
        OR (
            EXCLUDED.slot = mint.slot
            AND EXCLUDED.write_version > mint.write_version
        )
RETURNING created_at = modified_at