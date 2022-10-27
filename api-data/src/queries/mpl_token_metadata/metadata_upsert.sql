INSERT INTO metadata (
    id,
    key,
    update_authority,
    mint,
    name,
    symbol,
    uri,
    metadata_json,
    seller_fee_basis_points,
    primary_sale_happened,
    is_mutable,
    edition_nonce,
    token_standard,
    collection_key,
    collection_verified,
    uses_use_method,
    uses_remaining,
    uses_total,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
ON CONFLICT ON CONSTRAINT metadata_pkey DO UPDATE 
    SET
        key = EXCLUDED.key,
        update_authority = EXCLUDED.update_authority,
        mint = EXCLUDED.mint,
        name = EXCLUDED.name,
        symbol = EXCLUDED.symbol,
        uri = EXCLUDED.uri,
        metadata_json = EXCLUDED.metadata_json,
        seller_fee_basis_points = EXCLUDED.seller_fee_basis_points,
        primary_sale_happened = EXCLUDED.primary_sale_happened,
        is_mutable = EXCLUDED.is_mutable,
        edition_nonce = EXCLUDED.edition_nonce,
        token_standard = EXCLUDED.token_standard,
        collection_key = EXCLUDED.collection_key,
        collection_verified = EXCLUDED.collection_verified,
        uses_use_method = EXCLUDED.uses_use_method,
        uses_remaining = EXCLUDED.uses_remaining,
        uses_total = EXCLUDED.uses_total,
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > metadata.slot
        OR (
            EXCLUDED.slot = metadata.slot
            AND EXCLUDED.write_version > metadata.write_version
        )
RETURNING created_at = modified_at