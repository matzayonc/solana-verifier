use serde::Deserialize;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::{EncodableKey, Signer},
    transaction::Transaction,
};
use solana_verifier::{Entrypoint, ProofAccount, PROGRAM_ID};
use std::{path::PathBuf, str::FromStr};
use swiftness::{parse, types::StarkProof, TransformTo};

#[derive(Debug, Deserialize)]
#[non_exhaustive]
struct SolanaConfig {
    json_rpc_url: String,
    keypair_path: PathBuf,
}

pub fn read_proof() -> StarkProof {
    let small_json = include_str!("../resources/saya.json");
    let stark_proof = parse(small_json).unwrap();
    stark_proof.transform_to()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize components
    let config =
        PathBuf::from(std::env::var("HOME").unwrap()).join(".config/solana/cli/config.yml");

    let config: SolanaConfig = serde_yaml::from_reader(std::fs::File::open(config)?)?;
    let client = RpcClient::new_with_commitment(config.json_rpc_url, CommitmentConfig::processed());
    let payer = Keypair::read_from_file(config.keypair_path)?;

    println!("Using keypair {}, at {}", payer.pubkey(), client.url());

    let data_address = Pubkey::from_str("4tLiFAEWRcssT763nxBnKh1eocNeptpSxEtAwH8r96W5").unwrap();
    let data = client.get_account_data(&data_address).await?;

    let proof_account = bytemuck::from_bytes::<ProofAccount>(&data[1..]);
    if proof_account.proof != read_proof() {
        eprintln!("data in the account does not match the proof");
        // } else if let Err(e) = verify_recursive_bytes(&mut data) {
        // eprintln!("Local verification failed: {:?}", e);
    }

    let ix = Instruction {
        program_id: Pubkey::from_str(PROGRAM_ID)?,
        accounts: vec![AccountMeta::new(data_address, false)],
        data: bincode::serialize(&Entrypoint::VerifyProof {}).unwrap(),
    };

    let blockhash = client.get_latest_blockhash().await?;

    let tx = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::request_heap_frame(32 * 1024),
            // ComputeBudgetInstruction::set_compute_unit_limit(1400_000),
            ix,
        ],
        Some(&payer.pubkey()),
        &[&payer],
        blockhash,
    );

    client.send_and_confirm_transaction(&tx).await.unwrap();

    Ok(())
}
