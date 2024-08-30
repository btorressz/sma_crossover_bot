// Client
console.log("My address:", pg.wallet.publicKey.toString());
try {
    const balance = await pg.connection.getBalance(pg.wallet.publicKey);
    console.log(`My balance: ${balance / web3.LAMPORTS_PER_SOL} SOL`);
} catch (error) {
    console.error("Error fetching balance:", error);
}

// Load the SmaCrossoverBot program
let program;
try {
    program = anchor.workspace.SmaCrossoverBot as Program<typeof anchor.workspace.SmaCrossoverBot>;
} catch (error) {
    console.error("Error loading the SmaCrossoverBot program:", error);
}

// Generate keypairs for the bot state and price oracle
const botState = anchor.web3.Keypair.generate();
const priceOracle = anchor.web3.Keypair.generate();

try {
    // Airdrop SOL to the bot state and price oracle accounts to cover transaction fees
    await pg.connection.requestAirdrop(botState.publicKey, web3.LAMPORTS_PER_SOL);
    await pg.connection.confirmTransaction(
        await pg.connection.requestAirdrop(priceOracle.publicKey, web3.LAMPORTS_PER_SOL)
    );
} catch (error) {
    console.error("Error during SOL airdrop:", error);
}

// Initialize the bot state
try {
    await program.rpc.initialize(new anchor.BN(5), new anchor.BN(20), {
        accounts: {
            botState: botState.publicKey,
            user: pg.wallet.publicKey,
            systemProgram: web3.SystemProgram.programId,
        },
        signers: [botState],
    });
    console.log("Initialized bot state");
} catch (error) {
    console.error("Error initializing bot state:", error);
}

try {
    // Fetch the latest price from the oracle and update the price oracle account
    // This example assumes you have an oracle that provides the price as a 64-bit integer
    const latestPrice = 100; // Replace with actual price-fetching logic
    const priceData = Buffer.from(Uint8Array.of(...new anchor.BN(latestPrice).toArray("le", 8)));

    // Update the price oracle with the latest price
    await provider.connection.sendTransaction(
        new anchor.web3.Transaction().add(
            new anchor.web3.TransactionInstruction({
                keys: [{ pubkey: priceOracle.publicKey, isSigner: false, isWritable: true }],
                programId: anchor.web3.SystemProgram.programId, // Replace with actual Oracle program ID
                data: priceData,
            })
        ),
        [priceOracle]
    );
    console.log("Updated price oracle with latest price");
} catch (error) {
    console.error("Error updating price oracle:", error);
}

try {
    // Calculate SMA using price from the oracle
    await program.rpc.calculateSma({
        accounts: {
            botState: botState.publicKey,
            priceOracle: priceOracle.publicKey,
        },
    });
    console.log("SMA calculated");
} catch (error) {
    console.error("Error calculating SMA:", error);
}

try {
    // Detect crossover
    await program.rpc.detectCrossover({
        accounts: {
            botState: botState.publicKey,
        },
    });
    console.log("Detected SMA crossover");
} catch (error) {
    console.error("Error detecting SMA crossover:", error);
}

let botStateAccount;
try {
    // Fetch the bot state account to check the last signal (Buy/Sell)
    botStateAccount = await program.account.botState.fetch(botState.publicKey);
} catch (error) {
    console.error("Error fetching bot state account:", error);
}

// Initialize user and bot token accounts (if not already initialized)
const userTokenAccount = anchor.web3.Keypair.generate();
const botTokenAccount = anchor.web3.Keypair.generate();

try {
    await provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(userTokenAccount.publicKey, web3.LAMPORTS_PER_SOL)
    );

    await provider.connection.sendTransaction(
        new anchor.web3.Transaction().add(
            anchor.web3.SystemProgram.createAccount({
                fromPubkey: pg.wallet.publicKey,
                newAccountPubkey: userTokenAccount.publicKey,
                space: 165, // Adjust space according to the token account requirements
                lamports: await provider.connection.getMinimumBalanceForRentExemption(165),
                programId: anchor.web3.TOKEN_PROGRAM_ID,
            }),
            anchor.web3.SystemProgram.createAccount({
                fromPubkey: pg.wallet.publicKey,
                newAccountPubkey: botTokenAccount.publicKey,
                space: 165,
                lamports: await provider.connection.getMinimumBalanceForRentExemption(165),
                programId: anchor.web3.TOKEN_PROGRAM_ID,
            })
        ),
        [userTokenAccount, botTokenAccount]
    );
    console.log("Initialized user and bot token accounts");
} catch (error) {
    console.error("Error initializing token accounts:", error);
}

try {
    // If the signal is "Buy", execute a buy trade. If "Sell", execute a sell trade
    if (botStateAccount.lastSignal === "Buy") {
        await program.rpc.executeTrade("Buy", {
            accounts: {
                botState: botState.publicKey,
                user: pg.wallet.publicKey,
                userTokenAccount: userTokenAccount.publicKey,
                botTokenAccount: botTokenAccount.publicKey,
                tokenProgram: anchor.web3.TOKEN_PROGRAM_ID,
            },
        });
        console.log("Executed Buy trade");
    } else if (botStateAccount.lastSignal === "Sell") {
        await program.rpc.executeTrade("Sell", {
            accounts: {
                botState: botState.publicKey,
                user: pg.wallet.publicKey,
                userTokenAccount: userTokenAccount.publicKey,
                botTokenAccount: botTokenAccount.publicKey,
                tokenProgram: anchor.web3.TOKEN_PROGRAM_ID,
            },
        });
        console.log("Executed Sell trade");
    }
} catch (error) {
    console.error("Error executing trade:", error);
}
