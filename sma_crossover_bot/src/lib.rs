use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};

// This is your program's public key and it will update automatically when you build the project.
declare_id!("DW3q5N5KdcuUXaRpLxUpHWRu3vMGvsUyqYQZmw4Jmud4");

#[program]
mod sma_crossover_bot {
    use super::*;

    /// Initializes the bot state with the provided short and long SMA periods.
    pub fn initialize(ctx: Context<Initialize>, short_period: u64, long_period: u64) -> Result<()> {
        let bot_state = &mut ctx.accounts.bot_state;
        bot_state.short_sma = 0;
        bot_state.long_sma = 0;
        bot_state.last_price = 0;
        bot_state.prices = vec![];
        bot_state.short_period = short_period;
        bot_state.long_period = long_period;
        bot_state.admin = ctx.accounts.user.key();

        // Emit an event to log the initialization
        emit!(BotInitializedEvent {
            admin: bot_state.admin,
            short_period: bot_state.short_period,
            long_period: bot_state.long_period,
        });

        Ok(())
    }

    /// Calculates the short-term and long-term SMAs using the latest price from the oracle.
    pub fn calculate_sma(ctx: Context<CalculateSma>) -> Result<()> {
        let bot_state = &mut ctx.accounts.bot_state;

        // Ensure there are enough prices to calculate SMAs
        if bot_state.prices.len() < bot_state.long_period as usize {
            return err!(BotError::InsufficientPriceData);
        }

        // Fetch the latest price from the oracle
        let price = utils::get_price_from_oracle(&ctx.accounts.price_oracle)?;

        // Store the new price in the historical data
        bot_state.prices.push(price);
        if bot_state.prices.len() > bot_state.long_period as usize {
            bot_state.prices.remove(0);
        }

        // Calculate SMAs
        bot_state.short_sma = bot_state.prices.iter().rev().take(bot_state.short_period as usize).sum::<u64>() / bot_state.short_period;
        bot_state.long_sma = bot_state.prices.iter().rev().take(bot_state.long_period as usize).sum::<u64>() / bot_state.long_period;

        bot_state.last_price = price;

        // Emit an event to log the SMA calculation
        emit!(SmaCalculatedEvent {
            short_sma: bot_state.short_sma,
            long_sma: bot_state.long_sma,
            price,
        });

        Ok(())
    }

    /// Detects SMA crossover and generates a trade signal.
    pub fn detect_crossover(ctx: Context<DetectCrossover>) -> Result<()> {
        let bot_state = &mut ctx.accounts.bot_state;
        let signal = if bot_state.short_sma > bot_state.long_sma {
            TradeSignal::Buy
        } else {
            TradeSignal::Sell
        };

        // Emit an event to log the trade signal
        emit!(TradeSignalEvent {
            signal: signal.clone(),
            short_sma: bot_state.short_sma,
            long_sma: bot_state.long_sma,
            price: bot_state.last_price,
        });

        bot_state.last_signal = signal.clone();
        Ok(())
    }

    /// Executes a trade based on the generated trade signal.
    pub fn execute_trade(ctx: Context<ExecuteTrade>, signal: TradeSignal) -> Result<()> {
        let bot_state = &mut ctx.accounts.bot_state;
        let user = &ctx.accounts.user;

        // Ensure the user is the admin
        require!(user.key() == bot_state.admin, BotError::Unauthorized);

        match signal {
            TradeSignal::Buy => {
                // Implement buy logic
                let cpi_accounts = Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.bot_token_account.to_account_info(),
                    authority: user.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                token::transfer(cpi_ctx, 1)?; // Example transfer amount
            },
            TradeSignal::Sell => {
                // Implement sell logic
                let cpi_accounts = Transfer {
                    from: ctx.accounts.bot_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: user.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                token::transfer(cpi_ctx, 1)?; // Example transfer amount
            },
        }

        // Emit an event to log the trade execution
        emit!(TradeExecutionEvent {
            signal,
            executed_at: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Allows the admin to update the SMA periods.
    pub fn update_periods(ctx: Context<UpdatePeriods>, short_period: u64, long_period: u64) -> Result<()> {
        let bot_state = &mut ctx.accounts.bot_state;
        require!(ctx.accounts.user.key() == bot_state.admin, BotError::Unauthorized);
        require!(short_period > 0 && long_period > short_period, BotError::InvalidPeriod);

        bot_state.short_period = short_period;
        bot_state.long_period = long_period;

        // Emit an event to log the period update
        emit!(PeriodsUpdatedEvent {
            admin: ctx.accounts.user.key(),
            short_period,
            long_period,
        });

        Ok(())
    }
}

// Define the account structures

/// Account context for initializing the bot state.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 64 + 1024)] // Increased space for historical prices and new fields
    pub bot_state: Account<'info, BotState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Account context for calculating SMAs.
#[derive(Accounts)]
pub struct CalculateSma<'info> {
    #[account(mut)]
    pub bot_state: Account<'info, BotState>,
    /// The account info representing the price oracle.
    pub price_oracle: AccountInfo<'info>,
}

/// Account context for detecting SMA crossover.
#[derive(Accounts)]
pub struct DetectCrossover<'info> {
    #[account(mut)]
    pub bot_state: Account<'info, BotState>,
}

/// Account context for executing trades.
#[derive(Accounts)]
pub struct ExecuteTrade<'info> {
    #[account(mut)]
    pub bot_state: Account<'info, BotState>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub bot_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, token::Token>,
}

/// Account context for updating SMA periods.
#[derive(Accounts)]
pub struct UpdatePeriods<'info> {
    #[account(mut)]
    pub bot_state: Account<'info, BotState>,
    #[account(mut)]
    pub user: Signer<'info>,
}

// Define the state of the bot

/// The state of the bot, storing SMAs, historical prices, and configuration.
#[account]
pub struct BotState {
    pub short_sma: u64,
    pub long_sma: u64,
    pub last_price: u64,
    pub last_signal: TradeSignal,
    pub prices: Vec<u64>,
    pub short_period: u64,
    pub long_period: u64,
    pub admin: Pubkey,
}

// Define trade signals

/// Enumeration of trade signals: Buy or Sell.
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum TradeSignal {
    Buy,
    Sell,
}

// Events for logging

/// Event emitted when the bot is initialized.
#[event]
pub struct BotInitializedEvent {
    pub admin: Pubkey,
    pub short_period: u64,
    pub long_period: u64,
}

/// Event emitted when SMAs are calculated.
#[event]
pub struct SmaCalculatedEvent {
    pub short_sma: u64,
    pub long_sma: u64,
    pub price: u64,
}

/// Event emitted when a trade signal is generated.
#[event]
pub struct TradeSignalEvent {
    pub signal: TradeSignal,
    pub short_sma: u64,
    pub long_sma: u64,
    pub price: u64,
}

/// Event emitted when a trade is executed.
#[event]
pub struct TradeExecutionEvent {
    pub signal: TradeSignal,
    pub executed_at: i64,
}

/// Event emitted when SMA periods are updated.
#[event]
pub struct PeriodsUpdatedEvent {
    pub admin: Pubkey,
    pub short_period: u64,
    pub long_period: u64,
}

// Error definitions

/// Custom errors for the bot.
#[error_code]
pub enum BotError {
    #[msg("Insufficient funds to execute the trade.")]
    InsufficientFunds,
    #[msg("Unauthorized action.")]
    Unauthorized,
    #[msg("Failed to fetch price from oracle.")]
    OracleDataError,
    #[msg("Not enough price data to calculate SMA.")]
    InsufficientPriceData,
    #[msg("Invalid period values.")]
    InvalidPeriod,
}

// Utility functions
pub mod utils {
    use super::*;

    /// Utility function to fetch price from the oracle.
    pub fn get_price_from_oracle(oracle: &AccountInfo) -> Result<u64> {
        // Example logic to fetch price from oracle data
        let data = &oracle.try_borrow_data()?;
        let price: u64 = u64::from_le_bytes(data[0..8].try_into().unwrap());
        Ok(price)
    }
}
