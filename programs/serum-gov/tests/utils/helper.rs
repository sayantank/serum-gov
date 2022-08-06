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