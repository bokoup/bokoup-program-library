
import { Theme } from './Theme';
import { WalletContext } from './components/Wallet';
import DisconnectIcon from '@mui/icons-material/LinkOff';
import Toolbar from '@mui/material/Toolbar';
import Button from '@mui/material/Button';
import { WalletDisconnectButton, WalletMultiButton } from '@solana/wallet-adapter-material-ui';
import React, { FC, Fragment, useContext, useEffect } from 'react';
import Card from '@mui/material/Card';
import CardActions from '@mui/material/CardActions';
import CardContent from '@mui/material/CardContent';
import Container from '@mui/material/Container';
import Grid from '@mui/material/Grid';
import Typography from '@mui/material/Typography';
import { Context, Store } from './Store';
import { PromoCards } from './components/Promos'

export const App: FC = () => {
    return (
        <Theme>
            <WalletContext>
                <Store>
                    <Content />
                </Store>
            </WalletContext>
        </Theme>
    );
};

export const Content: FC = () => {
    const { state, dispatch } = useContext(Context);
    return (
        <Fragment>
            <Toolbar style={{ display: 'flex' }}>
                <Typography component="h1" variant="h6" style={{ flexGrow: 1 }}>
                    bokoup
                </Typography>
                <WalletMultiButton />
                {state.walletConnected && <WalletDisconnectButton startIcon={<DisconnectIcon />} style={{ marginLeft: 8 }} />}
            </Toolbar>
            <Container maxWidth="xl">
                <Grid container spacing={4} justifyContent="center">
                    <PromoCards />
                </Grid>
            </Container>

        </Fragment >
    );
};
