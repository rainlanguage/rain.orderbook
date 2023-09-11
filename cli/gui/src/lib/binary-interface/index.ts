import type * as types from './types'

class BinaryInterface {
    public uri = 'http://localhost:8080';

    constructor(_uri?: string) {
        if (_uri) this.uri = _uri;
    }

    public async deposit(config: types.DepositConfig): Promise<Response> {
        const response = await fetch(`${this.uri}/api/deposit`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(config)
        });

        if (!response.ok) {
            // Handle HTTP errors here if needed or just throw
            throw new Error(`HTTP error! Status: ${response.status}`);
        }

        return response;
    }
}