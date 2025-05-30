import assert from 'assert';
import { getLocal } from 'mockttp';
import { describe, it, beforeEach, afterEach } from 'vitest';
import { getTransaction, SgTransaction } from '../../dist/cjs';

const transaction1 = {
	id: 'tx1',
	from: '0x1',
	blockNumber: '1',
	timestamp: '1'
} as unknown as SgTransaction;

describe('Rain Orderbook JS API Package Bindgen Tests - SgTransaction', async function () {
	const mockServer = getLocal();
	beforeEach(() => mockServer.start(8093));
	afterEach(() => mockServer.stop());

	it('should fetch a single transaction', async () => {
		await mockServer
			.forPost('/sg1')
			.thenReply(200, JSON.stringify({ data: { transaction: transaction1 } }));

		const result = await getTransaction(mockServer.url + '/sg1', transaction1.id);
		if (result.error) assert.fail('expected to resolve, but failed');
		assert.equal(result.value.id, transaction1.id);
	});
});
