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

## Smart Contract Deployment
Compile 

Go to root directory, run `cargo +nightly contract build` to compile

Deploy

Deploy file in corresponding directory target/ink/*.contract

1. Go to  `https://patrastore.io/#/jupiter-a1/system/network` to get test tokens to deploy smart contract

2. Open  `https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fws.jupiter-poa.patract.cn#/explorer`

3. Select testnet Jupiter -> via Patract

Deploy 3 contracts step by step

1. Deploy erc20 nsure contract

- Initiate contrct name as 'nsure'
- Initiate parameter initialSupply = 1000000000000000000，name=nsure,symbol=nsure,decimals=10
- add minter，mint token，transfer nsure

2. Deploy capital_convert contract

- Set contract name as 'capitalConvert'
- Initiate parameter name=nDot,symbol=nDot,decimals=10,token=3gV4DFkJKtEPs3Y4fhqSQssx6duhcFvBfjkXbZgMe7BAh4py
- Execute setMaxConvert to set maximum amount
- convert dot to nDot

3. Deploy capital_stake contract

- Initiate contrct name as 'capitalStake'
- Initiate parameter signer = Your address，nsure= deployed nsure contract address，startBlock=current block number
- add by execute 'add(100,nDot 'contract address',true,100000000000000000)', to add stakable nDot token
- stake nDot

## License
Nsure dot contracts and all other utilities are licensed under [Apache 2.0](LICENSE).

## Contact
If you want any further information, feel free to contact us at **contact@nsure.network** :) ...


