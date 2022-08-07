#![allow(unused)]

use solana_program_test::*;
use solana_sdk::{
    program_pack::Pack, pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};

pub async fn airdrop(context: &mut ProgramTestContext, receiver: &Pubkey, amount: u64) {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            receiver,
            amount,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
}

// pub async fn create_token_account(
//     context: &mut ProgramTestContext,
//     account: &Keypair,
//     mint: &Pubkey,
//     manager: &Pubkey,
// ) {
//     let rent = context.banks_client.get_rent().await.unwrap();

//     let tx = Transaction::new_signed_with_payer(
//         &[
//             system_instruction::create_account(
//                 &context.payer.pubkey(),
//                 &account.pubkey(),
//                 rent.minimum_balance(spl_token::state::Account::LEN),
//                 spl_token::state::Account::LEN as u64,
//                 &spl_token::id(),
//             ),
//             spl_token::instruction::initialize_account(
//                 &spl_token::id(),
//                 &account.pubkey(),
//                 mint,
//                 manager,
//             )
//             .unwrap(),
//         ],
//         Some(&context.payer.pubkey()),
//         &[&context.payer, &account],
//         context.last_blockhash,
//     );

//     context.banks_client.process_transaction(tx).await.unwrap();
// }

pub async fn mint_tokens(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    mint: &Pubkey,
    account: &Pubkey,
    amount: u64,
    additional_signer: Option<&Keypair>,
) -> Result<(), BanksClientError> {
    let mut signing_keypairs = vec![authority, &context.payer];
    if let Some(signer) = additional_signer {
        signing_keypairs.push(signer);
    }

    let ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint,
        account,
        &authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &signing_keypairs,
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

pub async fn create_associated_token_account(
    context: &mut ProgramTestContext,
    wallet: &Keypair,
    token_mint: &Pubkey,
) -> Pubkey {
    let recent_blockhash = context.last_blockhash;

    let tx = Transaction::new_signed_with_payer(
        &[
            spl_associated_token_account::instruction::create_associated_token_account(
                &wallet.pubkey(),
                &wallet.pubkey(),
                token_mint,
            ),
        ],
        Some(&wallet.pubkey()),
        &[wallet],
        recent_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    spl_associated_token_account::get_associated_token_address(&wallet.pubkey(), token_mint)
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    authority: &Pubkey,
    decimals: u8,
) {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                authority,
                None,
                decimals,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
}
