use anchor_lang::prelude::*;

#[error_code]
pub enum SerumGovError {
    #[msg("Ticket is not currently claimable.")]
    TicketNotClaimable,

    #[msg("Invalid amount for redeeming MSRM.")]
    InvalidMSRMAmount,

    #[msg("Invalid RedeemTicket")]
    InvalidRedeemTicket,
}
