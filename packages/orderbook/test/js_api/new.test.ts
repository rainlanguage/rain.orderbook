import assert from 'assert';
import { describe, it } from 'vitest';
import { TestStruct } from '../../dist/cjs/js_api.js';
import { CustomError } from '../../dist/types/js_api';

describe('TestStruct', () => {
	it('should be able to call simpleFunction', () => {
		const result = TestStruct.simpleFunction();
		assert.equal(result.data, 'Hello, world!');
	});

	it('should be able to call errFunction', () => {
		let result = TestStruct.errFunction();
		if (result.data) {
			assert.fail('result.data should be undefined');
		}
		let error = {
			msg: 'JavaScript error: some error',
			readableMsg: 'JavaScript error: some error'
		} as CustomError;
		assert.deepEqual(result.error, error);
	});

	it('should be able to call simpleFunctionWithSelf', () => {
		let testStruct = TestStruct.new('beef');
		const result = testStruct.simpleFunctionWithSelf();
		assert.equal(result.data, 'Hello, beef!');
	});

	it('should be able to call errFunctionWithSelf', () => {
		let testStruct = TestStruct.new('beef');
		const result = testStruct.errFunctionWithSelf();
		if (result.data) {
			assert.fail('result.data should be undefined');
		}
		let error = {
			msg: 'JavaScript error: some error',
			readableMsg: 'JavaScript error: some error'
		} as CustomError;
		assert.deepEqual(result.error, error);
	});
});
