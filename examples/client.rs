use futures::{StreamExt, stream::FuturesUnordered};
use serde::Deserialize;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::{EncodableKey, Signer},
    system_instruction,
    transaction::Transaction,
};
use solana_verifier::{Entrypoint, PROGRAM_ID, ProofAccount};
use std::{path::PathBuf, str::FromStr, thread::sleep, time::Duration};
use tokio::fs;

const CHUNK_SIZE: usize = 500;

async fn send_transactions(
    client: &RpcClient,
    transactions: &[Transaction],
) -> Vec<Result<Signature, solana_rpc_client_api::client_error::Error>> {
    let mut futures = FuturesUnordered::new();

    for (idx, tx) in transactions.iter().enumerate() {
        sleep(Duration::from_millis(100));
        // Wrap each transaction in a future and track the result
        let future = async move { (idx, client.send_transaction(tx).await) };
        futures.push(future);
    }

    let mut results = Vec::new();

    while let Some(res) = futures.next().await {
        results.push(res.1)
    }

    println!("Results: {:?}", results);

    results
}

pub async fn read_proof_account() -> Box<ProofAccount> {
    let stark_proof = fs::read("resources/proof.bin").await.unwrap();
    Box::new(*bytemuck::from_bytes::<ProofAccount>(&stark_proof))
}

/// Creates a `Transaction` to create an account with rent exemption
async fn create_proof_data_account(
    client: &RpcClient,
    payer: &Keypair,
    proof_data_account: &Keypair,
    proof_size: usize,
    owner: &Pubkey,
) -> Result<Transaction, Box<dyn std::error::Error>> {
    let rent_exemption_amount = client
        .get_minimum_balance_for_rent_exemption(proof_size)
        .await?;

    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &proof_data_account.pubkey(),
        rent_exemption_amount,
        proof_size as u64,
        owner,
    );

    let blockhash = client.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[create_account_ix],
        Some(&payer.pubkey()),
        &[payer, proof_data_account],
        blockhash,
    );

    Ok(tx)
}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
struct SolanaConfig {
    json_rpc_url: String,
    keypair_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize components
    let config =
        PathBuf::from(std::env::var("HOME").unwrap()).join(".config/solana/cli/config.yml");

    let config: SolanaConfig = serde_yaml::from_reader(std::fs::File::open(config)?)?;
    let client = RpcClient::new(config.json_rpc_url.clone());
    let payer = Keypair::read_from_file(config.keypair_path)?;

    println!("Using keypair {}, at {}", payer.pubkey(), client.url());

    let account = read_proof_account().await;
    let stark_proof = bytemuck::bytes_of(&*account);

    let proof_data_account = Keypair::new();
    let program_id = Pubkey::from_str(PROGRAM_ID)?;

    println!("account pubkey: {:?}", proof_data_account.pubkey());
    client
        .send_and_confirm_transaction(
            &create_proof_data_account(
                &client,
                &payer,
                &proof_data_account,
                stark_proof.len() + 8, // +1 for the `stage` and 7 as padding
                &program_id,
            )
            .await?,
        )
        .await?;

    // for (section, section_data) in stark_proof.chunks(10000).enumerate() {
    // Allocate data instructions

    let instructions = stark_proof
        .chunks(CHUNK_SIZE)
        .enumerate()
        .map(|(i, data)| Instruction {
            program_id,
            accounts: vec![AccountMeta::new(proof_data_account.pubkey(), false)],
            data: bincode::serialize(&Entrypoint::PublishFragment {
                offset: i * CHUNK_SIZE,
                data,
            })
            .unwrap(),
        })
        .collect::<Vec<_>>();

    println!("Prepared instructions");

    let mut handles = Vec::new();
    for instructions in instructions.chunks(10) {
        let instructions = instructions.to_vec();
        let client = RpcClient::new(config.json_rpc_url.clone());
        let payer = Keypair::from_bytes(&payer.to_bytes()).unwrap();

        handles.push(tokio::spawn(async move {
            loop {
                let blockhash = client
                    .get_latest_blockhash()
                    .await
                    .expect("failed to connect to rpc");

                // Create corresponding transactions
                let transactions = instructions
                    .iter()
                    .map(|instruction| {
                        Transaction::new_signed_with_payer(
                            &[instruction.clone()],
                            Some(&payer.pubkey()),
                            &[&payer],
                            blockhash,
                        )
                    })
                    .collect::<Vec<_>>();

                let results = send_transactions(&client, &transactions).await;
                if results.iter().all(|r| r.is_ok()) {
                    break;
                }

                println!("Failed to send transactions, repeating batch.");
            }
        }));
    }

    for handle in handles {
        handle.await?;
    }

    println!("Sent publish instructions");

    loop {
        let data = client
            .get_account_data(&proof_data_account.pubkey())
            .await?;

        if data[8..].eq(stark_proof) {
            println!("proof_data_account correct!");
            break;
        } else {
            println!("proof_data_account data not matching!");
            sleep(Duration::from_secs(1));
        }
    }

    let schedule_ix = Instruction {
        program_id,
        accounts: vec![AccountMeta::new(proof_data_account.pubkey(), false)],
        data: bincode::serialize(&Entrypoint::Schedule {}).unwrap(),
    };

    let verify_ix = Instruction {
        program_id,
        accounts: vec![AccountMeta::new(proof_data_account.pubkey(), false)],
        data: bincode::serialize(&Entrypoint::VerifyProof {}).unwrap(),
    };

    let mut verify_ixs = (0..15).map(|_| verify_ix.clone()).collect::<Vec<_>>();
    verify_ixs.insert(0, schedule_ix);

    let blockhash = client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &verify_ixs,
        Some(&payer.pubkey()),
        &[&payer],
        blockhash,
    );

    client.send_and_confirm_transaction(&tx).await.unwrap();

    Ok(())
}
