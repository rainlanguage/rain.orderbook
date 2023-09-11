import { setOrderbookAddress } from '$lib/stores/orderbook'
import { setSubgraphClient } from '$lib/stores/subgraph'
import type { OrderbookComponentsConfig } from '$lib/types/config'
import { getAddress, isAddress } from 'viem'

export * from './orderbook'
export * from './subgraph'
export * from './token'

/**
 * Initialize this package.
 * After this the Orderbook contract instance will be available in the $orderbook store.
 * The subgraph client will be available in the $subgraphClient store.
 * @param config - Configuration object
 */
export const initOrderbook = async ({ address, subgraphEndpoint }: OrderbookComponentsConfig) => {
    if (!isAddress(address)) {
        console.warn(`Invalid address ${address}`)
        return
    }
    const _address = getAddress(address)
    if (!await checkOrderbookAddress(address, subgraphEndpoint)) return
    setOrderbookAddress(_address)
    setSubgraphClient(subgraphEndpoint)
}

/**
 * Check that the Orderbook address being used is actually being indexed by the subgraph endpoint.
 * @param orderbookAddress - Address of the Orderbook contract
 * @param subgraphEndpoint - Endpoint of the subgraph
 * @returns True if the Orderbook address is being indexed by the subgraph endpoint
 */
export const checkOrderbookAddress = async (orderbookAddress: string, subgraphEndpoint: string) => {
    const query = `
        query {
            orderBooks(where: {address: "${orderbookAddress}"}) {
                id
            }
        }
    `
    const response = await fetch(subgraphEndpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query })
    })
    const json = await response.json()

    if (!json.data?.orderBooks?.length) {
        console.warn(`No orderbook found at address ${orderbookAddress} on subgraph ${subgraphEndpoint}`)
        return false
    }
    return true
}

/**
 * Get the address of the first orderbook found on the subgraph endpoint.
 * @param subgraphEndpoint - Endpoint of the subgraph
 * @returns Address of the orderbook contract
 */
export const getOrderbookAddress = async (subgraphEndpoint: string): Promise<{ orderbookAddress?: string, error?: string }> => {
    const query = `
        query {
            orderBooks(first: 1) {
                address
            }
        }
    `
    try {
        const response = await fetch(subgraphEndpoint, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ query })
        })
        const json = await response.json()

        if (!json.data?.orderBooks?.length) {
            return { error: `No orderbook found on subgraph ${subgraphEndpoint}` }
        }
        return { orderbookAddress: json.data.orderBooks[0].address }
    } catch (error: any) {
        return { error: error?.message || 'Something went wrong with querying the endpoint' }
    }
}