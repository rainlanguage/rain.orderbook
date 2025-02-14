import assert from 'assert';
import { getLocal } from 'mockttp';
import { describe, it, beforeEach, afterEach } from 'vitest';
import { Transaction } from '../../dist/types/js_api.js';
import { getTransaction } from '../../dist/cjs/js_api.js';

const transaction1 = {
	id: 'tx1',
	from: '0x1',
	blockNumber: '1',
	timestamp: '1'
} as unknown as Transaction;

describe('Rain Orderbook JS API Package Bindgen Tests - Transaction', async function () {
	const mockServer = getLocal();
	beforeEach(() => mockServer.start(8093));
	afterEach(() => mockServer.stop());

	it('should fetch a single transaction', async () => {
		await mockServer
			.forPost('/sg1')
			.thenReply(200, JSON.stringify({ data: { transaction: transaction1 } }));

		try {
			const result: Transaction = await getTransaction(mockServer.url + '/sg1', transaction1.id);
			assert.equal(result.id, transaction1.id);
		} catch (e) {
			assert.fail('expected to resolve, but failed' + (e instanceof Error ? e.message : String(e)));
		}
	});
});
