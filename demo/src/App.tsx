import { Theme } from './Theme';
import { WalletContext } from './components/Wallet';
import DisconnectIcon from '@mui/icons-material/LinkOff';
import Toolbar from '@mui/material/Toolbar';
import Box from '@mui/material/Box';
import { WalletDisconnectButton, WalletMultiButton } from '@solana/wallet-adapter-material-ui';
import React, { FC, Fragment, useContext } from 'react';
import Container from '@mui/material/Container';
import Grid from '@mui/material/Grid';
import Typography from '@mui/material/Typography';
import { Context, Store } from './Store';
import { PromoCards } from './components/Promos';
const logo = require('./assets/logo.png');

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
    const { state } = useContext(Context);
    return (
        <Fragment>
            <Toolbar style={{ display: 'flex' }}>
                <Box sx={{ height: 40, width: 40, mr: 1 }} component="img" alt="bokoup logo" src={logo} />
                <Typography component="h1" variant="h6" style={{ flexGrow: 1 }}>
                    bokoup
                </Typography>
                <WalletMultiButton />
                {state.walletConnected && (
                    <WalletDisconnectButton startIcon={<DisconnectIcon />} style={{ marginLeft: 8 }} />
                )}
            </Toolbar>
            <Container maxWidth="xl">
                <Grid container spacing={4} justifyContent="center">
                    <PromoCards />
                </Grid>
            </Container>
        </Fragment>
    );
};
