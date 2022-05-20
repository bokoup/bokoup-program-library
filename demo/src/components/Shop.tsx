import React, { FC, useContext } from 'react';

import Avatar from '@mui/material/Avatar'
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Card from '@mui/material/Card';
import CardActions from '@mui/material/CardActions';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Divider from '@mui/material/Divider';
import Grid from '@mui/material/Grid';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import TextField from '@mui/material/TextField';
import { Context } from '../Store';
import { Product, Products } from '../types/types';
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

export const Shop: FC = () => {
    const { state, dispatch } = useContext(Context);

    function handleChange(event: React.ChangeEvent<HTMLInputElement>) {
        let products = Object.assign({}, state.products);
        products[event.target.id].quantity = Math.max(+event.target.value, 0);
        products[event.target.id].total = products[event.target.id].quantity * products[event.target.id].price;
        dispatch({ products })
    };

    function getTotal(products: Products): string {
        return Object.values(products).reduce(
            (total, product) => ((total += product.total), total),
            0
        ).toFixed(2);
    }

    return (
        <Grid item sx={{ flex: 1 }}>
            <Card raised>
                <CardHeader title={'Shop'} />
                <CardContent>
                    <Divider sx={{ pt: 1, mb: 1 }} />
                    <TableContainer >
                        <Table sx={{ minWidth: 650 }} aria-label="simple table">
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
                                {Object.values(state.products).map((row) => (
                                    <TableRow
                                        key={row.name}
                                        sx={{ '&:last-child td, &:last-child th': { border: 0 } }}
                                    >
                                        <TableCell component="th" scope="row">
                                            <Box sx={{ height: 80, width: 80, mr: 1 }} component="img" src={row.src} />
                                        </TableCell>
                                        <TableCell align="left">{row.description}</TableCell>
                                        <TableCell align="right">{row.price}</TableCell>
                                        <TableCell align="right">
                                            <TextField
                                                id={row.name.toLowerCase()}
                                                value={row.quantity}
                                                type="number"
                                                onChange={handleChange}
                                                sx={{ direction: 'rtl', width: '66%' }}
                                            /></TableCell>
                                        <TableCell align="right">{row.total.toFixed(2)}</TableCell>
                                    </TableRow>
                                ))}
                                <TableRow
                                    key={"total"}
                                    sx={{ '&:last-child td, &:last-child th': { border: 0 } }}
                                >
                                    {Array(3).fill(0).map((_, i) => <TableCell key={i} />)}
                                    <TableCell align="right">Subtotal:</TableCell>
                                    <TableCell align="right">{getTotal(state.products)}</TableCell>
                                </TableRow>
                            </TableBody>
                        </Table>
                    </TableContainer>
                </CardContent>
                <CardActions sx={{ justifyContent: 'center', mb: 2 }}>
                    <Button variant="contained" disabled={!state.walletConnected} color="primary" onClick={() => console.log("nada coolangatta")}>
                        APPLY PROMOS
                    </Button>
                </CardActions>
            </Card>
        </Grid>
    );
};



{/* <Grid container spacing={2}>
                        <Grid item lg={8}>
                            <TextField
                                required
                                id="amount"
                                label="Purchase Amount in $SOL"
                                type="number"
                                fullWidth
                                autoComplete="cc-name"
                                variant="standard"
                            />
                        </Grid>
                        <Grid item lg={4}>
                            <Button
                                variant="contained"
                                color="primary"
                                disabled={!state.walletConnected}
                                onClick={() => console.log('TODO: check for promos')}
                            >
                                CHECK FOR PROMOS
                            </Button>
                        </Grid>
                    </Grid> */}