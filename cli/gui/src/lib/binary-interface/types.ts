export enum RainNetworkOptions {
    Ethereum,
    Polygon,
    Mumbai,
    Fuji
}

export type DepositConfig = {
    network: RainNetworkOptions;  // Assuming RainNetworkOptions has an equivalent in TypeScript
    orderbook: string;
    tokenAddress: string;  // Converted snake_case to camelCase
    tokenDecimals: number;
    amount: string;
    vaultId?: string;  // Converted Option<T> to T | undefined
    mumbaiRpcUrl?: string;  // Converted snake_case to camelCase
    polygonRpcUrl?: string;  // Converted snake_case to camelCase
    ethereumRpcUrl?: string;  // Converted snake_case to camelCase
    fujiRpcUrl?: string;  // Converted snake_case to camelCase
    blocknativeApiKey?: string;  // Converted snake_case to camelCase
};