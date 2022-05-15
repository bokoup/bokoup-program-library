import * as anchor from '@project-serum/anchor';
import { AdminSettings, TokenMetadata } from '../src';
import { Metadata } from '@metaplex-foundation/mpl-token-metadata';
import { Promo, DataV2 } from '../src';
import { PublicKey, Keypair } from '@solana/web3.js';
import chai = require('chai');
import chaiAsPromised = require('chai-as-promised');
chai.use(chaiAsPromised);
const expect = chai.expect;

describe('promo', () => {
  const provider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);
  const tokenMetadata = new TokenMetadata(provider);
  const promoOwner = Keypair.generate();
  const platform = Keypair.generate();
  let adminSettings: PublicKey;
  let adminSettingsAccount: AdminSettings;
  let promo: PublicKey;
  let promoAccount: Promo;
  let metadataAccount: Metadata;

  it('funds accounts', async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(platform.publicKey, 2_000_000_000),
      'confirmed',
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(promoOwner.publicKey, 2_000_000_000),
      'confirmed',
    );
  });

  it('creates admin settings', async () => {
    [adminSettings] = await tokenMetadata.findAdminAddress();

    await tokenMetadata.createAdminSettings(platform, 10_000_000, 1_000_000);

    adminSettingsAccount = (await tokenMetadata.program.account.adminSettings.fetch(
      adminSettings,
    )) as AdminSettings;
    expect(adminSettingsAccount.platform.toString()).to.equal(
      platform.publicKey.toString(),
      'Admin platform incorrect.',
    );
  });

  it('Creates a promo', async () => {
    const metadataData: DataV2 = {
      name: 'Promotion #0',
      symbol: '42',
      uri: 'https://bokoup.so',
      sellerFeeBasisPoints: 0,
      creators: null,
      collection: null,
      uses: null,
    };

    const platformStartAccountInfo = await tokenMetadata.program.provider.connection.getAccountInfo(
      adminSettingsAccount.platform,
    );

    const maxMint = 1_000;
    const maxRedeem = 1;
    const expiry = new Date(Date.now() + 1000 * 60 * 60 * 24 * 10);

    promo = await tokenMetadata.createPromo(
      adminSettingsAccount.platform,
      metadataData,
      true,
      maxMint,
      maxRedeem,
      expiry,
      promoOwner,
    );

    promoAccount = (await tokenMetadata.program.account.promo.fetch(promo)) as Promo;
    console.log(promoAccount);

    metadataAccount = await Metadata.fromAccountAddress(provider.connection, promoAccount.metadata);
    console.log(metadataAccount);

    const platformAccountInfo = await tokenMetadata.program.provider.connection.getAccountInfo(
      adminSettingsAccount.platform,
    );
    if (platformStartAccountInfo !== null && platformAccountInfo !== null) {
      expect(platformAccountInfo.lamports).to.equal(
        platformStartAccountInfo.lamports + adminSettingsAccount.createPromoLamports.toNumber(),
        'Platform lamports incorrect.',
      );
    }
  });

  it('Mints a promo token', async () => {
    const [tokenAccountAccount, mintAccount] = await tokenMetadata
      .mintPromoToken(promoAccount.mint, promoOwner)
      .then((tokenAccount) =>
        Promise.all([
          tokenMetadata.getTokenAccount(tokenAccount),
          tokenMetadata.getMint(promoAccount.mint),
        ]),
      );
    console.log('tokenAccountAccount: ', tokenAccountAccount);
    console.log('mintAccount: ', mintAccount);
    expect(Number(tokenAccountAccount.amount)).to.equal(1, 'Token account amount incorrect.');
    expect(Number(mintAccount.supply)).to.equal(1, 'Mint supply incorrect.');
  });
});
