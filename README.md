# Vesting Smart Contract

This repository contains a **Vesting Smart Contract** designed to securely manage time-based token distribution on-chain. The contract enforces controlled release of tokens according to predefined vesting schedules, ensuring that token allocations are distributed transparently, predictably, and without manual intervention.

The vesting logic is implemented using **ink!**, making it compatible with smart-contract-enabled **Substrate-based blockchains**. It is designed to be minimal, auditable, and production-ready for real-world token economies.

---

## Overview

Token vesting is a critical component of sustainable tokenomics. This contract provides an on-chain mechanism to lock tokens and release them over time based on predefined rules. All vesting conditions are enforced at the smart contract level, eliminating reliance on off-chain processes or trusted intermediaries.

The contract supports multiple vesting schedules, approval-based transfers, and strict ownership controls to ensure secure and predictable token distribution.

---

## Features

- **Time-based vesting schedules**  
  Tokens are released according to predefined timelines, preventing premature access.

- **Owner-controlled approvals**  
  Only the designated vesting owner can approve or execute vesting-related transfers.

- **Multiple vesting schedules**  
  Supports managing multiple vesting entries for different recipients.

- **On-chain enforcement**  
  Vesting rules are enforced entirely on-chain, ensuring transparency and trust minimization.

- **Lightweight and auditable**  
  The contract is intentionally simple to reduce attack surface and ease auditing.

- **Substrate / ink! compatible**  
  Deployable on smart-contract-enabled Substrate chains.

---

## Use Cases

- Team and advisor token vesting  
- Investor lockups and cliff-based releases  
- Ecosystem incentives and rewards  
- Treasury-controlled token distribution  
- Governance-approved token emissions  

---

## Contract Design

At a high level, the contract:

1. Defines a **vesting owner** responsible for approvals and control
2. Stores vesting schedules indexed by a schedule identifier
3. Enforces time-based conditions before allowing token release
4. Emits events for transparency and off-chain indexing
5. Restricts sensitive operations to authorized accounts

This design ensures that token distribution follows clearly defined rules and cannot be bypassed or altered without proper authorization.

---

## Security Considerations

- Only the **vesting owner** can approve or execute vesting actions
- Vesting schedules cannot be claimed before their release conditions are met
- All critical state transitions emit events for traceability
- The contract avoids unnecessary complexity to minimize risk

> ⚠️ **Disclaimer:**  
> This smart contract has not been audited unless explicitly stated. Use in production environments at your own risk and consider a professional security audit before deployment.

---

## Development

### Prerequisites

- Rust (stable or nightly as required by ink!)
- ink! toolchain
- Substrate contract environment

### Build

```bash
cargo +nightly contract build
