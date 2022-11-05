import * as anchor from '@project-serum/anchor';
import { TokenMetadataProgram, AdminSettings, DataV2, PromoExtended, PromoGroup } from '../src';
import { PublicKey, Keypair, Transaction, Connection } from '@solana/web3.js';
import chai = require('chai');
import chaiAsPromised = require('chai-as-promised');
const fs = require('fs');
chai.use(chaiAsPromised);
const expect = chai.expect;
import * as dotenv from 'dotenv';
import path from 'path';
dotenv.config({ path: path.resolve(__dirname, '../../../demo-web/.env') });

describe('promo', () => {
  const tokenOwnerProvider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  // anchor.setProvider(provider);
  const tokenOwner = tokenOwnerProvider.wallet.publicKey;
  const tokenMetadataProgram = new TokenMetadataProgram(tokenOwnerProvider);

  const promoOwner = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync('/keys/promo_owner-keypair.json'))),
  );

  const process = require("process");
  const url = process.env.ANCHOR_PROVIDER_URL;
  if (url === undefined) {
    throw new Error("ANCHOR_PROVIDER_URL is not defined");
  }
  const options = anchor.AnchorProvider.defaultOptions();
  const connection = new Connection(url, options.commitment);
  const promoOwnerWallet = new anchor.Wallet(promoOwner)

  const promoOwnerProvider = new anchor.AnchorProvider(
    connection,
    promoOwnerWallet,
    options
  )

  const tokenMetadataProgramPromoOwner = new TokenMetadataProgram(promoOwnerProvider);

  const platform = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(process.env.REACT_APP_PLATFORM_KEYPAIR!)),
  );
  console.log('promoOwner: ', promoOwner.publicKey.toString());
  console.log('platform: ', platform.publicKey.toString());

  let adminSettings: PublicKey;
  let adminSettingsAccount: AdminSettings;
  let mint: PublicKey;
  let promoExtended: PromoExtended;

  let group: PublicKey;
  const groupSeed = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync('target/deploy/group_seed-keypair.json'))),
  ).publicKey;

  const groupMember1 = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync('target/deploy/group_member_1-keypair.json'))),
  );
  console.log("groupMember1", groupMember1.publicKey.toString())
  const plaformSigner = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync('target/deploy/platform_signer-keypair.json'))),
  );

  const tokenMetadataProgramGroupMember1 = new TokenMetadataProgram(new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(groupMember1),
    options
  ));

  let groupAccount: PromoGroup;



  it('funds accounts', async () => {
    const amount = 1_000_000_000;
    const transaction = new Transaction();
    const addresses = [platform.publicKey, promoOwner.publicKey, groupMember1.publicKey, plaformSigner.publicKey];
    addresses.forEach((address) => {
      transaction.add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: tokenMetadataProgram.payer.publicKey,
          lamports: amount,
          toPubkey: address,
        }),
      );
    });
    await tokenOwnerProvider.sendAndConfirm(transaction);
    const accountInfos = await Promise.all(
      addresses.map((address) => tokenOwnerProvider.connection.getAccountInfo(address)),
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

  it('creates group', async () => {

    const members = [promoOwner.publicKey, groupMember1.publicKey, plaformSigner.publicKey]
    const lamports = 500_000_000;
    const memo = "Created a new group for bokoup store group";

    [group] = await tokenMetadataProgramPromoOwner.createPromoGroup(groupSeed, members, lamports, memo);

    groupAccount = (await tokenMetadataProgram.program.account.promoGroup.fetch(
      group,
    )) as PromoGroup;
    expect(groupAccount.owner.toString()).to.equal(
      promoOwner.publicKey.toString(),
      'Group incorrect.',
    );
    console.log("accountOwner", promoOwner.publicKey.toString());

    console.log("groupAccount", groupAccount);
    console.log("groupSeed", groupSeed);

    const groupAccountInfo =
      await tokenMetadataProgram.program.provider.connection.getAccountInfo(
        group,
      );

    expect(groupAccountInfo?.lamports).to.equal(
      503626160,
      'Group lamports incorrect.',
    );

    console.log("groupAccountInfo", groupAccountInfo);
  });


  it('transfers cpi', async () => {
    tokenMetadataProgramPromoOwner.program.methods.transferCpi(1_000_000).accounts({
      group,
      platform: adminSettingsAccount.platform,
      adminSettings
    }).rpc()
  });


  it('Creates two promos', async () => {
    const metadataData1: DataV2 = {
      name: 'Test Promo',
      symbol: 'BTP',
      uri: 'https://arweave.net/frDiuZYzSVwYTwSUMR1YbggVkZqZfA7S9xsI3drPWBo',
      sellerFeeBasisPoints: 0,
      creators: null,
      collection: null,
      uses: null,
    };

    const metadataData2: DataV2 = {
      name: 'Test Promo 2',
      symbol: 'BTP2',
      uri: 'https://arweave.net/ZsjbP1xEBPGlS9ZLhMbR857KW9DxDwJbigl_4NSgz2E',
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

      const maxMint = 10;
      const maxRedeem = 5;

      const memo = {
        reference: metadataData.symbol,
        memo: "Created new promo"
      };

      mint = await tokenMetadataProgramPromoOwner.createPromo(
        metadataData,
        true,
        groupSeed,
        maxMint,
        maxRedeem,
        adminSettingsAccount.platform,
        JSON.stringify(memo)
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

  // This has group member1 pay for the transaction, which they are able to do because
  // of their membership in the group that owns the promo.
  it('Mints a promo token', async () => {
    const [tokenAccountAccount, mintAccount] = await tokenMetadataProgram
      .mintPromoToken(mint, groupMember1, groupSeed, "just a string for a memo")
      .then((tokenAccount) =>
        Promise.all([
          tokenMetadataProgram.getTokenAccount(tokenAccount),
          tokenMetadataProgram.getMintAccount(promoExtended.mintAccount.address),
        ]),
      );

    promoExtended = await tokenMetadataProgram.getPromoExtended(mint);

    expect(Number(tokenAccountAccount.amount)).to.equal(1, 'Token account amount incorrect.');
    expect(Number(mintAccount.supply)).to.equal(1, 'Mint supply incorrect.');
    expect(promoExtended.mintCount).to.equal(1, 'Promo mints incorrect.');

    console.log('tokenAccountAccount: ', tokenAccountAccount);
    console.log('mintAccount: ', mintAccount);
  });

  it('Delegates a promo token', async () => {
    // try different keys for memo
    const memo = {
      source: "spirit",
      platform: "shoes",
      location: "here"
    };

    const tokenAccountAccount = await tokenMetadataProgram
      .delegatePromoToken(mint, groupMember1.publicKey, groupSeed, JSON.stringify(memo))
      .then((tokenAccount) => tokenMetadataProgram.getTokenAccount(tokenAccount));
    expect(Number(tokenAccountAccount.delegatedAmount)).to.equal(1, 'Delegated amount incorrect.');
    console.log('tokenAccountAccount: ', tokenAccountAccount);
  });

  it('Burns a delegated promo token', async () => {
    const platformStartAccountInfo =
      await tokenMetadataProgram.program.provider.connection.getAccountInfo(
        adminSettingsAccount.platform,
      );

    console.log("mint", mint);
    const memo = {
      reference: "myReference",
      memo: "burned delegated token"
    };

    const tokenAccount = await tokenMetadataProgramGroupMember1
      .burnDelegatedPromoToken(mint, tokenOwner, platform.publicKey, groupSeed, JSON.stringify(memo))

    const mintAccount = await tokenMetadataProgram.getMintAccount(mint)

    // Fix indexer to delete account from db if it is closed.
    // await expect(tokenMetadataProgram.getTokenAccount(tokenAccount)).to.be.rejected

    promoExtended = await tokenMetadataProgram.getPromoExtended(mint);

    expect(Number(mintAccount.supply)).to.equal(0, 'Mint supply incorrect.');
    expect(promoExtended.burnCount).to.equal(1, 'Promo burns incorrect.');

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

});
