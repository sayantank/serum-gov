use anchor_lang::prelude::*;

#[error_code]
pub enum SerumGovError {
    #[msg("Ticket is not currently claimable.")]
    TicketNotClaimable,

    #[msg("Invalid amount for redeeming MSRM.")]
    InvalidMSRMAmount,

    #[msg("Invalid RedeemTicket")]
    InvalidRedeemTicket,

    #[msg("Invalid owner for ticket.")]
    InvalidTicketOwner,

    #[msg("Invalid amount for burning gSRM.")]
    InvalidGSRMAmount,

    #[msg("Too early to vest.")]
    TooEarlyToVest,
}
