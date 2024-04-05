use anchor_lang::prelude::*;
use anchor_client::solana_sdk::signature::Signer;
use anchor_client::Client;
use std::str::FromStr;

// Assumes you have a function to start the ICO and set up the initial state
#[test]
fn test_start_ico() {
    let program = Client::new().program("Fg6PaFhzQxQfen8DHuZQDVpokndbCWTJcDJTYYkg4LTG");
    let admin_pubkey = Pubkey::from_str("YourAdminAccountPubkey").unwrap();
    let admin = program.payer();
    assert_eq!(admin.pubkey(), admin_pubkey);
    
    // Example phase details: adjust according to your actual Phase struct
    let phase_details = vec![
        Phase { token_price: 100, start: 1625097600, end: 1627689600 }, // Example phase
        // Add more phases as needed
    ];
    
    // Start the ICO
    let tx = program.rpc().start_ico(phase_details);
    assert!(tx.is_ok());
    println!("ICO started successfully with transaction: {:?}", tx.unwrap());
}

// Test buying tokens
#[test]
fn test_buy_tokens() {
    let program = Client::new().program("Fg6PaFhzQxQfen8DHuZQDVpokndbCWTJcDJTYYkg4LTG");
    let buyer = program.payer();
    let amount_sol = 1_000_000_000; // 1 SOL for testing
    
    // Assume a function to get or create the buyer's PledgeToken account
    let buyer_token_account = get_or_create_pledge_token_account(&buyer);

    // Buy tokens
    let tx = program.rpc().buy_tokens(amount_sol, buyer_token_account.pubkey());
    assert!(tx.is_ok());
    println!("Tokens purchased successfully with transaction: {:?}", tx.unwrap());
}

// Placeholder for actual function to get or create a token account for the buyer
fn get_or_create_pledge_token_account(buyer: &Keypair) -> Keypair {
    // Implement token account creation or retrieval logic
    // Return the token account Keypair or Pubkey as needed
    Keypair::new() // Placeholder
}

