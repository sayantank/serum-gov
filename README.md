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

- PROGRAM_ID: `6j5LZrgXu5jkazUK859aZGMvSSguw7S5tLSxmrvgbDtj`

- GSRM_MINT: `CMi5Kbvt1qwhguyyWhq5Ht7aJx4mHX3wK6J7ekfB17VZ`
- SRM_MINT: `Ct4hKM9ULHYNMjuRMgwy8BJqhbadPVpdJT5thGa9qHqv`
- MSRM_MINT: `B9jfBeKRoYU9iPVcuTZGpSx2j4CetzHAbvXgYEW5Qhu6`

- Init signature: `31WongMEba7aZiUr1sxmL1B6zaMkT86rmAjFKfbGi6usmUpr4NxCCVTtPyNZA1Jey2UJnzq6xZdRxY32PN43xAkv`

- Test Realm: `Aod3XoyS64dA63BReZf9ENVp6UvXwcEYFs7pHVoABJFJ`

- REALM_1: `C2E8coHoNGEb1veDmWAH14xdKfzdf3BenMnKSfucUAy1`
- DAO_1_MINT: `23Shpjj1q7vsn6P2JxmB1h8SdHJpLDqDMHaR4DUtuqMW`

- REALM_2: `G1ZnwPJGnRt35nadw33df7hxKHkVu13juMjuE4Pp5FWN`
- DAO_2_MINT: `HRYLSoqWi646zoqqaxrBsYp1wp7qMmW8ix49wMMyKjEf`
