import * as anchor from '@project-serum/anchor';
import { TokenMetadataProgram, AdminSettings, DataV2, PromoExtended } from '../src';
import { PublicKey, Keypair, Transaction } from '@solana/web3.js';
import chai = require('chai');
import chaiAsPromised = require('chai-as-promised');
chai.use(chaiAsPromised);
const expect = chai.expect;
import * as dotenv from 'dotenv';
import path from 'path';
dotenv.config({ path: path.resolve(__dirname, '../../../demo/.env') });

describe('promo', () => {
  const provider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);
  const tokenMetadataProgram = new TokenMetadataProgram(provider);
  const promoOwner = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(process.env.REACT_APP_PROMO_OWNER_KEYPAIR!)),
  );
  const platform = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(process.env.REACT_APP_PLATFORM_KEYPAIR!)),
  );
  console.log('promoOwner: ', promoOwner.publicKey.toString());
  console.log('platform: ', platform.publicKey.toString());

  let adminSettings: PublicKey;
  let adminSettingsAccount: AdminSettings;
  let mint: PublicKey;
  let promoExtended: PromoExtended;

  it('funds accounts', async () => {
    const amount = 1_000_000_000;
    const transaction = new Transaction();
    const addresses = [platform.publicKey, promoOwner.publicKey];
    addresses.forEach((address) => {
      transaction.add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: tokenMetadataProgram.payer.publicKey,
          lamports: amount,
          toPubkey: address,
        }),
      );
    });
    await provider.sendAndConfirm(transaction);
    const accountInfos = await Promise.all(
      addresses.map((address) => provider.connection.getAccountInfo(address)),
    );
    accountInfos.map((account) => {
      expect(account!.lamports).to.equal(amount, 'Platform lamports incorrect.');
    });
  });

  it('creates admin settings', async () => {
    [adminSettings] = await tokenMetadataProgram.findAdminAddress();

    await tokenMetadataProgram.createAdminSettings(platform, 10_000_000, 1_000_000);

    adminSettingsAccount = (await tokenMetadataProgram.program.account.adminSettings.fetch(
      adminSettings,
    )) as AdminSettings;
    expect(adminSettingsAccount.platform.toString()).to.equal(
      platform.publicKey.toString(),
      'Admin platform incorrect.',
    );
  });

  it('Creates two promos', async () => {
    const metadataData1: DataV2 = {
      name: 'Promo 1',
      symbol: 'P1',
      uri: 'https://arweave.net/u27CJpMzXZnmrTwqXzHjXQnECxP0_iMzSjE-WMAec24',
      sellerFeeBasisPoints: 0,
      creators: null,
      collection: null,
      uses: null,
    };

    const metadataData2: DataV2 = {
      name: 'Promo 2',
      symbol: 'P2',
      uri: 'https://arweave.net/RK8RPpNWs1nVy0qqvKVAFcVQOxOltGMyDrJtt6SktTc',
      sellerFeeBasisPoints: 0,
      creators: null,
      collection: null,
      uses: null,
    };

    for (const metadataData of [metadataData1, metadataData2] as DataV2[]) {
      const platformStartAccountInfo =
        await tokenMetadataProgram.program.provider.connection.getAccountInfo(
          adminSettingsAccount.platform,
        );

      const maxMint = 1_000;
      const maxRedeem = 500;
      const expiry = new Date(Date.now() + 1000 * 60 * 60 * 24 * 10);

      mint = await tokenMetadataProgram.createPromo(
        adminSettingsAccount.platform,
        metadataData,
        true,
        maxMint,
        maxRedeem,
        expiry,
        promoOwner,
      );

      promoExtended = await tokenMetadataProgram.getPromoExtended(mint);
      console.log('promoExtended: ', promoExtended);
      console.log('mintAddress: ', promoExtended.mintAccount.address.toString());

      const platformAccountInfo =
        await tokenMetadataProgram.program.provider.connection.getAccountInfo(
          adminSettingsAccount.platform,
        );
      if (platformStartAccountInfo !== null && platformAccountInfo !== null) {
        expect(platformAccountInfo.lamports).to.equal(
          platformStartAccountInfo.lamports + adminSettingsAccount.createPromoLamports.toNumber(),
          'Platform lamports incorrect.',
        );
      }
    }
  });

  it('Mints a promo token', async () => {
    const [tokenAccountAccount, mintAccount] = await tokenMetadataProgram
      .mintPromoToken(
        mint,
        // promoOwner
      )
      .then((tokenAccount) =>
        Promise.all([
          tokenMetadataProgram.getTokenAccount(tokenAccount),
          tokenMetadataProgram.getMintAccount(promoExtended.mintAccount.address),
        ]),
      );

    promoExtended = await tokenMetadataProgram.getPromoExtended(mint);

    expect(Number(tokenAccountAccount.amount)).to.equal(1, 'Token account amount incorrect.');
    expect(Number(mintAccount.supply)).to.equal(1, 'Mint supply incorrect.');
    expect(promoExtended.mints).to.equal(1, 'Promo mints incorrect.');

    console.log('tokenAccountAccount: ', tokenAccountAccount);
    console.log('mintAccount: ', mintAccount);
  });

  it('Delegates a promo token', async () => {
    const tokenAccountAccount = await tokenMetadataProgram
      .delegatePromoToken(mint)
      .then((tokenAccount) => tokenMetadataProgram.getTokenAccount(tokenAccount));
    expect(Number(tokenAccountAccount.delegatedAmount)).to.equal(1, 'Delegated amount incorrect.');
  });

  it('Burns a promo token', async () => {
    const platformStartAccountInfo =
      await tokenMetadataProgram.program.provider.connection.getAccountInfo(
        adminSettingsAccount.platform,
      );

    const [tokenAccountAccount, mintAccount] = await tokenMetadataProgram
      .burnPromoToken(
        platform.publicKey,
        mint,
        // promoOwner
      )
      .then((tokenAccount) =>
        Promise.all([
          tokenMetadataProgram.getTokenAccount(tokenAccount),
          tokenMetadataProgram.getMintAccount(mint),
        ]),
      );

    promoExtended = await tokenMetadataProgram.getPromoExtended(mint);
    expect(Number(tokenAccountAccount.amount)).to.equal(0, 'Token account amount incorrect.');
    expect(Number(mintAccount.supply)).to.equal(0, 'Mint supply incorrect.');
    expect(promoExtended.burns).to.equal(1, 'Promo burns incorrect.');

    const platformAccountInfo =
      await tokenMetadataProgram.program.provider.connection.getAccountInfo(
        adminSettingsAccount.platform,
      );
    if (platformStartAccountInfo !== null && platformAccountInfo !== null) {
      expect(platformAccountInfo.lamports).to.equal(
        platformStartAccountInfo.lamports + adminSettingsAccount.burnPromoTokenLamports.toNumber(),
        'Platform lamports incorrect.',
      );
    }
  });

  it('Delegates and burns a promo token', async () => {
    await tokenMetadataProgram.mintPromoToken(
      mint,
      // promoOwner
    );

    await tokenMetadataProgram.delegateAndBurnPromoTokens(
      platform.publicKey,
      [mint],
      // promoOwner
    );

    promoExtended = await tokenMetadataProgram.getPromoExtended(mint);
    expect(promoExtended.burns).to.equal(2, 'Promo burns incorrect.');
  });
});
