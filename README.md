# sma_crossover_bot

## Overview

- **The SMA Crossover Bot is a smart contract developed on the Solana blockchain using the Anchor framework. This project was built entirely within the Solana Playground IDE, with no local environment setup required. It implements a simple moving average (SMA) trading strategy, which buys or sells assets based on the crossover of short-term and long-term SMAs.**

**Please note that this project is still under review, and feedback or contributions are highly encouraged to enhance it.**

## Features
- **Initialization: Configure the bot with custom short-term and long-term SMA periods.**
- **SMA Calculation: Dynamically calculate SMAs based on incoming price data from a price oracle.**
- **Crossover Detection: Detect when a short-term SMA crosses over a long-term SMA to generate trade signals.**
- **Trade Execution: Automatically execute buy or sell trades based on the generated trade signal.**
- **Period Update: Allows the bot admin to update SMA periods as needed.**
- Detailed Event Logging: Emit events for important actions like initialization, SMA calculation, signal detection, and trade execution.

  ## Smart Contract Overview
  **Modules**
- Initialize: Sets up the bot with the user's specified SMA periods.
- CalculateSma: Computes the SMAs based on historical price data.
- DetectCrossover: Analyzes SMAs to detect potential buy or sell signals.
- ExecuteTrade: Executes trades based on the detected signals.
- UpdatePeriods: Allows the admin to update the short and long SMA periods.
  
