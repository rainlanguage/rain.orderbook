import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Page from './+page.svelte';
import { keccak256HexString } from "@rainlanguage/orderbook/common";

describe('Page Component', () => {
	it('should load the page', async () => {
		render(Page);
		expect(screen.getByTestId('page-container')).toBeInTheDocument();
	});
	it('should load orderbook package', async () => {
		const result = keccak256HexString("0x1234");
		const expected = "0x56570de287d73cd1cb6092bb8fdee6173974955fdef345ae579ee9f475ea7432";
		expect(result).equals(expected);
	});
	
});
