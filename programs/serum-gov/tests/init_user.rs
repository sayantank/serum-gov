mod core;
mod utils;

// #[cfg(feature = "test-bpf")]
mod init_user {
    use crate::{
        core::{serum_gov::SerumGov, user::UserAccount},
        utils::helper::airdrop,
    };
    use anchor_lang::AccountDeserialize;
    use serum_gov::state::User;
    use solana_program_test::*;
    use solana_sdk::{signature::Keypair, signer::Signer};

    #[tokio::test]
    async fn success_init_user() {
        let program = ProgramTest::new("serum_gov", serum_gov::id(), None);

        let context = &mut program.start_with_context().await;

        let sbf = Keypair::new();
        airdrop(context, &sbf.pubkey(), 10_000_000_000).await;

        let serum_gov = SerumGov::new();

        serum_gov.setup_mints(context, &sbf).await;
        serum_gov.init(context, &sbf).await.unwrap();

        let user = UserAccount::new();
        airdrop(context, &user.owner.pubkey(), 10_000_000_000).await;

        user.init_user(context).await.unwrap();

        let user_account = context
            .banks_client
            .get_account(user.user_account)
            .await
            .unwrap()
            .unwrap();

        let user_account_data = User::try_deserialize(&mut user_account.data.as_ref()).unwrap();

        assert_eq!(user_account_data.claim_index, 0);
        assert_eq!(user_account_data.redeem_index, 0);
        assert_eq!(user_account_data.vest_index, 0);
        assert_eq!(user_account_data.owner, user.owner.pubkey());
    }
}
