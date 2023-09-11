export * from './vaults.js'
export * from './token.js'
export * from './expressions.js'

/**
 * @description Shorten a hex string greater than 9 characters, e.g. 0x1234567890abcdef => 0x1234...cdef
 * @param {string} hexString
 * @returns {string}
 */
export const shortenHexString = (hexString: string): string => {
    if (hexString.length <= 9) return hexString
    return `${hexString.slice(0, 6)}...${hexString.slice(-4)}`
}
