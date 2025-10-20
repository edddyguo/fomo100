import * as anchor from "@project-serum/anchor";
import { Program, toInstruction } from "@project-serum/anchor";
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";
import { EscrowMarketplace } from "../target/types/fomo100";
import {
  PublicKey,
  SystemProgram,
  Transaction,
  Connection,
  Commitment,
  sendAndConfirmRawTransaction,
  Keypair,
} from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import bs58 from "bs58";

import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  getAccount,
  mintTo,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { assert } from "chai";

describe("escrow_marketplace", () => {
  getAssociatedTokenAddress;
  const commitment: Commitment = "processed";
  const connection = new Connection("https://api.devnet.solana.com", {
    commitment,
    wsEndpoint: "wss://api.devnet.solana.com/",
  });
  const options = anchor.Provider.defaultOptions();
  const wallet = NodeWallet.local();
  const provider = new anchor.Provider(connection, wallet, options);

  anchor.setProvider(provider);

  const program = anchor.workspace
    .EscrowMarketplace as Program<EscrowMarketplace>;

  //mint A为coin
  //mint B为nft
  let mint_coin = null;
  let mint_token = null;
  let seller_coin_account = null;
  let seller_token_account = null;
  let buyer_coin_account = null;
  let buyer_token_account = null;
  let vault_account_pda = null;
  let vault_account_bump = null;
  let vault_authority_pda = null;

  const buyer_coin_amount = 1000;
  const nft_amount = 1;

  const escrowAccount = anchor.web3.Keypair.generate();
  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();
  const seller = anchor.web3.Keypair.generate();
  const buyer = anchor.web3.Keypair.generate();

  it("Initialize program state", async () => {
    // Airdropping tokens to a payer.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 2000000000),
      "processed"
    );

    // Fund Main Accounts
    await provider.send(
      (() => {
        const tx = new Transaction();
        tx.add(
          SystemProgram.transfer({
            fromPubkey: payer.publicKey,
            toPubkey: seller.publicKey,
            lamports: 200000000,
          }),
          SystemProgram.transfer({
            fromPubkey: payer.publicKey,
            toPubkey: buyer.publicKey,
            lamports: 100000000,
          })
        );
        return tx;
      })(),
      [payer]
    );
    //创建nft
    mint_coin = await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      mintAuthority.publicKey,
      0
    );
    //创建代币（k币）
    mint_token = await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      mintAuthority.publicKey,
      0
    );

    //创建卖家在k币上的ata账户
    seller_coin_account = await createAccount(
      provider.connection,
      payer,
      mint_coin,
      seller.publicKey
    ); //await mint_coin.createAccount(seller.publicKey);
    //创建买家在k币上的ata账户
    buyer_coin_account = await createAccount(
      provider.connection,
      payer,
      mint_coin,
      buyer.publicKey
    ); //await mint_coin.createAccount(buyer.publicKey);
    //创建卖家在该NFT上的ata账户
    seller_token_account = await createAccount(
      provider.connection,
      payer,
      mint_token,
      seller.publicKey
    ); //await mint_token.createAccount(seller.publicKey)
    //创建买家在该NFT上的ata账户
    buyer_token_account = await createAccount(
      provider.connection,
      payer,
      mint_token,
      buyer.publicKey
    ); //await mint_token.createAccount(buyer.publicKey);

    //初始化创建NFT并给到卖家的nft的ata账户
    await mintTo(
      provider.connection,
      payer,
      mint_token,
      seller_token_account,
      payer.publicKey,
      nft_amount
    );
    //初始化铸币1000个给买家的k币的ata账户
    await mintTo(
      provider.connection,
      payer,
      mint_coin,
      buyer_coin_account,
      payer.publicKey,
      buyer_coin_amount
    );

    let _initializerTokenAccountA = await getAccount(
      provider.connection,
      seller_token_account
    );
    let _takerTokenAccountB = await getAccount(
      provider.connection,
      buyer_coin_account
    );

    assert.ok(
      (Number(_initializerTokenAccountA.amount) as number) == nft_amount
    );
    assert.ok(Number(_takerTokenAccountB.amount) == buyer_coin_amount);
    getAssociatedTokenAddress(
      new PublicKey("7AMLNT5QjsmoTf8mVGTXb4pvfnS8dWxhC3rxfNpbubJ9"),
      new PublicKey("7AMLNT5QjsmoTf8mVGTXb4pvfnS8dWxhC3rxfNpbubJ9")
    );
  });

  //测试挂单接口
  it("sell nft", async () => {
    mint_token = new PublicKey("BmvDXZQzFcDr7MuLJDVUkvfBj3Gua7iyjXoVLqiaMWCJ");
    //let seller_key = new PublicKey("7AMLNT5QjsmoTf8mVGTXb4pvfnS8dWxhC3rxfNpbubJ9");
    let yanxiang_signer = Keypair.fromSecretKey(
      bs58.decode(
        "4V1Rwvj9N55JMEYF5GVAEbkkBc7MBjgbREVBWMuK1qPWUYKNmmBzhCngLSyfciZEEMFA2xSk7Go65fDZe1ujsWJm"
      )
    );
    const [_vault_account_pda, _vault_account_bump] =
      await PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("escrow_vault")),
          mint_token.toBuffer(),
        ],
        program.programId
      );
    vault_account_pda = _vault_account_pda;
    vault_account_bump = _vault_account_bump;

    const [_vault_authority_pda, _vault_authority_bump] =
      await PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("escrow_owner")),
          mint_token.toBuffer(),
        ],
        program.programId
      );

    const [escrow_info_pda, _test3] = await PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("escrow_info")),
        mint_token.toBuffer(),
      ],
      program.programId
    );

    vault_authority_pda = _vault_authority_pda;
    let settings_key = new PublicKey(
      "2m3fbrWQPuppKRSSa4UCoc5RtpMXZqwZzFkrxeVK8eA8"
    );
    let seller_token_account = new PublicKey(
      "7iMr5iEVSEcXPbAoqjytme4kuVj7NMTDnfDcoq7unKxL"
    );

    console.log(
      "",
      seller.publicKey.toBase58(),
      mint_token.toBase58(),
      vault_account_pda.toBase58(),
      seller_token_account.toBase58(),
      escrow_info_pda.toBase58(),
      settings_key.toBase58(),
      vault_authority_pda.toBase58()
    );
    let ins1 = program.instruction.sell(
      vault_authority_pda,
      null,
      new anchor.BN(buyer_coin_amount),
      {
        accounts: {
          seller: yanxiang_signer.publicKey, //seller.publicKey,
          nftMint: mint_token,
          vaultAccount: vault_account_pda,
          sellerTokenAccount: seller_token_account,
          escrowAccount: escrow_info_pda,
          settingAccount: settings_key,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        //signers: [seller],
      }
    );

    let transaction = new Transaction().add(ins1);
    console.log("1000-", JSON.stringify(transaction));
    transaction.feePayer = new PublicKey(yanxiang_signer.publicKey);
    console.log("2000-", JSON.stringify(transaction));
    const anyTransaction = transaction;
    anyTransaction.recentBlockhash = (
      await connection.getLatestBlockhash()
    ).blockhash;

    console.log("3000-", JSON.stringify(transaction));

    //transaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
    /* const serializedTransaction = transaction.serialize({
      requireAllSignatures: false,
    });*/
    console.log("40000-", JSON.stringify(transaction));

    console.log("", JSON.stringify(transaction));
    //const a = await connection.sendTransaction(transaction, [yanxiang_signer]);
    //console.log("---",a);
    /*  let _vault = await getAccount(provider.connection,vault_account_pda);

      let _escrowAccount = await program.account.sellOrder.fetch(
          escrow_info_pda
      );

      // Check that the new owner is the PDA.
      assert.ok(_vault.owner.equals(vault_authority_pda));

      // Check that the values in the escrow account match what we expect.
      assert.ok(_escrowAccount.wallet.equals(seller.publicKey));
      assert.ok(_escrowAccount.price.toNumber() == buyer_coin_amount);
      assert.ok(
        _escrowAccount.nftTokenAccount.equals(seller_token_account)
      );*/
  });
  /* //买单
  it("buy nft", async () => {
    await program.rpc.buy({
      accounts: {
        buyer: buyer.publicKey,
        buyerCoinAccount: buyer_coin_account,
        buyerTokenAccount: buyer_token_account,
        kCoinMintAccount: new PublicKey("5d1i4wKHhGXXkdZB22iKD1SqU6pkBeTCwFEMqo7xy39h"),
        nftTokenMintAccount: mint_token,
        sellerCoinAccount: seller_coin_account,
        sellerTokenAccount: seller_token_account,
        seller: seller.publicKey,
        escrowAccount: escrowAccount.publicKey,
        vaultAccount: vault_account_pda,
        vaultAuthority: vault_authority_pda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: new PublicKey("11111111111111111111111111111111"),
        associatedTokenProgram: new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),
        rent: new PublicKey("SysvarRent111111111111111111111111111111111")
      },
      signers: [buyer]
    });
    //    let _vault = await getAccount(provider.connection,vault_account_pda);
    let _buyer_coin_account = await getAccount(provider.connection,buyer_coin_account);
    let _buyer_token_account = await getAccount(provider.connection,buyer_token_account);
    let _seller_coin_account = await getAccount(provider.connection,seller_coin_account);
    let _seller_token_account = await getAccount(provider.connection,seller_token_account);

    assert.ok(Number(_buyer_coin_account.amount) == 0);
    assert.ok(Number(_seller_coin_account.amount) == buyer_coin_amount);
    assert.ok(Number(_seller_token_account.amount) == 0);
    assert.ok(Number(_buyer_token_account.amount) == nft_amount);
  });*/

  it("sell nft and then cancel", async () => {
    //初始化创建NFT并给到卖家的nft的ata账户
    /* await mintTo(
        provider.connection,
        payer,
        mint_token,
        seller_token_account,
        payer.publicKey,
        nft_amount
    );
    await program.rpc.sell(
        vault_authority_pda.publicKey,
        new anchor.BN(buyer_coin_amount),
        {
          accounts: {
            seller: seller.publicKey,
            nftMint: mint_token,
            vaultAccount: vault_account_pda,
            sellerTokenAccount: seller_token_account,
            escrowAccount: escrowAccount.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            tokenProgram: TOKEN_PROGRAM_ID,
          },
          signers: [escrowAccount, seller],
        }
    );*/
    /*await program.rpc.cancel({
      accounts: {
        seller: seller.publicKey,
        sellerTokenAccount: seller_token_account,
        vaultAccount: vault_account_pda,
        vaultAuthority: vault_authority_pda,
        escrowAccount: escrowAccount.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [seller]
    });
    // Check the final owner should be the provider public key.
    let _initializerTokenAccountA = await getAccount(provider.connection,seller_token_account);
    assert.ok(_initializerTokenAccountA.owner.equals(seller.publicKey));

    // Check all the funds are still there.
    assert.ok(Number(_initializerTokenAccountA.amount) == nft_amount);*/
  });
});
