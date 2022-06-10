import React, { FC, useContext, useEffect } from 'react';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Card from '@mui/material/Card';
import CardActions from '@mui/material/CardActions';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Divider from '@mui/material/Divider';
import Grid from '@mui/material/Grid';
import Switch from '@mui/material/Switch';
import { styled } from '@mui/material/styles';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { Keypair } from '@solana/web3.js';

import { Context, getTokenAccounts, PROMO1, PROMO2 } from '../Store';
import { Action, Product, Products, ShopTotal, State, ShopPromos, ShopPromo } from '../types/types';
import { PromoExtendeds } from '@bokoup/bpl-token-metadata';

import donut from '../assets/donut.png';
import infinity from '../assets/infinity.png';
import blob from '../assets/blob.png';

function createProduct(
    name: string,
    description: string,
    src: '*.png',
    price: number,
    quantity: number,
    total: number
): Product {
    return { name, description, src, price, quantity, total };
}

export const initialProducts: Products = {
    donut: createProduct('Donut', 'Super dope donut glass. So tasty.', donut, 0.42, 0, 0),
    infinity: createProduct('Infinity', 'This infinity glass goes on for ever. Yum.', infinity, 0.69, 0, 0),
    blob: createProduct('Blob', 'One of a kind blob glass. Best desert ever.', blob, 0.88, 0, 0),
};

export const initialShopTotal: ShopTotal = {
    quantity: 0,
    subtotal: 0,
    discount: 0,
    total: 0,
};

export function getShopTotal(state: State, dispatch: React.Dispatch<Action>) {
    const [subtotal, quantity] = Object.values(state.products).reduce(
        ([subtotal, quantity], product) => {
            subtotal += product.total;
            quantity += product.quantity;
            return [subtotal, quantity];
        },
        [0, 0]
    );

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
    const minPrice = Object.values(state.products).reduce((minPrice, product) => {
        minPrice = product.price < minPrice ? product.price : minPrice;
        return minPrice;
    }, 100);

    const shopPromos: ShopPromos = {};

    if (state.walletConnected) {
        const tokenAccount2 = state.tokenAccounts[PROMO2];
        if (tokenAccount2 && tokenAccount2.amount > 0 && state.shopTotal.quantity > 1) {
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
        if (tokenAccount1 && tokenAccount1.amount > 0 && state.shopTotal.subtotal - promo2Applied > 1) {
            const shopPromo: ShopPromo = {
                mint: tokenAccount1.mint,
                discount: (state.shopTotal.subtotal - promo2Applied) * 0.25,
                applied: 0,
            };

            if (state.shopPromos && state.shopPromos[PROMO1]) {
                shopPromo.applied = state.shopPromos[PROMO1].applied;
            }

            shopPromos[PROMO1] = shopPromo;
        }
        dispatch({ shopPromos });
    }
}

const StyledTableRow = styled(TableRow)(({ theme }) => ({
    td: {
        color: theme.palette.text.secondary,
    },
    th: {
        color: theme.palette.text,
        fontSize: 12,
    },
}));

const StyledTableRowPromoHeader = styled(TableRow)(({ theme }) => ({
    td: {
        color: theme.palette.text,
        fontSize: 12,
    },
}));

export const ProductRows: FC<{
    products: Products;
    subtotal: number;
    handleQuantityChange: (event: React.ChangeEvent<HTMLInputElement>) => void;
}> = ({ products, subtotal, handleQuantityChange }) => {
    return (
        <React.Fragment>
            {Object.values(products).map((row) => (
                <StyledTableRow key={row.name}>
                    <TableCell>
                        <Box sx={{ height: 80, width: 80, minWidth: 80 }} component="img" src={row.src} />
                    </TableCell>
                    <TableCell align="left">{row.description}</TableCell>
                    <TableCell align="right">{row.price}</TableCell>
                    <TableCell align="right">
                        <TextField
                            id={row.name.toLowerCase()}
                            value={row.quantity}
                            type="number"
                            onChange={handleQuantityChange}
                            sx={{ direction: 'rtl', width: '66%' }}
                        />
                    </TableCell>
                    <TableCell align="right">{row.total.toFixed(2)}</TableCell>
                </StyledTableRow>
            ))}

            <TableRow key={'subtotal'} sx={{ 'td, th': { border: 0 } }}>
                {Array(3)
                    .fill(0)
                    .map((_, i) => (
                        <TableCell key={i} />
                    ))}
                <TableCell align="right">Subtotal:</TableCell>
                <TableCell align="right">{subtotal.toFixed(2)}</TableCell>
            </TableRow>
        </React.Fragment>
    );
};

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
            <TableRow key={'fill1'}>
                {Array(5)
                    .fill(0)
                    .map((_, i) => (
                        <TableCell key={i} />
                    ))}
            </TableRow>
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
                            sx={{ height: 80, width: 80, minWidth: 80 }}
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
                    <TableCell align="right">{shopPromo.applied.toFixed(2)}</TableCell>
                </StyledTableRow>
            ))}
            <TableRow key={'discount'} sx={{ 'td, th': { border: 0 } }}>
                {Array(3)
                    .fill(0)
                    .map((_, i) => (
                        <TableCell key={i} />
                    ))}
                <TableCell align="right">Discount:</TableCell>
                <TableCell align="right">{shopTotal.discount.toFixed(2)}</TableCell>
            </TableRow>
        </React.Fragment>
    );
};

export const Shop: FC = () => {
    const { state, dispatch } = useContext(Context);
    // demo only - not secure
    const promoOwner = Keypair.fromSecretKey(
        new Uint8Array(JSON.parse(process.env.REACT_APP_PROMO_OWNER_KEYPAIR!)),
    );

    useEffect(() => {
        getShopPromos(state, dispatch);
    }, []);

    function handleQuantityChange(event: React.ChangeEvent<HTMLInputElement>) {
        const products = Object.assign({}, state.products);
        products[event.target.id].quantity = Math.max(+event.target.value, 0);
        products[event.target.id].total = products[event.target.id].quantity * products[event.target.id].price;
        dispatch({ products });
    }

    async function handleCheckout() {
        const mints = Object.values(state.shopPromos).map((shopPromo) => shopPromo.mint);
        if (mints) {
            const platform = await state.program.fetchPlatformAddress();
            await state.program.delegateAndBurnPromoTokens(platform, mints, promoOwner);
            const [promoExtendeds] = await Promise.all([
                state.program.updatePromoExtendeds(state.promoExtendeds),
                getTokenAccounts(state, dispatch),
            ]);
            dispatch({ promoExtendeds });
        }
        const products = Object.assign({}, state.products);
        Object.values(products).forEach((product) => {
            product.quantity = 0;
            product.total = 0;
        });
        dispatch({ products });
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
        <Grid item sx={{ flex: 1 }}>
            <Card raised>
                <CardHeader
                    title={'Shop'}
                    subheader={
                        <Typography color="text.secondary">
                            Connect wallet. Get some promos. Change quantities to see available promos. Checkout only
                            uses applied promos - no SOL transferred for product amounts.
                        </Typography>
                    }
                />
                <CardContent>
                    <Divider sx={{ pt: 1, mb: 1 }} />
                    <TableContainer>
                        <Table sx={{ minWidth: 420 }} aria-label="simple table">
                            <TableHead>
                                <StyledTableRow>
                                    <TableCell>PRODUCT</TableCell>
                                    <TableCell>DESCRIPTION</TableCell>
                                    <TableCell align="right">PRICE</TableCell>
                                    <TableCell align="right">QUANTITY</TableCell>
                                    <TableCell align="right">TOTAL</TableCell>
                                </StyledTableRow>
                            </TableHead>
                            <TableBody>
                                <ProductRows
                                    products={state.products}
                                    subtotal={state.shopTotal.subtotal}
                                    handleQuantityChange={handleQuantityChange}
                                />
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
                                            {state.shopTotal.total.toFixed(2)}
                                        </Typography>
                                    </TableCell>
                                </TableRow>
                            </TableBody>
                        </Table>
                    </TableContainer>
                </CardContent>
                <CardActions sx={{ justifyContent: 'center', mb: 2 }}>
                    <Button
                        variant="contained"
                        disabled={!state.walletConnected || state.shopTotal.total == 0}
                        color="primary"
                        onClick={handleCheckout}
                    >
                        CHECK OUT
                    </Button>
                </CardActions>
            </Card>
        </Grid>
    );
};
