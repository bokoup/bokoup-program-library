import React, { FC, useContext, Fragment } from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import { styled } from '@mui/material/styles';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableRow from '@mui/material/TableRow';
import Typography from '@mui/material/Typography';

import { Keypair } from '@solana/web3.js';

import { Context, getTokenAccounts, PROMO1, PROMO2 } from '../Store';
import { Action, ShopTotal, State, ShopPromos, ShopPromo } from '../types/types';
import { PromoExtendeds } from '@bokoup/bpl-token-metadata';

export const initialShopTotal: ShopTotal = {
    quantity: 0,
    subtotal: 0,
    discount: 0,
    total: 0,
};

export function getShopTotal(state: State, dispatch: React.Dispatch<Action>) {
    const quantity = state.cart.item_count ? state.cart.item_count : 0;
    const subtotal = state.cart.total_price ? state.cart.total_price : 0;

    const discount = Object.values(state.shopPromos).reduce((discount, shopPromo) => {
        discount += shopPromo.applied;
        return discount;
    }, 0);
    const shopTotal: ShopTotal = {
        quantity,
        subtotal,
        discount,
        total: subtotal - discount,
    };
    dispatch({ shopTotal });
}

export function getShopPromos(state: State, dispatch: React.Dispatch<Action>) {
    const minPrice = Object.keys(state.cart).length === 0 ? 0 : Object.values(state.cart.items).reduce((minPrice, product) => {
        minPrice = product.final_price < minPrice ? product.final_price : minPrice;
        return minPrice;
    }, 1000);

    const shopPromos: ShopPromos = {};

    if (state.walletConnected && Object.keys(state.cart).length > 0) {
        const tokenAccount2 = state.tokenAccounts[PROMO2];
        if (tokenAccount2 && tokenAccount2.amount > 0 && state.cart.item_count > 1) {
            const shopPromo: ShopPromo = {
                mint: tokenAccount2.mint,
                discount: minPrice,
                applied: 0,
            };

            if (state.shopPromos && state.shopPromos[PROMO2]) {
                shopPromo.applied = state.shopPromos[PROMO2].applied;
            }
            shopPromos[PROMO2] = shopPromo;
        }

        const tokenAccount1 = state.tokenAccounts[PROMO1];
        const promo2Applied = shopPromos && shopPromos[PROMO2] ? shopPromos[PROMO2].applied : 0;
        if (tokenAccount1 && tokenAccount1.amount > 0 && state.cart.items_subtotal_price - promo2Applied > 100) {
            const shopPromo: ShopPromo = {
                mint: tokenAccount1.mint,
                discount: (state.cart.items_subtotal_price - promo2Applied) * 0.25,
                applied: 0,
            };

            if (state.shopPromos && state.shopPromos[PROMO1] && state.shopPromos[PROMO1].applied) {
                shopPromo.applied = shopPromo.discount;
            }

            shopPromos[PROMO1] = shopPromo;
        }
        dispatch({ shopPromos });
    }
}

const StyledTableRow = styled(TableRow)(({ theme }) => ({
    td: {
        color: theme.palette.text,
        fontSize: 14,
    },
    th: {
        color: theme.palette.text,
        fontSize: 14,
    },
}));

const StyledTableRowPromoHeader = styled(TableRow)(({ theme }) => ({
    td: {
        color: theme.palette.text.secondary,
        fontSize: 10,
    },
}));

export const PromoRows: FC<{
    shopPromos: ShopPromos;
    shopTotal: ShopTotal;
    promoExtendeds: PromoExtendeds;
    dispatch: React.Dispatch<Action>;
}> = ({ shopPromos, shopTotal, promoExtendeds, dispatch }) => {
    function handleApplyChange(key: string, promos: ShopPromos) {
        const shopPromosNew = Object.assign({}, promos);
        if (promos[key].applied > 0) {
            shopPromosNew[key].applied = 0;
        } else {
            shopPromosNew[key].applied = promos[key].discount;
        }

        dispatch({ shopPromos: shopPromosNew });
    }

    return (
        <React.Fragment>
            <StyledTableRowPromoHeader sx={{ 'td, th': { padding: 2 } }}>
                <TableCell>PROMO</TableCell>
                <TableCell sx={{ colspan: 2 }}>DESCRIPTION</TableCell>
                <TableCell></TableCell>
                <TableCell align="right">APPLY</TableCell>
                <TableCell align="right">DISCOUNT</TableCell>
            </StyledTableRowPromoHeader>
            {Object.entries(shopPromos).map(([key, shopPromo]) => (
                <StyledTableRow key={key}>
                    <TableCell>
                        <Box
                            sx={{ height: 108, width: 108, minWidth: 108 }}
                            component="img"
                            src={promoExtendeds[key].metadataJson.image}
                        />
                    </TableCell>
                    <TableCell colSpan={2}>{promoExtendeds[key].metadataJson.description}</TableCell>
                    <TableCell align="right">
                        <Switch
                            checked={shopPromo.applied > 0}
                            onChange={() => handleApplyChange(key, shopPromos)}
                            inputProps={{ 'aria-label': `${key}` }}
                        />
                    </TableCell>
                    <TableCell align="right">{(shopPromo.applied / 100).toFixed(2)}</TableCell>
                </StyledTableRow>
            ))}
            <StyledTableRow key={'discount'} sx={{ 'td, th': { border: 0 } }}>
                {Array(3)
                    .fill(0)
                    .map((_, i) => (
                        <TableCell key={i} />
                    ))}
                <TableCell align="right">Discount:</TableCell>
                <TableCell align="right">{(shopTotal.discount / 100).toFixed(2)}</TableCell>
            </StyledTableRow>
        </React.Fragment>
    );
};

export const Shop: FC = () => {
    const { state, dispatch } = useContext(Context);
    // demo only - not secure
    const promoOwner = Keypair.fromSecretKey(
        new Uint8Array(JSON.parse(process.env.REACT_APP_PROMO_OWNER_KEYPAIR!)),
    );

    async function handleCheckout() {
        const mints = Object.values(state.shopPromos)
            .filter(shopPromo => shopPromo.applied)
            .map((shopPromo) => shopPromo.mint);
        let checkoutUrl = '/cart/checkout';
        if (mints.length > 0) {
            const platform = await state.program.fetchPlatformAddress();
            await state.program.delegateAndBurnPromoTokens(platform, mints, promoOwner);
            const [promoExtendeds] = await Promise.all([
                state.program.updatePromoExtendeds(state.promoExtendeds),
                getTokenAccounts(state, dispatch),
            ]);
            dispatch({ promoExtendeds });

            let searchParams = new URLSearchParams();
            mints.forEach(mint => searchParams.append('discount', mint.toString()));
            checkoutUrl = `${checkoutUrl}?${searchParams.toString()}`;
        }
        console.log(checkoutUrl);
        location.href = checkoutUrl;
    }

    const promoRows =
        state.walletConnected && Object.keys(state.shopPromos).length ? (
            <PromoRows
                shopPromos={state.shopPromos}
                shopTotal={state.shopTotal}
                promoExtendeds={state.promoExtendeds}
                dispatch={dispatch}
            />
        ) : null;

    return (
        <Fragment>
            <TableContainer>
                <Table sx={{ minWidth: 420 }} aria-label="simple table">
                    <TableBody>
                        {promoRows}
                        <TableRow key={'total'} sx={{ 'td, th': { border: 0 } }}>
                            {Array(3)
                                .fill(0)
                                .map((_, i) => (
                                    <TableCell key={i} />
                                ))}
                            <TableCell align="right">
                                <Typography sx={{ fontSize: 18 }}>Total:</Typography>
                            </TableCell>
                            <TableCell align="right">
                                <Typography sx={{ fontSize: 18 }}>
                                    {(state.shopTotal.total / 100).toFixed(2)}
                                </Typography>
                            </TableCell>
                        </TableRow>
                    </TableBody>
                </Table>
            </TableContainer>
            <Box style={{ display: 'flex', flexDirection: 'row-reverse' }}>
                <Button sx={{ px: 18, fontSize: 15, letterSpacing: 1 }}
                    variant="contained"
                    disabled={Object.keys(state.cart).length === 0}
                    color="primary"
                    onClick={handleCheckout}
                >
                    Checkout
                </Button>
            </Box>
        </Fragment>
    );
};
