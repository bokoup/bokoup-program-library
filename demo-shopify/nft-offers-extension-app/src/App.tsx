import { Theme } from './Theme';
import { WalletContext } from './components/Wallet';
import Toolbar from '@mui/material/Toolbar';
import { WalletMultiButton } from '@solana/wallet-adapter-material-ui';
import { FC, Fragment } from 'react';
import Container from '@mui/material/Container';
import { Store } from './Store';
import { Shop } from './components/Shop';

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
    return (
        <Fragment>
            <Toolbar disableGutters style={{ display: 'flex' }}>
                <WalletMultiButton sx={{ px: 12, fontSize: 15, letterSpacing: 1 }} />
            </Toolbar>
            <Container disableGutters maxWidth="xl">
                <Shop />
            </Container>
        </Fragment>
    );
};
