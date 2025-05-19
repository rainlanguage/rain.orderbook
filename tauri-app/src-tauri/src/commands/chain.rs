use crate::error::CommandResult;
use alloy_ethers_typecast::transaction::ReadableClientHttp;

#[tauri::command]
pub async fn get_chainid(rpc_url: String) -> CommandResult<u64> {
    let chain_id = ReadableClientHttp::new_from_url(rpc_url)?
        .get_chainid()
        .await?;

    let chain_id_u64: u64 = chain_id.try_into()?;

    Ok(chain_id_u64)
}

#[tauri::command]
pub async fn get_block_number(rpc_url: String) -> CommandResult<u64> {
    let block_number = ReadableClientHttp::new_from_url(rpc_url)?
        .get_block_number()
        .await?;
    Ok(block_number)
}

#[cfg(test)]
mod tests {
    use alloy_ethers_typecast::transaction::ReadableClientError;
    use httpmock::prelude::*;
    use serde_json::json;

    use super::*;
    use crate::error::CommandError;

    #[tokio::test]
    async fn test_get_chainid_ok() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/rpc-1");
            let res_body = json!({ "jsonrpc":"2.0", "id":1, "result": "0x1" });
            then.status(200).body(res_body.to_string());
        });

        server.mock(|when, then| {
            when.path("/rpc-fe");
            let res_body = json!({ "jsonrpc":"2.0", "id":1, "result": "0xfe" });
            then.status(200).body(res_body.to_string());
        });

        let rpc_url = server.url("/rpc-1");
        let chain_id = get_chainid(rpc_url).await.unwrap();
        assert_eq!(chain_id, 1);

        let rpc_url = server.url("/rpc-fe");
        let chain_id = get_chainid(rpc_url).await.unwrap();
        assert_eq!(chain_id, 0xfe);
    }

    #[tokio::test]
    async fn test_get_chainid_err() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/rpc-1").body_contains("eth_chainId");
            let res_body = json!({ "jsonrpc":"2.0", "id":1, "result": 1 });
            then.status(200).body(res_body.to_string());
        });

        let rpc_url = server.url("/rpc-1");
        let err = get_chainid(rpc_url).await.unwrap_err();
        assert!(matches!(
            err,
            CommandError::ReadableClientError(ReadableClientError::ReadChainIdError(msg))
            if msg.contains("Deserialization Error: invalid type")
        ));

        server.mock(|when, then| {
            when.path("/rpc-2");
            then.status(404);
        });

        let rpc_url = server.url("/rpc-2");
        let err = get_chainid(rpc_url).await.unwrap_err();
        assert!(matches!(
            err,
            CommandError::ReadableClientError(ReadableClientError::ReadChainIdError(msg))
            if msg.contains("Deserialization Error: EOF")
        ));

        server.mock(|when, then| {
            when.path("/rpc-3").body_contains("eth_chainId");
            let res_body = json!({ "jsonrpc":"2.0", "id":1, "result": "0xyz" });
            then.status(200).body(res_body.to_string());
        });

        let rpc_url = server.url("/rpc-3");
        let err = get_chainid(rpc_url).await.unwrap_err();
        assert!(matches!(
            err,
            CommandError::ReadableClientError(ReadableClientError::ReadChainIdError(msg))
            if msg.contains("Deserialization Error: invalid hex character: y")
        ));
    }

    #[tokio::test]
    async fn test_get_block_number_ok() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/rpc-1").body_contains("eth_blockNumber");
            let res_body = json!({ "jsonrpc": "2.0", "id": 1, "result": "0x15536ee" });
            then.status(200).body(res_body.to_string());
        });

        let rpc_url = server.url("/rpc-1");
        let block_number = get_block_number(rpc_url).await.unwrap();
        assert_eq!(block_number, 0x15536ee);

        server.mock(|when, then| {
            when.path("/rpc-2").body_contains("eth_blockNumber");
            let res_body = json!({ "jsonrpc": "2.0", "id": 2, "result": "0xabcdef" });
            then.status(200).body(res_body.to_string());
        });

        let rpc_url = server.url("/rpc-2");
        let block_number = get_block_number(rpc_url).await.unwrap();
        assert_eq!(block_number, 0xabcdef);
    }

    #[tokio::test]
    async fn test_get_block_number_err() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/rpc-1").body_contains("eth_blockNumber");
            let res_body = json!({ "jsonrpc": "2.0", "id": 1, "result": null });
            then.status(200).body(res_body.to_string());
        });

        let rpc_url = server.url("/rpc-1");
        let err = get_block_number(rpc_url).await.unwrap_err();
        assert!(matches!(
            err,
            CommandError::ReadableClientError(ReadableClientError::ReadBlockNumberError(msg))
            if msg.contains("Deserialization Error: invalid type: null")
        ));

        server.mock(|when, then| {
            when.path("/rpc-2").body_contains("eth_blockNumber");
            let res_body = json!({ "jsonrpc": "2.0", "id": 2, "error": { "code": -32000, "message": "Internal error" } });
            then.status(200).body(res_body.to_string());
        });

        let rpc_url = server.url("/rpc-2");
        let err = get_block_number(rpc_url).await.unwrap_err();
        assert!(matches!(
            err,
            CommandError::ReadableClientError(ReadableClientError::ReadBlockNumberError(msg))
            if msg.contains("message: Internal error")
        ));

        server.mock(|when, then| {
            when.path("/rpc-3").body_contains("eth_blockNumber");
            let res_body = json!({ "jsonrpc":"2.0", "id":1, "result": "0xyz" });
            then.status(200).body(res_body.to_string());
        });

        let rpc_url = server.url("/rpc-3");
        let err = get_block_number(rpc_url).await.unwrap_err();
        assert!(matches!(
            err,
            CommandError::ReadableClientError(ReadableClientError::ReadBlockNumberError(msg))
            if msg.contains("Deserialization Error: invalid hex character: y")
        ));
    }
}
