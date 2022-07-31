#[cfg(not(feature = "devnet"))]
pub mod mints {
    use solana_program::{pubkey, pubkey::Pubkey};
    pub const SRM: Pubkey = pubkey!("SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt");
    pub const MSRM: Pubkey = pubkey!("MSRMcoVyrFxnSgo5uXwone5SKcGhT1KEJMFEkMEWf9L");
}

#[cfg(feature = "devnet")]
pub mod mints {
    use solana_program::{pubkey, pubkey::Pubkey};
    pub const SRM: Pubkey = pubkey!("57z5KG1EHj5SV79xR1GVzEvkjWSJHgA7XMuPE457Rain");
    pub const MSRM: Pubkey = pubkey!("Hqyx6oJbZ2LBdshEP9ApdSMoo1xKQSgBjEAAbzJhMbZY");
}
