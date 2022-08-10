pub mod mints {
    use solana_program::{pubkey, pubkey::Pubkey};
    pub const SRM: Pubkey = pubkey!("SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt");
    pub const MSRM: Pubkey = pubkey!("MSRMcoVyrFxnSgo5uXwone5SKcGhT1KEJMFEkMEWf9L");
}

#[cfg(not(feature = "test-bpf"))]
pub mod parameters {
    pub const CLAIM_DELAY: i64 = 1000;
    pub const REDEEM_DELAY: i64 = 1000;
    pub const CLIFF_PERIOD: i64 = 1000;
    pub const LINEAR_VESTING_PERIOD: i64 = 1000;
}

#[cfg(feature = "test-bpf")]
pub mod parameters {
    pub const CLAIM_DELAY: i64 = 2;
    pub const REDEEM_DELAY: i64 = 2;
    pub const CLIFF_PERIOD: i64 = 5;
    pub const LINEAR_VESTING_PERIOD: i64 = 30;
}
