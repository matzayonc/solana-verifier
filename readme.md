# Swiftness Solana Program

Solana program, using Swiftness to verify Cairo proofs on the Solana blockchain.

## Overview

This program is used to verify Cairo proofs on the Solana blockchain. Due to the constraints of the Solana program interface, the proof is split into multiple (hundreds) of transactions.

The process of verification is done in 3 stages, varying in the callers access to the proof account.

### Publish

In the first stage, the caller has full access to the proof account, except the `stage` field itself. The caller uploads `bytemucked` proof, together with some helper values in the `cache` field.

### Schedule

In the second stage, the caller doesn't have access to the proof account. The contract will split verification into multiple tasks and add them to the `schedule` field.
Because of the variable size of of the proof, number of tasks in not constant. After creating the schedule it can be modified to further split big tasks into smaller subtasks.

### Verification cranking

The contract will execute tasks, one by one, until the proof is verified. Once the tasks stack is empty, the `stage` field is updated to `Verified`. The verified proof can the be used to create a Fact, and the memory used for another proof.

## Task model

Because of the memory constraints it's important to keep as much data in the `cache` field as possible. This effectively means that most of variables used in the verification process are now global variables.
Because this is error prone, the `View` structs are introduced. This structs keep references to the relevant parts of the proof, and are passed to the `Task`s.

## Development methodology

The target of this project is to create a contract with tasks small enough to be verified in a single transaction. At the start there is only a single task, `VerifyProof`, and it will be split into smaller tasks.
The most computationally expensive part of the `VerifyProof` task will be split, until all of them will fit the limit.

The compute limit can be lowered during development to show measurable progress in development.

## Usage

### Local Node Setup

Use the Solana CLI to create a new account.

```bash
solana-keygen new
```

Start Local Validator and set it as default endpoint

```bash
solana-test-validator
solana config set -u localhost
```

### Program Setup

Build the program, this will generate a new program id.

```bash
cargo build-sbf
```

Update the program id in `src/lib.rs`, this has to be done only once.

```bash
solana address -k target/deploy/solana_verifier-keypair.json
```

Proceed to deploy the program.

### Deployment

After setting up, new changes can be made to the program by rebuilding and redeploying.

```bash
cargo build-sbf && solana program deploy target/deploy/solana_verifier.so
```

### Usage

Run client to send and verify an example proof

```bash
cargo run --example client
```

To only verify already uploaded proofs, run the validate example, but update the address of the proof data account.

```bash
cargo run --example validate
```

### Tests

Run the tests, requires more stack space than default.

```bash
RUST_MIN_STACK=4096000 cargo test
```
