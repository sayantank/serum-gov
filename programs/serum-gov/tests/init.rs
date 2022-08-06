mod core;
mod utils;

// #[cfg(feature = "test-bpf")]
mod init {
    use crate::{core::serum_gov::SerumGov, utils::helper::airdrop};
    use anchor_lang::AccountDeserialize;
    use anchor_spl::token::{Mint, TokenAccount};
    use solana_program_test::*;
    use solana_sdk::{signature::Keypair, signer::Signer};

    #[tokio::test]
    async fn success_init() {
        let program = ProgramTest::new("serum_gov", serum_gov::id(), None);

        let context = &mut program.start_with_context().await;

        let sbf = Keypair::new();
        airdrop(context, &sbf.pubkey(), 10_000_000_000).await;

        let serum_gov = SerumGov::new();

        serum_gov.setup_mints(context, &sbf).await;
        serum_gov.init(context, &sbf).await.unwrap();

        let srm_vault_account = context
            .banks_client
            .get_account(serum_gov.srm_vault)
            .await
            .unwrap()
            .unwrap();

        let srm_vault =
            TokenAccount::try_deserialize(&mut srm_vault_account.data.as_ref()).unwrap();

        assert_eq!(srm_vault.amount, 0);
        assert_eq!(srm_vault.owner, serum_gov.authority);
        assert_eq!(srm_vault.mint, serum_gov.srm_mint.pubkey());

        let msrm_vault_account = context
            .banks_client
            .get_account(serum_gov.msrm_vault)
            .await
            .unwrap()
            .unwrap();
        let msrm_vault =
            TokenAccount::try_deserialize(&mut msrm_vault_account.data.as_ref()).unwrap();
        assert_eq!(msrm_vault.amount, 0);
        assert_eq!(msrm_vault.owner, serum_gov.authority);
        assert_eq!(msrm_vault.mint, serum_gov.msrm_mint.pubkey());

        let gsrm_mint_account = context
            .banks_client
            .get_account(serum_gov.gsrm_mint)
            .await
            .unwrap()
            .unwrap();
        let gsrm_mint = Mint::try_deserialize(&mut gsrm_mint_account.data.as_ref()).unwrap();

        context.warp_to_slot(2).unwrap();

        assert_eq!(gsrm_mint.decimals, 6);
        assert_eq!(gsrm_mint.mint_authority.unwrap(), serum_gov.authority);
        assert_eq!(gsrm_mint.is_initialized, true);
        assert_eq!(gsrm_mint.supply, 0);
    }

    #[tokio::test]
    async fn fail_init_twice() {
        let program = ProgramTest::new("serum_gov", serum_gov::id(), None);

        let context = &mut program.start_with_context().await;

        let sbf = Keypair::new();
        airdrop(context, &sbf.pubkey(), 10_000_000_000).await;

        let serum_gov = SerumGov::new();

        serum_gov.setup_mints(context, &sbf).await;

        serum_gov.init(context, &sbf).await.unwrap();

        context.warp_to_slot(3).unwrap();

        let err = serum_gov.init(context, &sbf).await.unwrap_err();

        match err {
            BanksClientError::TransactionError(_) => assert!(true),
            _ => assert!(false),
        }
    }
}
