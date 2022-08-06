pub mod mints {
    use solana_program::{pubkey, pubkey::Pubkey};
    pub const SRM: Pubkey = pubkey!("SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt");
    pub const MSRM: Pubkey = pubkey!("MSRMcoVyrFxnSgo5uXwone5SKcGhT1KEJMFEkMEWf9L");
}

pub mod parameters {
    pub const CLAIM_DELAY: i64 = 1000;
    pub const REDEEM_DELAY: i64 = 1000;
}
