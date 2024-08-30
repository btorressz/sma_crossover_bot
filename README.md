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
