INSERT INTO purchase_receipt (
    id,
    bookkeeper,
    buyer,
    seller,
    auction_house,
    metadata,
    token_size,
    price,
    bump,
    created_at_on_chain,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
ON CONFLICT ON CONSTRAINT purchase_receipt_pkey DO UPDATE 
    SET
        bookkeeper = EXCLUDED.bookkeeper,
        buyer = EXCLUDED.buyer,
        seller = EXCLUDED.seller,
        auction_house = EXCLUDED.auction_house,
        metadata = EXCLUDED.metadata,
        token_size = EXCLUDED.token_size,
        price = EXCLUDED.price,
        bump = EXCLUDED.bump,
        created_at_on_chain = EXCLUDED.created_at_on_chain,
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > purchase_receipt.slot
        OR (
            EXCLUDED.slot = purchase_receipt.slot
            AND EXCLUDED.write_version > purchase_receipt.write_version
        )
RETURNING created_at = modified_at