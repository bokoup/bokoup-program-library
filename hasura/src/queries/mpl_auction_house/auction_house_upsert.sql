INSERT INTO auction_house (
    id,
    auction_house_fee_account,
    auction_house_treasury,
    treasury_withdrawal_destination,
    fee_withdrawal_destination,
    treasury_mint,
    authority,
    creator,
    bump,
    treasury_bump,
    fee_payer_bump,
    seller_fee_basis_points,
    requires_sign_off,
    can_change_sale_price,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
ON CONFLICT ON CONSTRAINT auction_house_pkey DO UPDATE 
    SET
        auction_house_fee_account = EXCLUDED.auction_house_fee_account,
        auction_house_treasury = EXCLUDED.auction_house_treasury,
        treasury_withdrawal_destination = EXCLUDED.treasury_withdrawal_destination,
        fee_withdrawal_destination = EXCLUDED.fee_withdrawal_destination,
        treasury_mint = EXCLUDED.treasury_mint,
        authority = EXCLUDED.authority,
        creator = EXCLUDED.creator,
        bump = EXCLUDED.bump,
        treasury_bump = EXCLUDED.treasury_bump,
        fee_payer_bump = EXCLUDED.fee_payer_bump,
        seller_fee_basis_points = EXCLUDED.seller_fee_basis_points,
        requires_sign_off = EXCLUDED.requires_sign_off,
        can_change_sale_price = EXCLUDED.can_change_sale_price,
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > auction_house.slot
        OR (
            EXCLUDED.slot = auction_house.slot
            AND EXCLUDED.write_version > auction_house.write_version
        )
RETURNING created_at = modified_at