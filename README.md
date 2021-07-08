# dot-contract

## Description 
Nsure is an open insurance platform for Open Finance. The project borrows the idea of Lloyd’s London, a market place to trade insurance risks, where premiums are determined by a Dynamic Pricing Model. Capital mining will be implemented to secure capital required to back the risks at any point of time. A 3-phase crowd voting mechanism is used to ensure every claim is handled professionally. You can get all the information here: https://nsure.network/Nsure_WP_0.7.pdf

## Contract Functions
Support capital mining and underwriter deposit/withdraw.
This is the mvp for nsure.

capital_converter converts the dot token to ndot which is the token used to stake in Nsure's capital pool. Ndot reperesents your share when deposited into the capital pool. Rewards in Nsure will be distributed based on time weighted manner.

erc20 is a smart contract similiar with erc20 token contract.

capital_stake is the staking mining contract. Stake nDot to get reward in Nsure token.

underwrite is the contract for Nsure token staking. Rewards can be adjusted.

## Test
Run `cargo +nightly test` to do testing.

## License
Nsure dot contracts and all other utilities are licensed under [Apache 2.0](LICENSE).

## Contact
If you want any further information, feel free to contact us at **contact@nsure.network** :) ...


