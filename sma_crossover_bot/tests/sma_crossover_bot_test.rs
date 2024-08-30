use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, TokenAccount, Token};
use solana_program::instruction::Instruction;
use solana_program::system_instruction;
use solana_program_test::*;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use std::convert::TryInto;
use sma_crossover_bot::{self, id, instruction, processor::process_instruction, state::BotState, TradeSignal};

#[tokio::test]
async fn test_initialize() {
    let program_test = ProgramTest::new(
        "sma_crossover_bot",
        id(),
        processor!(sma_crossover_bot::entry),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let bot_state = Keypair::new();

    let initialize_ix = instruction::Initialize {
        short_period: 5,
        long_period: 20,
    }
    .data();

    let accounts = vec![
        AccountMeta::new(bot_state.pubkey(), true),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(system_program::ID, false),
    ];

    let instruction = Instruction {
        program_id: id(),
        accounts,
        data: initialize_ix,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &bot_state], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    let bot_state_account = banks_client
        .get_account(bot_state.pubkey())
        .await
        .expect("Account not found")
        .expect("Account empty");

    let bot_state_data: BotState = try_from_slice_unchecked(&bot_state_account.data).unwrap();
    assert_eq!(bot_state_data.short_period, 5);
    assert_eq!(bot_state_data.long_period, 20);
    assert_eq!(bot_state_data.admin, payer.pubkey());
}

#[tokio::test]
async fn test_calculate_sma() {
    let program_test = ProgramTest::new(
        "sma_crossover_bot",
        id(),
        processor!(sma_crossover_bot::entry),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let bot_state = Keypair::new();
    let price_oracle = Keypair::new();

    // Initialize the bot state
    let initialize_ix = instruction::Initialize {
        short_period: 5,
        long_period: 20,
    }
    .data();

    let accounts = vec![
        AccountMeta::new(bot_state.pubkey(), true),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(system_program::ID, false),
    ];

    let instruction = Instruction {
        program_id: id(),
        accounts,
        data: initialize_ix,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &bot_state], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    // Create the price oracle account and write initial price data
    let price_data: Vec<u8> = vec![100, 0, 0, 0, 0, 0, 0, 0]; // Price is 100

    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &price_oracle.pubkey(),
        1_000_000,
        price_data.len() as u64,
        &system_program::ID,
    );

    let write_data_ix = Instruction {
        program_id: system_program::ID,
        accounts: vec![
            AccountMeta::new(price_oracle.pubkey(), false),
            AccountMeta::new(payer.pubkey(), true),
        ],
        data: price_data.clone(),
    };

    let mut transaction = Transaction::new_with_payer(
        &[create_account_ix, write_data_ix],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[&payer, &price_oracle], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Calculate SMA using the price from the oracle
    let calculate_sma_ix = instruction::CalculateSma {}.data();

    let accounts = vec![
        AccountMeta::new(bot_state.pubkey(), true),
        AccountMeta::new(price_oracle.pubkey(), false),
    ];

    let instruction = Instruction {
        program_id: id(),
        accounts,
        data: calculate_sma_ix,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    let bot_state_account = banks_client
        .get_account(bot_state.pubkey())
        .await
        .expect("Account not found")
        .expect("Account empty");

    let bot_state_data: BotState = try_from_slice_unchecked(&bot_state_account.data).unwrap();
    assert_eq!(bot_state_data.short_sma, 100);
    assert_eq!(bot_state_data.long_sma, 100);
    assert_eq!(bot_state_data.last_price, 100);
}

#[tokio::test]
async fn test_detect_crossover() {
    let program_test = ProgramTest::new(
        "sma_crossover_bot",
        id(),
        processor!(sma_crossover_bot::entry),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let bot_state = Keypair::new();

    // Initialize the bot state
    let initialize_ix = instruction::Initialize {
        short_period: 5,
        long_period: 20,
    }
    .data();

    let accounts = vec![
        AccountMeta::new(bot_state.pubkey(), true),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(system_program::ID, false),
    ];

    let instruction = Instruction {
        program_id: id(),
        accounts,
        data: initialize_ix,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &bot_state], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    // Simulate SMA calculation
    let calculate_sma_ix = instruction::CalculateSma {}.data();

    let accounts = vec![
        AccountMeta::new(bot_state.pubkey(), true),
        AccountMeta::new(payer.pubkey(), false), // Assume the oracle gives correct price
    ];

    let instruction = Instruction {
        program_id: id(),
        accounts,
        data: calculate_sma_ix,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    // Detect crossover
    let detect_crossover_ix = instruction::DetectCrossover {}.data();

    let accounts = vec![AccountMeta::new(bot_state.pubkey(), true)];

    let instruction = Instruction {
        program_id: id(),
        accounts,
        data: detect_crossover_ix,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    let bot_state_account = banks_client
        .get_account(bot_state.pubkey())
        .await
        .expect("Account not found")
        .expect("Account empty");

    let bot_state_data: BotState = try_from_slice_unchecked(&bot_state_account.data).unwrap();
    assert_eq!(bot_state_data.last_signal, TradeSignal::Buy); // Assuming the signal is Buy based on mock
}

#[tokio::test]
async fn test_execute_trade() {
    let program_test = ProgramTest::new(
        "sma_crossover_bot",
        id(),
        processor!(sma_crossover_bot::entry),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let bot_state = Keypair::new();
    let user_token_account = Keypair::new();
    let bot_token_account = Keypair::new();

    // Initialize the bot state
    let initialize_ix = instruction::Initialize {
        short_period: 5,
        long_period: 20,
    }
    .data();

    let accounts = vec![
        AccountMeta::new(bot_state.pubkey(), true),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(system_program::ID, false),
    ];

    let instruction = Instruction {
        program_id: id(),
        accounts,
        data: initialize_ix,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &bot_state], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    // Simulate a crossover detection
    let detect_crossover_ix = instruction::DetectCrossover {}.data();

    let accounts = vec![AccountMeta::new(bot_state.pubkey(), true)];

    let instruction = Instruction {
        program_id: id(),
        accounts,
        data: detect_crossover_ix,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    // Execute trade based on the signal
    let execute_trade_ix = instruction::ExecuteTrade {
        signal: TradeSignal::Buy,
    }
    .data();

    let accounts = vec![
        AccountMeta::new(bot_state.pubkey(), true),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(user_token_account.pubkey(), false),
        AccountMeta::new(bot_token_account.pubkey(), false),
        AccountMeta::new(token::ID, false),
    ];

    let instruction = Instruction {
        program_id: id(),
        accounts,
        data: execute_trade_ix,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();

    let bot_state_account = banks_client
        .get_account(bot_state.pubkey())
        .await
        .expect("Account not found")
        .expect("Account empty");

    let bot_state_data: BotState = try_from_slice_unchecked(&bot_state_account.data).unwrap();
    assert_eq!(bot_state_data.last_signal, TradeSignal::Buy); // Assuming the signal was Buy
}
