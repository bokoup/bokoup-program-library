import React, { FC, useContext } from 'react';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Card from '@mui/material/Card';
import CardActions from '@mui/material/CardActions';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Divider from '@mui/material/Divider';
import Grid from '@mui/material/Grid';
import Switch from '@mui/material/Switch';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { Context, } from '../Store';
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
    total: number,
): Product {
    return { name, description, src, price, quantity, total };
}

export const initialProducts: Products = {
    donut: createProduct('Donut', 'Super dope donut glass. So tasty.', donut, 0.42, 0, 0),
    infinity: createProduct('Infinity', 'This infinity glass goes on for ever. Yum.', infinity, 0.69, 0, 0),
    blob: createProduct('Blob', 'One of a kind blob glass. Best desert ever.', blob, 0.88, 0, 0),
}

export const initialShopTotal: ShopTotal = {
    quantity: 0,
    subtotal: 0,
    discount: 0,
    total: 0,
    shopPromos: {}
}


export function getShopTotal(state: State, dispatch: React.Dispatch<Action>) {
    let [subtotal, quantity, minPrice] = Object.values(state.products).reduce(
        ([subtotal, quantity, minPrice], product) => {
            subtotal += product.total;
            quantity += product.quantity;
            minPrice = product.price < minPrice ? product.price : minPrice;
            return [subtotal, quantity, minPrice]
        },
        [0, 0, 100]
    );

    let shopPromos: ShopPromos = {};
    let discount = 0;
    let total = subtotal;

    if (state.walletConnected) {
        let promo1Mint = 'YibmxF3rHAN1og5dNEAZ82dpSzcYeRkNECr3czZkyst';
        let tokenAccount1 = state.tokenAccounts[promo1Mint];
        if (tokenAccount1 && tokenAccount1.amount > 0 && quantity > 1) {
            let shopPromo: ShopPromo = {
                mint: tokenAccount1.mint,
                discount: minPrice,
                applied: 0
            };

            if (state.shopTotal.shopPromos && state.shopTotal.shopPromos[promo1Mint]) {
                shopPromo.applied = state.shopTotal.shopPromos[promo1Mint].applied;
            }

            shopPromos[promo1Mint] = shopPromo;
            discount = shopPromo.applied ? shopPromo.discount : discount;
            total = subtotal - discount
        };

        let promo2Mint = 'FjuhWzDDqG9aR5g95VKRUWndVYt7wVq78cptPrLMKTgA';
        let tokenAccount2 = state.tokenAccounts[promo1Mint];
        if (tokenAccount2 && tokenAccount2.amount > 0 && total > 1) {
            let shopPromo: ShopPromo = {
                mint: tokenAccount2.mint,
                discount: total * 0.25,
                applied: 0
            };

            if (state.shopTotal.shopPromos && state.shopTotal.shopPromos[promo2Mint]) {
                shopPromo.applied = state.shopTotal.shopPromos[promo2Mint].applied;
            }

            shopPromos[promo2Mint] = shopPromo;
            discount = shopPromo.applied ? shopPromo.discount : discount;
            total = subtotal - discount
        };
    };

    const shopTotal: ShopTotal = {
        quantity,
        subtotal,
        discount,
        total,
        shopPromos
    };
    dispatch({ shopTotal });

}

export const ProductRows: FC<{
    products: Products,
    subtotal: number,
    handleQuantityChange: (event: React.ChangeEvent<HTMLInputElement>) => void
}> = ({ products, subtotal, handleQuantityChange }) => {
    console.log("renderProductRows");
    return (
        <React.Fragment>
            {Object.values(products).map((row) => (
                <TableRow
                    key={row.name}
                >
                    <TableCell component="th" scope="row">
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
                        /></TableCell>
                    <TableCell align="right">{row.total.toFixed(2)}</TableCell>
                </TableRow>
            ))}

            <TableRow
                key={"subtotal"}
                sx={{ 'td, th': { border: 0 } }}
            >
                {Array(3).fill(0).map((_, i) => <TableCell key={i} />)}
                <TableCell align="right">Subtotal:</TableCell>
                <TableCell align="right">{subtotal.toFixed(2)}</TableCell>
            </TableRow>
        </React.Fragment>
    )
}

export const PromoRows: FC<{
    shopTotal: ShopTotal,
    promoExtendeds: PromoExtendeds,
    dispatch: React.Dispatch<Action>
}> = ({ shopTotal, promoExtendeds, dispatch }) => {
    console.log("renderPromoRows");

    function handleApplyChange(key: string, shopTotal: ShopTotal) {
        let shopTotalNew = Object.assign({}, shopTotal);
        if (shopTotalNew.shopPromos[key].applied > 0) {
            shopTotalNew.shopPromos[key].applied = 0;
            shopTotalNew.discount -= shopTotalNew.shopPromos[key].discount;
            shopTotalNew.total += shopTotalNew.shopPromos[key].discount;
        } else {
            shopTotalNew.shopPromos[key].applied = shopTotalNew.shopPromos[key].discount;
            shopTotalNew.discount += shopTotalNew.shopPromos[key].discount;
            shopTotalNew.total -= shopTotalNew.shopPromos[key].discount;
        };
        dispatch({ shopTotal: shopTotalNew })
    };

    return (
        <React.Fragment>
            <TableRow
                key={"fill1"}
            >
                {Array(5).fill(0).map((_, i) => <TableCell key={i} />)}
            </TableRow>
            <TableRow sx={{ 'td, th': { padding: 2 } }}>
                <TableCell>Promo</TableCell>
                <TableCell sx={{ colspan: 2 }} >Description</TableCell>
                <TableCell ></TableCell>
                <TableCell align="right">Apply</TableCell>
                <TableCell align="right">Discount</TableCell>
            </TableRow>
            {Object.entries(shopTotal.shopPromos).map(([key, shopPromo]) => (
                <TableRow
                    key={key}
                >
                    <TableCell component="th" scope="row">
                        <Box sx={{ height: 80, width: 80, minWidth: 80 }} component="img" src={promoExtendeds[key].metadataJson.image} />
                    </TableCell>
                    <TableCell colSpan={2} >{promoExtendeds[key].metadataJson.description}</TableCell>
                    <TableCell align="right">
                        <Switch
                            checked={shopPromo.applied > 0}
                            onChange={() => handleApplyChange(key, shopTotal)}
                            inputProps={{ 'aria-label': `${key}` }}
                        /></TableCell>
                    <TableCell align="right">{shopPromo.applied.toFixed(2)}</TableCell>
                </TableRow>
            ))}
            <TableRow
                key={"discount"}
                sx={{ 'td, th': { border: 0 } }}
            >
                {Array(3).fill(0).map((_, i) => <TableCell key={i} />)}
                <TableCell align="right">Discount:</TableCell>
                <TableCell align="right">{shopTotal.discount.toFixed(2)}</TableCell>
            </TableRow>
        </React.Fragment>
    )
}

export const Shop: FC = () => {
    console.log("renderShop");
    const { state, dispatch } = useContext(Context);

    function handleQuantityChange(event: React.ChangeEvent<HTMLInputElement>) {
        let products = Object.assign({}, state.products);
        products[event.target.id].quantity = Math.max(+event.target.value, 0);
        products[event.target.id].total = products[event.target.id].quantity * products[event.target.id].price;
        dispatch({ products })
    };

    const promoRows = state.walletConnected && Object.keys(state.shopTotal.shopPromos).length
        ? <PromoRows shopTotal={state.shopTotal} promoExtendeds={state.promoExtendeds} dispatch={dispatch} />
        : null;

    return (
        <Grid item sx={{ flex: 1 }}>
            <Card raised>
                <CardHeader title={'Shop'} />
                <CardContent>
                    <Divider sx={{ pt: 1, mb: 1 }} />
                    <TableContainer >
                        <Table sx={{ minWidth: 420 }} aria-label="simple table">
                            <TableHead>
                                <TableRow>
                                    <TableCell>Product</TableCell>
                                    <TableCell >Description</TableCell>
                                    <TableCell align="right">Price</TableCell>
                                    <TableCell align="right">Quantity</TableCell>
                                    <TableCell align="right">Total</TableCell>
                                </TableRow>
                            </TableHead>
                            <TableBody>
                                <ProductRows products={state.products} subtotal={state.shopTotal.subtotal} handleQuantityChange={handleQuantityChange} />
                                {promoRows}
                                <TableRow
                                    key={"total"}
                                    sx={{ 'td, th': { border: 0 } }}
                                >
                                    {Array(3).fill(0).map((_, i) => <TableCell key={i} />)}
                                    <TableCell align="right">
                                        <Typography sx={{ fontSize: 18 }}>
                                            Total:
                                        </Typography>
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
                    <Button variant="contained" disabled={!state.walletConnected || state.shopTotal.total == 0} color="primary" onClick={() => console.log("nada coolangatta")}>
                        CHECK OUT
                    </Button>
                </CardActions>
            </Card>
        </Grid>
    );
};



