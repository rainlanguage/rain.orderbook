export type DepositConfig = {
    orderbook: string; // address of the orderbook

    tokenAddress: string; // address of the token to deposit

    tokenDecimals: number; // decimals corresponding to the token

    amount: string; // amount to deposit

    vaultId?: string; // optional vault id to deposit in (in decimals)

    addressIndex?: number; // address index of the wallet to accessed. default 0.

    rpcUrl: string; // mumbai rpc url, default read from env variables

    blocknativeApiKey?: string; // blocknative api key for gas oracle
};
