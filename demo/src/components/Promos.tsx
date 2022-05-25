import { FC, useContext, Fragment } from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Card from '@mui/material/Card';
import CardActions from '@mui/material/CardActions';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import CardMedia from '@mui/material/CardMedia';
import Chip from '@mui/material/Chip';
import Divider from '@mui/material/Divider';
import Grid from '@mui/material/Grid';
import Link from '@mui/material/Link';
import Typography from '@mui/material/Typography';
import { Context, getTokenAccount } from '../Store';
import { Attribute } from '@bokoup/bpl-token-metadata';
import { Keypair } from '@solana/web3.js';
import { encodeURL } from '@solana/pay';
import { QRCodeSVG } from 'qrcode.react';
import { debug } from 'console';

export const PromoCard: FC<{ mintString: string }> = ({ mintString }) => {
    const { state, dispatch } = useContext(Context);

    function getExplorerLink(address: string): string {
        const baseUrl = 'https://explorer.solana.com/address';
        let link;
        switch (state.network) {
            case 'http://127.0.0.1:8899':
                link = `${baseUrl}/${address}?cluster=custom&customUrl=${encodeURIComponent(state.network)}`;
                break;
            case 'https://api.devnet.solana.com':
                link = `${baseUrl}/${address}?cluster=devnet`;
                break;
            default:
                link = `${baseUrl}/${address}?cluster=devnet`;
        }
        return link;
    }

    const promoExtended = state.promoExtendeds[mintString];
    const tokenAccount = state.tokenAccounts[mintString];

    const stats: Attribute[] = [
        { traitType: 'supply', value: Number(promoExtended.mintAccount.supply) },
        { traitType: 'minted', value: promoExtended.mints },
        { traitType: 'burned', value: promoExtended.burns },
        { traitType: 'maxMint', value: promoExtended.maxMint! },
        { traitType: 'maxBurn', value: promoExtended.maxBurn! },
    ];

    const myStats: Attribute[] = [{ traitType: 'owned', value: tokenAccount ? Number(tokenAccount.amount) : 0 }];

    // demo only - not secure
    const promoOwner = Keypair.fromSecretKey(
        new Uint8Array(JSON.parse(process.env.REACT_APP_PROMO_OWNER_KEYPAIR!)),
    );

    async function handleClick() {
        await state.program
            .mintPromoToken(promoExtended.mintAccount.address, promoOwner)
            .then(() => Promise.all([getTokenAccount(state, dispatch, promoExtended.mintAccount.address)]));
    }

    const url = encodeURL({
        link: new URL(
            `promo/${promoExtended.mintAccount.address.toString()}/${promoExtended.metadataJson.name}`,
            process.env.REACT_APP_API_URL!
        ),
    });

    return (
        <Grid item sm={6}>
            <Card raised>
                <CardHeader
                    title={promoExtended.metadataAccount.data.name}
                    subheader={
                        <Link
                            sx={{ fontSize: 12 }}
                            underline="hover"
                            color="primary.light"
                            href={getExplorerLink(mintString)}
                            target="_blank"
                        >
                            {mintString.slice(0, 16)}
                        </Link>
                    }
                />
                <CardMedia component="img" src={promoExtended.metadataJson.image} />
                <CardContent>
                    <Typography sx={{ fontSize: 12, pb: 1 }} component="div">
                        DETAILS
                    </Typography>

                    <Typography sx={{ fontSize: 14 }} color="text.secondary" component="div">
                        {promoExtended.metadataJson.description}
                    </Typography>
                    <Divider sx={{ pt: 1, mb: 1 }} />
                    <Typography sx={{ fontSize: 12 }} component="div">
                        FEATURES
                    </Typography>
                    <Attributes attributes={promoExtended.metadataJson.attributes as Attribute[]} />
                    <Divider sx={{ pt: 1, mb: 1 }} />
                    <Stats stats={stats} title={'STATS'} />
                    {state.walletConnected ? <Stats stats={myStats} title="MY PROMOS" /> : null}
                </CardContent>
                <CardActions sx={{ justifyContent: 'space-evenly', mb: 2 }}>
                    <Button variant="contained" disabled={!state.walletConnected} color="primary" onClick={handleClick}>
                        GET PROMO
                    </Button>
                    <Box sx={{ pt: 1, px: 1, backgroundColor: 'white' }}>
                        <QRCodeSVG
                            value={url.toString()}
                            size={100}
                            bgColor="transparent"
                            imageSettings={{
                                src: 'https://solana.com/favicon.ico',
                                x: undefined,
                                y: undefined,
                                height: 20,
                                width: 20,
                                excavate: true,
                            }}
                        />
                    </Box>
                </CardActions>
            </Card>
        </Grid>
    );
};

export const PromoCards: FC = () => {
    const { state } = useContext(Context);
    const promoCards = Object.entries(state.promoExtendeds).map(([mintString], i) => (
        <PromoCard key={i} mintString={mintString} />
    ));

    return (
        <Grid item container spacing={4}>
            {promoCards}
        </Grid>
    );
};

export const Stats: FC<{ stats: Attribute[]; title: string }> = ({ stats, title }) => {
    return (
        <Fragment>
            <Typography sx={{ fontSize: 12 }} component="div">
                {title}
            </Typography>
            <Attributes attributes={stats} />
            <Divider sx={{ pt: 1, mb: 1 }} />
        </Fragment>
    );
};

export const Attributes: FC<{ attributes: Attribute[] }> = ({ attributes }) => {
    return (
        <Box>
            {attributes.map((a, i) => (
                <Chip key={i} sx={{ m: 0.5 }} label={`${a.traitType}: ${a.value}`} />
            ))}
        </Box>
    );
};
