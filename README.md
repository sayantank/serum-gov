# Serum Governance Token Program 💧

This program allows locking and vesting of SRM/MSRM tokens to receive **gSRM** tokens, which can then be used to take part in the Serum DAO Governance.

## How it works?

### Prerequisites

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

## Addresses:

### Main `serum_gov` Instance

- Program ID: [`FBcTbv5rLy7MQkkAU2uDzAEjjZDeu2BVLVRJGxyz6hnV`](https://explorer.solana.com/address/FBcTbv5rLy7MQkkAU2uDzAEjjZDeu2BVLVRJGxyz6hnV)
- Init Signature: [`21K6657LRHDnuwDnP53ncM1UKBVhViVvmtyxUxNwGyomph4hUA6piumgrroUmoBUGeDnR9nhP83QV6CiT5prVp8C`](https://explorer.solana.com/tx/21K6657LRHDnuwDnP53ncM1UKBVhViVvmtyxUxNwGyomph4hUA6piumgrroUmoBUGeDnR9nhP83QV6CiT5prVp8C)
- gSRM Mint: [`G6DyPo5NjpW5kAvZwvM7hx1KeTUgGmuykPMdKuwWRvER`](https://explorer.solana.com/address/G6DyPo5NjpW5kAvZwvM7hx1KeTUgGmuykPMdKuwWRvER)

### Test `serum_gov` Instance

These should be used for testing on mainnet-beta and devnet.

- Program ID: [`EDV6BNBY6pLb4aCJCc5LnELdA9xTywnDZ2m3cWfCbpwZ`](https://explorer.solana.com/address/EDV6BNBY6pLb4aCJCc5LnELdA9xTywnDZ2m3cWfCbpwZ)
- SRM: [`2xKASju8WCUK6zC54TP4h6WhHdqdcWMNoFpqAdvXvHV6`](https://explorer.solana.com/address/2xKASju8WCUK6zC54TP4h6WhHdqdcWMNoFpqAdvXvHV6)
- MSRM: [`BoFBTKtdMXC4YALXtNV5tmw1xNWtjxTrR17PvZGmKhmP`](https://explorer.solana.com/address/BoFBTKtdMXC4YALXtNV5tmw1xNWtjxTrR17PvZGmKhmP)

### Realms

#### SPL Governance: [`G41fmJzd29v7Qmdi8ZyTBBYa98ghh3cwHBTexqCG1PQJ`](https://explorer.solana.com/address/G41fmJzd29v7Qmdi8ZyTBBYa98ghh3cwHBTexqCG1PQJ)

- Mainnet Realm: [`G3FBDbsRiJjcjYuazrH6mRShFMjr9RQn4SxVVxocJavA`](https://explorer.solana.com/address/G3FBDbsRiJjcjYuazrH6mRShFMjr9RQn4SxVVxocJavA)
- Council Token: [`GwNuCfsN5bEdtQyghvyqEU8BMornrpnGiGv8tBjTPj3Q`](https://explorer.solana.com/address/GwNuCfsN5bEdtQyghvyqEU8BMornrpnGiGv8tBjTPj3Q)

---

- Devnet Realm: [`439YMWzq623G6EMowVjTFcHnn4y13tBa876NKdzvjcEr`](https://explorer.solana.com/address/439YMWzq623G6EMowVjTFcHnn4y13tBa876NKdzvjcEr)
- Mainnet **TEST** Realm: [`3pFLtCJzoewv9aB4JZDhGdRb4xQeJRtVpd66QgpNTDwP`](https://explorer.solana.com/address/3pFLtCJzoewv9aB4JZDhGdRb4xQeJRtVpd66QgpNTDwP)
