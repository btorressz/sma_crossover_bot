async function getRecentBlockhashWithRetry(retries = 3) {
  for (let i = 0; i < retries; i++) {
    try {
      const recentBlockhash = await provider.connection.getRecentBlockhash();
      return recentBlockhash;
    } catch (error) {
      if (i === retries - 1) {
        throw error; // Rethrow if last attempt fails
      }
      console.log("Retrying to get recent blockhash...");
      await new Promise((resolve) => setTimeout(resolve, 1000)); // Wait 1 second before retrying
    }
  }
}

describe("SmaCrossoverBot", () => {
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  const program = anchor.workspace.SmaCrossoverBot as Program<typeof anchor.workspace.SmaCrossoverBot>;

  it("Initializes the bot state!", async () => {
    const botState = anchor.web3.Keypair.generate();

    await getRecentBlockhashWithRetry();

    await program.rpc.initialize(new anchor.BN(5), new anchor.BN(20), {
      accounts: {
        botState: botState.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [botState],
    });

    const account = await program.account.botState.fetch(botState.publicKey);
    assert.equal(account.shortPeriod.toNumber(), 5);
    assert.equal(account.longPeriod.toNumber(), 20);
    assert.equal(account.admin.toString(), provider.wallet.publicKey.toString());
  });

  it("Fails to calculate SMA if price data is insufficient", async () => {
    const botState = anchor.web3.Keypair.generate();
    const priceOracle = anchor.web3.Keypair.generate();

    await getRecentBlockhashWithRetry();

    await program.rpc.initialize(new anchor.BN(5), new anchor.BN(20), {
      accounts: {
        botState: botState.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [botState],
    });

    // Simulate insufficient price data
    try {
      await program.rpc.calculateSma({
        accounts: {
          botState: botState.publicKey,
          priceOracle: priceOracle.publicKey,
        },
      });
      assert.fail("Expected error not thrown");
    } catch (err) {
      assert.equal(err.error.errorCode.code, "InsufficientPriceData");
    }
  });

  it("Calculates SMA with sufficient price data", async () => {
    const botState = anchor.web3.Keypair.generate();
    const priceOracle = anchor.web3.Keypair.generate();

    await getRecentBlockhashWithRetry();

    // Initialize the bot state
    await program.rpc.initialize(new anchor.BN(5), new anchor.BN(20), {
      accounts: {
        botState: botState.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [botState],
    });

    // Mock oracle data with sufficient historical prices
    const priceData = Buffer.from(Uint8Array.of(...new anchor.BN(100).toArray("le", 8)));
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(priceOracle.publicKey, 1e9)
    );
    await provider.connection.sendTransaction(
      new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.createAccount({
          fromPubkey: provider.wallet.publicKey,
          newAccountPubkey: priceOracle.publicKey,
          space: priceData.length,
          lamports: await provider.connection.getMinimumBalanceForRentExemption(priceData.length),
          programId: anchor.web3.SystemProgram.programId,
        }),
        new anchor.web3.TransactionInstruction({
          keys: [{ pubkey: priceOracle.publicKey, isSigner: false, isWritable: true }],
          programId: anchor.web3.SystemProgram.programId,
          data: priceData,
        })
      ),
      [priceOracle]
    );

    // Calculate SMA using the mocked price oracle
    await program.rpc.calculateSma({
      accounts: {
        botState: botState.publicKey,
        priceOracle: priceOracle.publicKey,
      },
    });

    const account = await program.account.botState.fetch(botState.publicKey);
    assert.equal(account.shortSma.toNumber(), 100);
    assert.equal(account.longSma.toNumber(), 100);
    assert.equal(account.lastPrice.toNumber(), 100);
  });

  it("Detects SMA crossover and generates a trade signal!", async () => {
    const botState = anchor.web3.Keypair.generate();

    await getRecentBlockhashWithRetry();

    // Initialize the bot state
    await program.rpc.initialize(new anchor.BN(5), new anchor.BN(20), {
      accounts: {
        botState: botState.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [botState],
    });

    // Mock some SMA calculations
    await program.rpc.calculateSma({
      accounts: {
        botState: botState.publicKey,
        priceOracle: provider.wallet.publicKey, // Assume the oracle gives correct price
      },
    });

    // Detect crossover
    await program.rpc.detectCrossover({
      accounts: {
        botState: botState.publicKey,
      },
    });

    const account = await program.account.botState.fetch(botState.publicKey);
    assert.equal(account.lastSignal, "Buy"); // Assuming the signal is Buy based on mock
  });

  it("Executes trade based on the generated signal!", async () => {
    const botState = anchor.web3.Keypair.generate();
    const userTokenAccount = anchor.web3.Keypair.generate();
    const botTokenAccount = anchor.web3.Keypair.generate();

    await getRecentBlockhashWithRetry();

    // Initialize the bot state
    await program.rpc.initialize(new anchor.BN(5), new anchor.BN(20), {
      accounts: {
        botState: botState.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [botState],
    });

    // Mock trade signal
    await program.rpc.detectCrossover({
      accounts: {
        botState: botState.publicKey,
      },
    });

    // Execute trade
    await program.rpc.executeTrade("Buy", {
      accounts: {
        botState: botState.publicKey,
        user: provider.wallet.publicKey,
        userTokenAccount: userTokenAccount.publicKey,
        botTokenAccount: botTokenAccount.publicKey,
        tokenProgram: anchor.web3.TOKEN_PROGRAM_ID,
      },
    });

    // Verify the trade execution (mock example, add actual logic)
    const account = await program.account.botState.fetch(botState.publicKey);
    assert.equal(account.lastSignal, "Buy"); // Assuming the signal was Buy
  });

  it("Fails to update periods if unauthorized", async () => {
    const botState = anchor.web3.Keypair.generate();
    const otherUser = anchor.web3.Keypair.generate();

    await getRecentBlockhashWithRetry();

    // Initialize the bot state
    await program.rpc.initialize(new anchor.BN(5), new anchor.BN(20), {
      accounts: {
        botState: botState.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [botState],
    });

    // Attempt to update periods with an unauthorized user
    try {
      await program.rpc.updatePeriods(new anchor.BN(10), new anchor.BN(30), {
        accounts: {
          botState: botState.publicKey,
          user: otherUser.publicKey,
        },
        signers: [otherUser],
      });
      assert.fail("Expected error not thrown");
    } catch (err) {
      assert.equal(err.error.errorCode.code, "Unauthorized");
    }
  });
});
