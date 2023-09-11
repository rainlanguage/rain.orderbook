import type { Address } from 'viem'
import { isAddress } from 'viem'
import { fetchToken as wagmiFetchToken } from "@wagmi/core"
import type { Token } from '$lib/types'

/**
 * @description A wrapper around Wagmi's fetchToken that returns a Token type. Also checks for a valid address.
 * @param address - The address of the token to fetch.
 * @returns - The fetched token.
 */
export const fetchToken = async (address: string): Promise<{ isValid: true, token: Token } | { isValid: false }> => {
    if (!isAddress(address)) return { isValid: false }
    const token = await wagmiFetchToken({ address })
    return {
        isValid: true,
        token: {
            name: token.name,
            symbol: token.symbol,
            decimals: token.decimals,
            address: token.address
        }
    }
}
