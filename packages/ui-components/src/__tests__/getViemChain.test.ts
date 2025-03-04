import { getViemChain } from '../lib/services/getViemChain';
import { describe, it, expect } from 'vitest';

describe('getViemChain', () => {
    it('should return the chain for a given chainId', () => {
        const chain = getViemChain('mainnet');
        expect(chain).toBeDefined();
    });

    it('should return the chain for a given chainId', () => {
        const chain = getViemChain('fakeChainThatDoesNotExist');
        expect(chain).not.toBeDefined();
    });
});