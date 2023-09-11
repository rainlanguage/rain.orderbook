import type { TokenVaultsQuery } from '$lib/gql/generated/graphql'
import type { Token } from '$lib/types'
import type { Address } from 'viem'

/**
 * Generate a vaultId.
 * This can be used as a unique identifier for a vault, and is a random 32 byte hex string.
 */
export const generateVaultId = (): bigint => {
    let randomBigInt = BigInt(0);
    for (let i = 0; i < 32; i++) {
        let randomByte = Math.floor(Math.random() * 256);
        randomBigInt = (randomBigInt << BigInt(8)) | BigInt(randomByte);
    }
    return randomBigInt;
}

/**
 * @description - Convert a vault from a subgraph query to a Token type.
 * @param vault - The vault to convert.
 * @returns - The converted vault.
 */
export const vaultQueryToToken = (vault: TokenVaultsQuery['tokenVaults'][0]): Token => {
    return {
        name: vault.token.name,
        symbol: vault.token.symbol,
        decimals: vault.token.decimals,
        address: vault.token.id as Address
    }
}