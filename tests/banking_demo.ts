import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { Ibank } from "../target/types/ibank";
import { randomBytes } from "crypto";
import {
  awaitComputationFinalization,
  getArciumEnv,
  getCompDefAccOffset,
  getArciumProgAddress,
  uploadCircuit,
  buildFinalizeCompDefTx,
  RescueCipher,
  deserializeLE,
  getMXEAccAddress,
  getMempoolAccAddress,
  getCompDefAccAddress,
  getExecutingPoolAccAddress,
  x25519,
  getComputationAccAddress,
  getArciumAccountBaseSeed,
  getMXEPublicKey,
} from "@arcium-hq/client";
import * as fs from "fs";
import * as os from "os";
import { expect } from "chai";

describe("Privacy-First Banking Demo", () => {
  const owner = readKpJson(`${os.homedir()}/.config/solana/id.json`);

  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.Blackjack as Program<Ibank>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;

  type Event = anchor.IdlEvents<(typeof program)["idl"]>;
  const awaitEvent = async <E extends keyof Event>(
    eventName: E,
    timeoutMs = 60000
  ): Promise<Event[E]> => {
    let listenerId: number;
    let timeoutId: NodeJS.Timeout;
    const event = await new Promise<Event[E]>((res, rej) => {
      listenerId = program.addEventListener(eventName as any, (event) => {
        if (timeoutId) clearTimeout(timeoutId);
        res(event);
      });
      timeoutId = setTimeout(() => {
        program.removeEventListener(listenerId);
        rej(new Error(`Event ${eventName} timed out after ${timeoutMs}ms`));
      }, timeoutMs);
    });
    await program.removeEventListener(listenerId);
    return event;
  };

  const arciumEnv = getArciumEnv();

  it("Should execute a complete privacy-first banking flow", async () => {
    console.log("Owner address:", owner.publicKey.toBase58());

    // --- Initialize Computation Definitions ---
    console.log("Initializing computation definitions...");
    await Promise.all([
      initInitializeAccountsCompDef(program as any, owner, false, false).then(
        (sig) => console.log("Initialize Accounts CompDef Init Sig:", sig)
      ),
      initProcessPaymentCompDef(program as any, owner, false, false).then(
        (sig) => console.log("Process Payment CompDef Init Sig:", sig)
      ),
      initCheckBalanceCompDef(program as any, owner, false, false).then((sig) =>
        console.log("Check Balance CompDef Init Sig:", sig)
      ),
      initCalculateRewardsCompDef(program as any, owner, false, false).then(
        (sig) => console.log("Calculate Rewards CompDef Init Sig:", sig)
      ),
    ]);
    console.log("All computation definitions initialized.");
    await new Promise((res) => setTimeout(res, 2000));

    // --- Setup Cryptography ---
    const privateKey = x25519.utils.randomSecretKey();
    const publicKey = x25519.getPublicKey(privateKey);
    const mxePublicKey = await getMXEPublicKeyWithRetry(
      provider as anchor.AnchorProvider,
      program.programId
    );

    console.log("MXE x25519 pubkey is", mxePublicKey);
    const sharedSecret = x25519.getSharedSecret(privateKey, mxePublicKey);
    const cipher = new RescueCipher(sharedSecret);

    // --- Setup Account IDs and PDAs ---
    const account1Id = BigInt(Math.floor(Math.random() * 1000000));
    const account2Id = BigInt(Math.floor(Math.random() * 1000000));
    const transactionId = BigInt(Math.floor(Math.random() * 1000000));

    const account1IdBuffer = Buffer.alloc(8);
    account1IdBuffer.writeBigUInt64LE(account1Id);
    const account2IdBuffer = Buffer.alloc(8);
    account2IdBuffer.writeBigUInt64LE(account2Id);
    const transactionIdBuffer = Buffer.alloc(8);
    transactionIdBuffer.writeBigUInt64LE(transactionId);

    const account1PDA = PublicKey.findProgramAddressSync(
      [Buffer.from("user_account"), account1IdBuffer],
      program.programId
    )[0];
    const account2PDA = PublicKey.findProgramAddressSync(
      [Buffer.from("user_account"), account2IdBuffer],
      program.programId
    )[0];
    const transactionPDA = PublicKey.findProgramAddressSync(
      [Buffer.from("transaction"), transactionIdBuffer],
      program.programId
    )[0];

    console.log(`Account 1 ID: ${account1Id}, PDA: ${account1PDA.toBase58()}`);
    console.log(`Account 2 ID: ${account2Id}, PDA: ${account2PDA.toBase58()}`);

    // --- Initialize Account 1 ---
    console.log("\n=== Initializing Account 1 ===");
    const initAccount1Offset = new anchor.BN(randomBytes(8));
    const mxeNonce1 = randomBytes(16);
    const clientNonce1 = randomBytes(16);
    const initialBalance1 = 10000; // Starting with 10000 units

    const accountInitializedEventPromise1 = awaitEvent(
      "accountInitializedEvent"
    );

    console.log("Calling initialize_user_account for Account 1...");
    const initAccount1Sig = await program.methods
      .initializeUserAccount(
        initAccount1Offset,
        new anchor.BN(account1Id.toString()),
        new anchor.BN(initialBalance1),
        new anchor.BN(deserializeLE(mxeNonce1).toString()),
        Array.from(publicKey),
        new anchor.BN(deserializeLE(clientNonce1).toString())
      )
      .accountsPartial({
        computationAccount: getComputationAccAddress(
          program.programId,
          initAccount1Offset
        ),
        clusterAccount: arciumEnv.arciumClusterPubkey,
        mxeAccount: getMXEAccAddress(program.programId),
        mempoolAccount: getMempoolAccAddress(program.programId),
        executingPool: getExecutingPoolAccAddress(program.programId),
        compDefAccount: getCompDefAccAddress(
          program.programId,
          Buffer.from(
            getCompDefAccOffset("initialize_accounts")
          ).readUInt32LE()
        ),
        userAccount: account1PDA,
        payer: owner.publicKey,
      })
      .signers([owner])
      .rpc({ commitment: "confirmed" });
    console.log("Initialize Account 1 TX Signature:", initAccount1Sig);

    console.log("Waiting for account 1 initialization finalization...");
    await awaitComputationFinalization(
      provider,
      initAccount1Offset,
      program.programId,
      "confirmed"
    );
    console.log("Account 1 initialization finalized.");

    const accountInitializedEvent1 = await accountInitializedEventPromise1;
    console.log("Received AccountInitializedEvent for Account 1.");
    console.log("Account 1 Balance Nonce:", accountInitializedEvent1.balanceNonce.toString());

    let account1State = await program.account.userAccount.fetch(account1PDA);
    expect(account1State.accountState).to.deep.equal({ active: {} });
    console.log("Account 1 initialized successfully!");

    // --- Initialize Account 2 ---
    console.log("\n=== Initializing Account 2 ===");
    const initAccount2Offset = new anchor.BN(randomBytes(8));
    const mxeNonce2 = randomBytes(16);
    const clientNonce2 = randomBytes(16);
    const initialBalance2 = 5000; // Starting with 5000 units

    const accountInitializedEventPromise2 = awaitEvent(
      "accountInitializedEvent"
    );

    console.log("Calling initialize_user_account for Account 2...");
    const initAccount2Sig = await program.methods
      .initializeUserAccount(
        initAccount2Offset,
        new anchor.BN(account2Id.toString()),
        new anchor.BN(initialBalance2),
        new anchor.BN(deserializeLE(mxeNonce2).toString()),
        Array.from(publicKey),
        new anchor.BN(deserializeLE(clientNonce2).toString())
      )
      .accountsPartial({
        computationAccount: getComputationAccAddress(
          program.programId,
          initAccount2Offset
        ),
        clusterAccount: arciumEnv.arciumClusterPubkey,
        mxeAccount: getMXEAccAddress(program.programId),
        mempoolAccount: getMempoolAccAddress(program.programId),
        executingPool: getExecutingPoolAccAddress(program.programId),
        compDefAccount: getCompDefAccAddress(
          program.programId,
          Buffer.from(
            getCompDefAccOffset("initialize_accounts")
          ).readUInt32LE()
        ),
        userAccount: account2PDA,
        payer: owner.publicKey,
      })
      .signers([owner])
      .rpc({ commitment: "confirmed" });
    console.log("Initialize Account 2 TX Signature:", initAccount2Sig);

    console.log("Waiting for account 2 initialization finalization...");
    await awaitComputationFinalization(
      provider,
      initAccount2Offset,
      program.programId,
      "confirmed"
    );
    console.log("Account 2 initialization finalized.");

    const accountInitializedEvent2 = await accountInitializedEventPromise2;
    console.log("Received AccountInitializedEvent for Account 2.");
    console.log("Account 2 Balance Nonce:", accountInitializedEvent2.balanceNonce.toString());

    let account2State = await program.account.userAccount.fetch(account2PDA);
    expect(account2State.accountState).to.deep.equal({ active: {} });
    console.log("Account 2 initialized successfully!");

    // --- Process Payment from Account 1 to Account 2 ---
    console.log("\n=== Processing Payment (Account 1 → Account 2) ===");
    const paymentOffset = new anchor.BN(randomBytes(8));
    const paymentAmount = 1500; // Transfer 1500 units

    const paymentProcessedEventPromise = awaitEvent("paymentProcessedEvent");

    console.log(`Sending ${paymentAmount} units from Account 1 to Account 2...`);
    const receiverNewNonce = randomBytes(16);
    const paymentSig = await program.methods
      .processPayment(
        paymentOffset,
        new anchor.BN(transactionId.toString()),
        new anchor.BN(paymentAmount),
        new anchor.BN(deserializeLE(receiverNewNonce).toString())
      )
      .accountsPartial({
        computationAccount: getComputationAccAddress(
          program.programId,
          paymentOffset
        ),
        clusterAccount: arciumEnv.arciumClusterPubkey,
        mxeAccount: getMXEAccAddress(program.programId),
        mempoolAccount: getMempoolAccAddress(program.programId),
        executingPool: getExecutingPoolAccAddress(program.programId),
        compDefAccount: getCompDefAccAddress(
          program.programId,
          Buffer.from(getCompDefAccOffset("process_payment")).readUInt32LE()
        ),
        senderAccount: account1PDA,
        receiverAccount: account2PDA,
        transaction: transactionPDA,
        payer: owner.publicKey,
      })
      .signers([owner])
      .rpc({ commitment: "confirmed" });
    console.log("Process Payment TX Signature:", paymentSig);

    console.log("Waiting for payment processing finalization...");
    await awaitComputationFinalization(
      provider,
      paymentOffset,
      program.programId,
      "confirmed"
    );
    console.log("Payment processing finalized.");

    const paymentProcessedEvent = await paymentProcessedEventPromise;
    console.log("Received PaymentProcessedEvent.");
    console.log("Transaction ID:", paymentProcessedEvent.transactionId.toString());

    const transactionState = await program.account.transaction.fetch(
      transactionPDA
    );
    expect(transactionState.status).to.deep.equal({ completed: {} });
    console.log("Payment completed successfully!");

    // --- Check Balance for Account 1 ---
    console.log("\n=== Checking Balance for Account 1 ===");
    const checkBalanceOffset = new anchor.BN(randomBytes(8));
    const balanceThreshold = 5000; // Check if balance > 5000

    const balanceCheckEventPromise = awaitEvent("balanceCheckEvent");

    console.log(`Checking if Account 1 balance > ${balanceThreshold}...`);
    const checkBalanceSig = await program.methods
      .checkBalance(
        checkBalanceOffset,
        new anchor.BN(account1Id.toString()),
        new anchor.BN(balanceThreshold)
      )
      .accountsPartial({
        computationAccount: getComputationAccAddress(
          program.programId,
          checkBalanceOffset
        ),
        clusterAccount: arciumEnv.arciumClusterPubkey,
        mxeAccount: getMXEAccAddress(program.programId),
        mempoolAccount: getMempoolAccAddress(program.programId),
        executingPool: getExecutingPoolAccAddress(program.programId),
        compDefAccount: getCompDefAccAddress(
          program.programId,
          Buffer.from(getCompDefAccOffset("check_balance")).readUInt32LE()
        ),
        userAccount: account1PDA,
        payer: owner.publicKey,
      })
      .signers([owner])
      .rpc({ commitment: "confirmed" });
    console.log("Check Balance TX Signature:", checkBalanceSig);

    console.log("Waiting for balance check finalization...");
    await awaitComputationFinalization(
      provider,
      checkBalanceOffset,
      program.programId,
      "confirmed"
    );
    console.log("Balance check finalized.");

    const balanceCheckEvent = await balanceCheckEventPromise;
    console.log("Received BalanceCheckEvent.");
    console.log("Is Above Threshold:", balanceCheckEvent.isAboveThreshold);
    // After sending 1500 from 10000, balance should be 8500, which is > 5000
    expect(balanceCheckEvent.isAboveThreshold).to.be.true;
    console.log("Balance check completed successfully!");

    // --- Calculate Rewards for Account 1 ---
    console.log("\n=== Calculating Rewards for Account 1 ===");
    const rewardsOffset = new anchor.BN(randomBytes(8));

    const rewardsCalculatedEventPromise = awaitEvent("rewardsCalculatedEvent");

    console.log("Calculating rewards for Account 1...");
    const rewardsSig = await program.methods
      .calculateRewards(rewardsOffset, new anchor.BN(account1Id.toString()))
      .accountsPartial({
        computationAccount: getComputationAccAddress(
          program.programId,
          rewardsOffset
        ),
        clusterAccount: arciumEnv.arciumClusterPubkey,
        mxeAccount: getMXEAccAddress(program.programId),
        mempoolAccount: getMempoolAccAddress(program.programId),
        executingPool: getExecutingPoolAccAddress(program.programId),
        compDefAccount: getCompDefAccAddress(
          program.programId,
          Buffer.from(getCompDefAccOffset("calculate_rewards")).readUInt32LE()
        ),
        userAccount: account1PDA,
        payer: owner.publicKey,
      })
      .signers([owner])
      .rpc({ commitment: "confirmed" });
    console.log("Calculate Rewards TX Signature:", rewardsSig);

    console.log("Waiting for rewards calculation finalization...");
    await awaitComputationFinalization(
      provider,
      rewardsOffset,
      program.programId,
      "confirmed"
    );
    console.log("Rewards calculation finalized.");

    const rewardsCalculatedEvent = await rewardsCalculatedEventPromise;
    console.log("Received RewardsCalculatedEvent.");
    console.log("Reward Points:", rewardsCalculatedEvent.rewardPoints.toString());
    console.log("Total Rewards:", rewardsCalculatedEvent.totalRewards.toString());

    account1State = await program.account.userAccount.fetch(account1PDA);
    expect(account1State.rewardPoints.toNumber()).to.be.greaterThan(0);
    console.log("Rewards calculated successfully!");

    console.log("\n=== Banking Demo Complete ===");
    console.log("✅ All operations executed successfully!");
    console.log("✅ Account 1 has completed 1 transaction");
    console.log("✅ Account 1 earned reward points");
    console.log("✅ All balances remain encrypted on-chain");
  });

  // --- Helper Functions for Computation Definition Initialization ---

  async function initInitializeAccountsCompDef(
    program: Program<Ibank>,
    owner: Keypair,
    uploadRawCircuit: boolean,
    offchainSource: boolean
  ): Promise<string> {
    const baseSeedCompDefAcc = getArciumAccountBaseSeed(
      "ComputationDefinitionAccount"
    );
    const offset = getCompDefAccOffset("initialize_accounts");

    const compDefPDA = PublicKey.findProgramAddressSync(
      [baseSeedCompDefAcc, program.programId.toBuffer(), offset],
      getArciumProgAddress()
    )[0];

    console.log("Initialize Accounts CompDef PDA:", compDefPDA.toBase58());

    try {
      await program.account.computationDefinitionAccount.fetch(compDefPDA);
      console.log("Initialize Accounts CompDef already initialized.");
      return "Already Initialized";
    } catch (e) {
      // Not initialized, proceed
    }

    const sig = await program.methods
      .initInitializeAccountsCompDef()
      .accounts({
        compDefAccount: compDefPDA,
        payer: owner.publicKey,
        mxeAccount: getMXEAccAddress(program.programId),
      })
      .rpc({ commitment: "confirmed" });

    if (uploadRawCircuit) {
      const rawCircuit = fs.readFileSync("build/initialize_accounts.arcis");

      await uploadCircuit(
        provider as anchor.AnchorProvider,
        "initialize_accounts",
        program.programId,
        rawCircuit,
        true
      );
    } else if (!offchainSource) {
      console.log("Finalizing Initialize Accounts CompDef...");
      const finalizeTx = await buildFinalizeCompDefTx(
        provider,
        Buffer.from(offset).readUInt32LE(),
        program.programId
      );
      const latestBlockhash = await provider.connection.getLatestBlockhash();
      finalizeTx.recentBlockhash = latestBlockhash.blockhash;
      finalizeTx.lastValidBlockHeight = latestBlockhash.lastValidBlockHeight;
      finalizeTx.sign(owner);
      await provider.sendAndConfirm(finalizeTx, [owner], {
        commitment: "confirmed",
      });
      console.log("Initialize Accounts CompDef finalized.");
    }
    return sig;
  }

  async function initProcessPaymentCompDef(
    program: Program<Ibank>,
    owner: Keypair,
    uploadRawCircuit: boolean,
    offchainSource: boolean
  ): Promise<string> {
    const baseSeedCompDefAcc = getArciumAccountBaseSeed(
      "ComputationDefinitionAccount"
    );
    const offset = getCompDefAccOffset("process_payment");

    const compDefPDA = PublicKey.findProgramAddressSync(
      [baseSeedCompDefAcc, program.programId.toBuffer(), offset],
      getArciumProgAddress()
    )[0];

    console.log("Process Payment CompDef PDA:", compDefPDA.toBase58());

    try {
      await program.account.computationDefinitionAccount.fetch(compDefPDA);
      console.log("Process Payment CompDef already initialized.");
      return "Already Initialized";
    } catch (e) {
      // Not initialized, proceed
    }

    const sig = await program.methods
      .initProcessPaymentCompDef()
      .accounts({
        compDefAccount: compDefPDA,
        payer: owner.publicKey,
        mxeAccount: getMXEAccAddress(program.programId),
      })
      .rpc({ commitment: "confirmed" });

    if (uploadRawCircuit) {
      const rawCircuit = fs.readFileSync("build/process_payment.arcis");

      await uploadCircuit(
        provider as anchor.AnchorProvider,
        "process_payment",
        program.programId,
        rawCircuit,
        true
      );
    } else if (!offchainSource) {
      console.log("Finalizing Process Payment CompDef...");
      const finalizeTx = await buildFinalizeCompDefTx(
        provider,
        Buffer.from(offset).readUInt32LE(),
        program.programId
      );
      const latestBlockhash = await provider.connection.getLatestBlockhash();
      finalizeTx.recentBlockhash = latestBlockhash.blockhash;
      finalizeTx.lastValidBlockHeight = latestBlockhash.lastValidBlockHeight;
      finalizeTx.sign(owner);
      await provider.sendAndConfirm(finalizeTx, [owner], {
        commitment: "confirmed",
      });
      console.log("Process Payment CompDef finalized.");
    }
    return sig;
  }

  async function initCheckBalanceCompDef(
    program: Program<Ibank>,
    owner: Keypair,
    uploadRawCircuit: boolean,
    offchainSource: boolean
  ): Promise<string> {
    const baseSeedCompDefAcc = getArciumAccountBaseSeed(
      "ComputationDefinitionAccount"
    );
    const offset = getCompDefAccOffset("check_balance");

    const compDefPDA = PublicKey.findProgramAddressSync(
      [baseSeedCompDefAcc, program.programId.toBuffer(), offset],
      getArciumProgAddress()
    )[0];

    console.log("Check Balance CompDef PDA:", compDefPDA.toBase58());

    try {
      await program.account.computationDefinitionAccount.fetch(compDefPDA);
      console.log("Check Balance CompDef already initialized.");
      return "Already Initialized";
    } catch (e) {
      // Not initialized, proceed
    }

    const sig = await program.methods
      .initCheckBalanceCompDef()
      .accounts({
        compDefAccount: compDefPDA,
        payer: owner.publicKey,
        mxeAccount: getMXEAccAddress(program.programId),
      })
      .rpc({ commitment: "confirmed" });

    if (uploadRawCircuit) {
      const rawCircuit = fs.readFileSync("build/check_balance.arcis");

      await uploadCircuit(
        provider as anchor.AnchorProvider,
        "check_balance",
        program.programId,
        rawCircuit,
        true
      );
    } else if (!offchainSource) {
      console.log("Finalizing Check Balance CompDef...");
      const finalizeTx = await buildFinalizeCompDefTx(
        provider,
        Buffer.from(offset).readUInt32LE(),
        program.programId
      );
      const latestBlockhash = await provider.connection.getLatestBlockhash();
      finalizeTx.recentBlockhash = latestBlockhash.blockhash;
      finalizeTx.lastValidBlockHeight = latestBlockhash.lastValidBlockHeight;
      finalizeTx.sign(owner);
      await provider.sendAndConfirm(finalizeTx, [owner], {
        commitment: "confirmed",
      });
      console.log("Check Balance CompDef finalized.");
    }
    return sig;
  }

  async function initCalculateRewardsCompDef(
    program: Program<Ibank>,
    owner: Keypair,
    uploadRawCircuit: boolean,
    offchainSource: boolean
  ): Promise<string> {
    const baseSeedCompDefAcc = getArciumAccountBaseSeed(
      "ComputationDefinitionAccount"
    );
    const offset = getCompDefAccOffset("calculate_rewards");

    const compDefPDA = PublicKey.findProgramAddressSync(
      [baseSeedCompDefAcc, program.programId.toBuffer(), offset],
      getArciumProgAddress()
    )[0];

    console.log("Calculate Rewards CompDef PDA:", compDefPDA.toBase58());

    try {
      await program.account.computationDefinitionAccount.fetch(compDefPDA);
      console.log("Calculate Rewards CompDef already initialized.");
      return "Already Initialized";
    } catch (e) {
      // Not initialized, proceed
    }

    const sig = await program.methods
      .initCalculateRewardsCompDef()
      .accounts({
        compDefAccount: compDefPDA,
        payer: owner.publicKey,
        mxeAccount: getMXEAccAddress(program.programId),
      })
      .rpc({ commitment: "confirmed" });

    if (uploadRawCircuit) {
      const rawCircuit = fs.readFileSync("build/calculate_rewards.arcis");

      await uploadCircuit(
        provider as anchor.AnchorProvider,
        "calculate_rewards",
        program.programId,
        rawCircuit,
        true
      );
    } else if (!offchainSource) {
      console.log("Finalizing Calculate Rewards CompDef...");
      const finalizeTx = await buildFinalizeCompDefTx(
        provider,
        Buffer.from(offset).readUInt32LE(),
        program.programId
      );
      const latestBlockhash = await provider.connection.getLatestBlockhash();
      finalizeTx.recentBlockhash = latestBlockhash.blockhash;
      finalizeTx.lastValidBlockHeight = latestBlockhash.lastValidBlockHeight;
      finalizeTx.sign(owner);
      await provider.sendAndConfirm(finalizeTx, [owner], {
        commitment: "confirmed",
      });
      console.log("Calculate Rewards CompDef finalized.");
    }
    return sig;
  }
});

// --- Helper Functions ---

async function getMXEPublicKeyWithRetry(
  provider: anchor.AnchorProvider,
  programId: PublicKey,
  maxRetries: number = 10,
  retryDelayMs: number = 500
): Promise<Uint8Array> {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      const mxePublicKey = await getMXEPublicKey(provider, programId);
      if (mxePublicKey) {
        return mxePublicKey;
      }
    } catch (error) {
      console.log(`Attempt ${attempt} failed to fetch MXE public key:`, error);
    }

    if (attempt < maxRetries) {
      console.log(
        `Retrying in ${retryDelayMs}ms... (attempt ${attempt}/${maxRetries})`
      );
      await new Promise((resolve) => setTimeout(resolve, retryDelayMs));
    }
  }

  throw new Error(
    `Failed to fetch MXE public key after ${maxRetries} attempts`
  );
}

function readKpJson(path: string): anchor.web3.Keypair {
  const file = fs.readFileSync(path);
  return anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(file.toString()))
  );
}
