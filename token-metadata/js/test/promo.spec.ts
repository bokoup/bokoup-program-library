import * as anchor from '@project-serum/anchor';
import { TokenMetadataProgram } from '../src';
import { getMint, getAccount } from '@solana/spl-token';
import { Metadata } from '@metaplex-foundation/mpl-token-metadata';
import { Promo, DataV2 } from '../src';
import {
  Transaction,
  PublicKey,
  Keypair,
  AccountMeta,
  sendAndConfirmRawTransaction,
  SYSVAR_INSTRUCTIONS_PUBKEY,
} from '@solana/web3.js';
const BN = require('bn.js');
const chai = require('chai');
const chaiAsPromised = require('chai-as-promised');
chai.use(chaiAsPromised);
const expect = chai.expect;

describe('promo', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = TokenMetadataProgram.getProgram(anchor.AnchorProvider.env());
  const provider = program.provider as anchor.AnchorProvider;
  const payer = provider.wallet as anchor.Wallet;
  let platform = Keypair.generate();
  let adminSettings: PublicKey;
  const mint = Keypair.generate();
  let metadata = Keypair.generate().publicKey;
  let promo: PublicKey;
  let promoInfo: Promo;
  let metadataInfo: Metadata;

  const promoData: Promo = {
    owner: payer.publicKey,
    mint: mint.publicKey,
    metadata,
    maxMint: 1_000,
    maxRedeem: 500,
    expiry: new BN(((Date.now() + 1000 * 60 * 60 * 24 * 10).valueOf() / 1000).toFixed(0))
  };

  it('creates admin settings', async () => {
    [adminSettings] = await TokenMetadataProgram.findAdminAddress();

    await TokenMetadataProgram.getPlatformAddress(program, platform);

    const adminSettingsAccount = await program.account.adminSettings.fetch(adminSettings);
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

    promo = await TokenMetadataProgram.createPromo(program, promoData, metadataData, true, mint, platform.publicKey);

    promoInfo = await program.account.promo.fetch(promo) as Promo;
    console.log(new Date(promoInfo.expiry.toNumber() * 1000));

    metadataInfo = await Metadata.fromAccountAddress(program.provider.connection, promoInfo.metadata);
    console.log(metadataInfo);
  });

});
