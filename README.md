# Serum Governance Token Program 💧

This program allows locking and vesting of SRM/MSRM tokens to receive **gSRM** tokens, which can then be used to take part in the Serum DAO Governance.

## How it works?

Users will need to initialize a `User` account for themselves using the `init_user` instruction before they can execute further instructions from the program.

### Locking SRM/MSRM 🔒

Users can then lock their SRM and MSRM tokens using the `deposit_locked_srm` and `deposit_locked_msrm` instructions respectively, which would then,

1. Create a `LockedAccount` for the deposited tokens.
2. Issue a `ClaimTicket`, which can be used to claim the gSRM tokens, after `CLAIM_DELAY` seconds have passed since creation, using the `claim` instruction.

The claimed gSRM can also be used to redeem the SRM/MSRM tokens that were locked. This is done by the `burn_locked_gsrm` instruction, which also takes in the `LockedAccount` to make sure misuse of the claimed gSRM and allows redeeming of tokens in batches if required. The `burn_locked_gsrm` instruction would then,

1. Update `LockedAccount` according to the amount of gSRM burned.
2. Issue a `RedeemTicket`, which can be used to receive the SRM/MSRM tokens back, after `REDEEM_DELAY` seconds have passed since creation, using the `redeem_srm` and `redeem_msrm` instructions.

---

### Vesting SRM 📈

Users can vest SRM for another user using the `deposit_vest_srm` instruction, which would then,

1. Create a `VestAccount` for the deposited tokens. This also stores information such as `CLIFF_PERIOD` and `LINEAR_VESTING_PERIOD` which are configurable constants in the program.
2. Issue a `ClaimTicket`, which can be used to claim the gSRM tokens, after `CLAIM_DELAY` seconds have passed since creation, using the `claim` instruction.

The owner of the VestAccount can then redeem SRM tokens using the claimed gSRM tokens, following a linear vesting schedule. This is done using the `burn_vest_gsrm` instruction, which takes in the `VestAccount` to calculate the amount of SRM that has vested. The `burn_vest_gsrm` instruction would then,

1. Update `VestAccount` according to the amount of gSRM burned.
2. Issue a `RedeemTicket`, which can be used to receive the SRM tokens, after `REDEEM_DELAY` seconds have passed since creation, using the `redeem_srm` instruction. The amount of SRM tokens that can be redeemed is calculated using the `CLIFF_PERIOD`, `LINEAR_VEST_PERIOD`, `clock.unix_timestamp` and `VestAccount.gsrm_burned`.

## Devnet Addresses:

- SRM_MINT: `6pJxsghghEcgpwmukyhtaFEjibAPdQyXQQb69Jn7CdST`
- MSRM_MINT: `5Y51LyyackBZPvzLWwjB9mTpGP5Y8Tz9EXdr6w2k4SgZ`

Init signature: `WVE9dGwmhKHhJxX5Ngzyw3dbfrVVgQv277yYrwRdRcKprELwRQ77jHiUM1CxamxtZDdV4yC3L7iaEemCwRRf8jT`
