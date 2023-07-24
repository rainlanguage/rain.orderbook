use crate::utils::utils::get_web3;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use web3::ethabi::{self, Token};
use web3::types::{
    BlockId, BlockNumber, Bytes, TransactionReceipt, TransactionRequest, H256, U256,
};
use web3::{
    contract::{Contract, Options},
    transports::Http,
    types::H160,
};

pub async fn deploy_touch_deployer() -> anyhow::Result<()> {
    let deployer = get_web3().eth().accounts().await?[0];
    let rainterpreter = rainterpreter_deploy(deployer).await?;
    let rainterpreter_store = rainterpreter_store_deploy(deployer).await?;
    let expression_deployer = rainterpreter_expression_deployer_deploy(
        rainterpreter.address(),
        rainterpreter_store.address(),
        deployer,
    )
    .await?;
    Ok(expression_deployer)
}

pub async fn rainterpreter_deploy(deployer: H160) -> anyhow::Result<Contract<Http>> {
    let mut json = String::new();
    let mut file = File::open("tests/utils/deploy/touch_deployer/Rainterpreter.json")?;
    file.read_to_string(&mut json)?;

    let json: Value = serde_json::from_str(&json)?;
    Ok(basic_deploy(
        json["abi"].to_string(),
        json["bytecode"]["object"].to_string(),
        deployer,
    )
    .await?)
}

pub async fn rainterpreter_store_deploy(deployer: H160) -> anyhow::Result<Contract<Http>> {
    let mut json = String::new();
    let mut file = File::open("tests/utils/deploy/touch_deployer/RainterpreterStore.json")?;
    file.read_to_string(&mut json)?;

    let json: Value = serde_json::from_str(&json)?;
    Ok(basic_deploy(
        json["abi"].to_string(),
        json["bytecode"]["object"].to_string(),
        deployer,
    )
    .await?)
}

pub async fn basic_deploy(
    abi: String,
    bytecode: String,
    deployer: H160,
) -> anyhow::Result<Contract<Http>> {
    let contract = Contract::deploy(get_web3().eth(), abi.as_bytes())?;
    let contract = contract
        .confirmations(0)
        .options(Options::with(|opt| opt.gas = Some(30_000_000.into())))
        .execute(bytecode, (), deployer)
        .await?;
    Ok(contract)
}

pub async fn rainterpreter_expression_deployer_deploy(
    interpreter: H160,
    store: H160,
    deployer: H160,
) -> anyhow::Result<()> {
    let provider = get_web3();
    let mut json = String::new();
    let mut file =
        File::open("tests/utils/deploy/touch_deployer/RainterpreterExpressionDeployer.json")?;
    file.read_to_string(&mut json)?;

    let json: Value = serde_json::from_str(&json)?;
    let abi = json["abi"].to_string();
    let bytecode: String = json["bytecode"]["object"].to_string();
    let meta = get_rain_meta_document_from_opmeta()?;

    let constructor_inputs = ethabi::encode(&[
        Token::Address(interpreter),
        Token::Address(store),
        Token::Bytes(meta),
    ]);

    let constructor_inputs = format!("0x{:?}", hex::encode(constructor_inputs.clone()));
    let contract = Contract::deploy(provider.eth(), abi.as_bytes())?;
    let contract = contract
        .confirmations(0)
        .options(Options::with(|opt| {
            opt.gas = Some(30_000_000.into());
        }))
        .execute(bytecode, constructor_inputs, deployer)
        .await?;

    // let tx = TransactionRequest {
    //     from: deployer,
    //     data: Bytes(constructor_inputs).into(),
    //     value: U256::from("80000000000000000").into(),
    //     ..Default::default()
    // };

    // let trx: H256 = provider.eth().send_transaction(tx).await?;
    // let receipt: TransactionReceipt = provider.eth().transaction_receipt(trx).await?.unwrap();
    // let expression_deployer = receipt.contract_address.unwrap();
    // let contract = Contract::from_json(provider.eth(), expression_deployer, abi.as_bytes())?;
    let store: H160 = contract
        .query("store", (), None, Options::default(), None)
        .await?;
    println!("store {}", store);
    Ok(())
}

fn get_rain_meta_document_from_opmeta() -> anyhow::Result<Vec<u8>> {
    let meta_string = "ff0a89c674ee7874a40059110d789ced5d6d6fdc3612fe9e5f4118012ee9ad1cefba497cfdd6a6f7125c9b14692e87439a03b8127797b0246e44c9f6f6d0ff7e3343ea5dd46adffc86fd90d8962871f8cc33c32139a43e3f61ec7ff08fb1939847e2e43b7612085f05c29bbc7c7532327702a17dbcf323dd619ce950fa82a9199bca543395a5f83b67f0085e61573ccc047bc6e355ba90f19ca998a50bc174cafdcbe7a7f95bd552243c0ee0c59fe9422e484d18782649ed1335597e807a641c881b962a46a558227880d5cd12159d569f4129b19a8b111bbffc62afff3172d71a8a789e2ebaaa7d97455391144d87bab156aad248d159f1d988bd2eeaa59f5f2c08325e66542697e264c91390221589ae205395b28a8eca125f9c14b7feb0bf99bafec881ce525bc9d85ee1a1e45a500527a44bd2363e45cf340821621721fe4a77985691606992c53e4f456090214406520214094072168bebfa0347ae1c902ba3fef220e05ca4bb71cb57cb9537806037cb1019763e6912ec17d4158f59060c4166181a115d2e9816c88a54b0319baee08796bf0b53427710e76c2f28d2eb878372d1018a6d2d36b668ba131a7fc1651ccaf8d25b260063139f8ffc5268f637015607ad64bfa63c14dfcfa0291687826cfe2515008d6a32bc37f97bd9fb84fba1a077fc4933aa858a9a87b06c227416a66c0a2d08d070cb5a0e85f20c84a9d846a5c13c08401c8d4685a21971a9f860626b14dfe315f91b750419b04a4243b11289ae6dc1535bd5b50c4336259ac96037e3300a752a5ec5a9b8499b0a7f1b6b91800ec1266c09e60b10894ca2f0a8600f8587f355984531a93451d78610b9be36f1aee63d3bb94aa87f4b8f77d646b41f37cf88eb2db85e3431fc1521326680b7297689998853998812557afe94fd4b8b5916b2994a988ef9522f549a22ac8867c8d1d2fc85f02fe9120ba5a65028032581c1aa2b1980c9385dd256689fd5d0de175ca56ada541bca34bc68d95665599d7af4d48e50e412d675759b9d29c235dcf81dd8cf5418780e3bff452440b908d10f15c014309f03f8ea0a0847e19691df786afc3bc70224d34c2f852f671263b155ae1213f79830089fba9641bad88c9416b30e9cc7e75bfa808a4d2664574814201836da1d9c8dd1082643aaa356f60767061f5dd48a1877d6fa7ac4fe32a44eab57176b8ba65ac5acadf7db117b35a45e13e179a4e2ae8adf92ee6dcf690a53509ac5ae40f47ca8ed347890bf6d239ba2b7d67b64bd44df023761702106db9b4baa27f9c39d8e10ccab3bba32e6968ac84653a5ef23c796c568a416ce6b992e98cea65e5980ccb40ca334c4511857557ce866839c354a964e25bbbaeef1106e19743d0cb1bb6a8d0b6bcad5d0cde3d743ea72bb996d2849cfdc33429648e485ab81e10c06d2187fe2f860163b23c44079d70b19b6c6041f0c23e99ef52c312b1564989c02b5e37c906ec89a4100149ade3e9058bf66523300eb21f0730d672ad72972ffc0e3393ef7d98653d502be8ae05d3402a0114732671e1b9fec44bbc63b693ee0cff0d203f372cd48a8d074f738a8b88dea2ae9343cf0e906c5c97bc366f72401a01337c93e6004da0e86cf1c5560ebbc56153fa109151e3ea621a1c6c00a6c08bbf166ff5073fd691e18d7bc3fc5d09bfb7e926fbd0bc662dd51da64689476ec651e7d2f43a68c3d8c4a9c36876278918854b2ea8e8d60dc8ccc37c4077f019c8e715492247cc560c85c3108632168169206968c2665b78e8386519384b313af9d1631d420d2d5b29392783daf0c2b1935a1c079834ae39aa439db7c24dfa14b6a9e4b895d5320ffc0798f675a415728d315bb14becf2f272f5f3db733ac95f0a07467cd19c13d8d1d698ceeeea041459397d0436f6e90afefd8205de37e91f8de78fcf2a537e5218fc1cdaa59533f7f1729016d4bd829aabf7e7883cf01a1513be61af77d0591db8686b59f695a92c3b353b2c32760adc81b2c45503dbbceb7366177471a6dfdc0afa9dfb2a2869674a79a70e6909eb6f74de38d27342564b057bfb8a39db923e16736147ece5eb049773cdc1ffb357aa3a1c6db1483a2e76fd8e4f9f621f48eccdd43ec5d707a6b47e3e036306d10c32767dbf89fc9d9e3f33e5bfa9212c07538a72ae5a1a7b3e5326cc55139d2e66e07d0f709daed91b28d5f0354bec252f5bcbc35375e40664b6fca52c4e0eeb1dc77275960b78f7eb2431783755765fb10ed3d08e6df15f803ece6e5f9f8dc53d771b91c5f5be824cce87609313ec2b05b859ae394d68e12eea707cb9ac82bd8c2a5f434fcf564bc4d17068f1dfb304bb50a84bd1cc372c4a11ea01b1c7b0430ef6fdc51e0d79364a7b3a43d972fbe66989590e22cfeacb2f404a84d29e72e5dac464c9e8a533687d011f311d205a0fdbb48d429fb20ae68095fcef237d8b918cdce0e66ec115f4d8567441b8edc5907728969bd13b39cb90dd0de6449228069310c19ae049b73dd7002968a0743a07398504c9c810e735b002dce5b6eea7477ef380d957fe999e952575a07656b58a4a87cbe46b4b1490e9ba3b2757954975ba524394db1a73c5a760bdf217bf1c460f1dbd2b61a54153fc6bc0556b402eb6bfe6d2476b50cbc4fb335df0730f28fb23095cb30cf5c044e0025162239659fcc051f180bb62e245e6581f065c4439c5784968ab948466c9a01d92192e22c9234f53955fb4bed58336d3079b8d3731d5ad659845a05643dd21765c323dee38bfc02deb3c5ca7b78c1b94e2baf5a89f41233c32abab7aacc39a0c1f1613e1a0fc3d5518f5be8119584b8f7641cb7ba5a2e758f4aa01359aa6b1c39827d09ee2f98427b1c31713a3f6537ecbf6c05ff7e3f6a6b4b6d01b84e6d45fca635b411a1f06d2f06b7659445a89886333dfad03bd22669acea20f18253bdb2b5e65e53af8c8feabd67ea957143bdd29da714a956e0f386877e86d9d3e06f559085aaa6dcbae73d2a695b2529f79a3060ddd4c9cf067e093a797754c19e540030bb54a0b369cbeb65539a26430d1c3ddc9d6a8eb4538ff0a76e55fab8a3687cd19ebda13ba09299bc1101049012a7258a2ca677b92229f97c7c51fe790d5a1668871a146c66d78c02375368febeaecc951ff3baecbc9249fc980a4ac23782079d79e1935a8a779b0c9851f9eafce2bca4c3c82da1e629eeb6eaccad793b63638682a048d9d2ecbdca1f60f0df229fedc2d4f959a8aed992ebb4d82c083147f7ee0110708dfc83644f5416075eb65c2b7ba0ae63233d3d826da94a4fb755cc80c0bed4986f192aad5d39f96783455f638507dcf6e89a97ca0dc50b567049fa0731984aee52310db4e146e223291f0229f3b9b7c2cb95fc1cdd229d5b839782c53152f4fdfb9f75179d3103988abddb8c9c665fb6d7e7da718ad914abd98ba92e5547af7e34a08a19d4a2a82a856c2a68491cf66e779be1712b05a6ba22319389c63836f67005cb8a80ab36007311a550f6066dbb354b5be4fbc58dd4986f1b8b1113a116769151d352586355e018166f1116f7e4890b0e43104fb696864d92f8b97bf71c29d1ead0ea9e52b6a11b2f384079dc158a68811b536cb99a5a1b8ac7ebe7892979c0cc86be2d341f694b74be8d0664154860f48c83fb2a707e1ab713422344df9a6269ad66897766f763e79557207d815c182ec18ccb704f1250ed33e88f763cb406a8e6e4e2d78c875eaa5a734e2a02c50a438b107b9489e5656361dd2cbf55f21434d9ce8b7165db8e6ddc8a71f43cd9571e36b8476d7c717628da11d4e30d039ec98e790c5fdd98434f3dc0ad0ff2e6b4cd9dd08d08dd9cb447af5eb9b53faf8e38b9d46a334a3c8c67ba5dbbb5a3eec366cccee78ae9945656f3f8f6501f5dcf60a9b87a53005f69cd0a9f8fd5e11238eec2bce6a9530f529305b43c5a35022a5124cb7079af9a0a30ff4adf81e3dad1f76bcff4202378ad72a3160ae8386f8bba58d93e78db9395728f985cdb47e8d6c032997afc6adeb3fa8360f144a60b688ff419877e044249f4ffe69092d50817decb899eea407a64c65df950ec60cee066387a1be4bb7531bb926b02a8f5a1ea0bd95ac0f98f1461604364780b303265d70b0523b76efcf289889ad75509a3000ac3b89b5b8174279c08873ea07a926fd26bd58f8c3e651f9061387972cd93c0b8d647c5b3be1c1984af234fa661bd31cddc8430fc5ec2d03b4eadedba8cf6fe530a9bbc0692d631850d4ca632e6c9aa0149a6f3a300eced192ef5e1b8089ab850c103c76cd20bda2c54aa95895a7158c6070d7458655ffbc0bc9541a11727e0440f4ce2c6c7b64fc50ac30b0a43f0950fdee6a8d97db8cca3352637170aa44e208688048f870510266dfee953fd35499fddb06fd8eaf9d3a78f39aa98bb9354116419b7faca0aca63f682dd0c0bc74c77797bbde54ea060abfb50095be307470f18aab909651fbc3986ee642e4244cdc7676b40f15514d192c7a3c1841abd06968181c1a342a5bfe3ef4f345b1f8297db211e9323eecb0b43d496edd3586d7afa4d7e722025a3237f5677cc9fdbc2ac2f41dc629639417b66d6e2dd447b5ec775c59e65b196f3188f50e51ac20a9b67f7bc12c7cf78a432cd783857c69a7f3bc9437e690e909eae9886f83481277e3b796cda809eb37fea07e3aa350e91d011103928334aba67d33d3b4144cd77a60c9a8407688dd7b1312c5f3ad126ed9d902a8ef4752ea6ca19261b5c89048f97492a5b3fcb33dce70ab7b24cd515ed9a283eb6503970edb878b281b2418bf9b6b08a42edee30bcd9b72facf244472f995320ca7bcb20d7e19110f79b10bd49df5596b4f2bf4bb33779e0f93708f6abed29a65455d7938e0ade4cc17da9e02a09443255ead2bb9a78571c402fce576828fb63eb6c8af7f8ec0ff0eca709ce6c957f8e19bde9605d5e29747e44420dbddacef5faf737de56641e1e8598932436adeb7d7ebe8459edc4030807d7583f0062688d1ff1a94dd6ce50ddbb1e175161d0b8c120b7570905f22d111197313a177b72457c05b168d7e19de4126c61967cfce73b5cc19cd18104e66409adc2808eaee3ec5778fba7c9c1c847c2efed5c98120a70a5579d46676f550e2779984da49f989695e9ae66e26d666edfe74676fa58db40d3b675ac273d762a1a785d35e9fb0ac1daf6e1795a96b5f0d317f24ab4860c94f38d25198e4acda2538de9181398477158fac0b0806046ce56e811edd129961c1ecf8fe2e8308092fb95d38770839c0da67cf375277b08fe27aae2d3f8369333ca566d7902d416671a150795ac514ad5225df0bb33c7446be4ff01bf7dc7d9a558bd3079314b2e93e2b4648a7741658af23f80a0e6eb49f954cb25a65ad68e20aee6aeef9dbc505d779090cb6203847a63f660f26dd8fe9d40c0dfc6adf380e66b2a4bf94a4d480f89d40601d246c95f5dc7f050139db44b25c495d4fb2f557bee099dc2472801e1b52960029cc2398c985ae2980b4f5da9f02fffb2d2edee85fe32629fcff1bff1d9faaf53384ee3ddf2805dfb758a4921f66667adc94a6cbfa93f1bfe40fdbb59f66623ed1e0b607680d1dc5e86848388475d9207e4f2f07a0f0d4318a587f51eca414200549acb77c347a2e2b7c4c7f17a3e9e1f828fe73bf01154730bc794a3b2ef3b813b7c2a21d4c9dc757eb6329d15c8597bfb0f0d43f2220c8b8844d8098fdc091fee5badb6491b64bfda2736c9e45e87101d64e385a96b286a67744db93cdc29339c8af4a67202d0f5858648c1d3e55e698bae09f64335c7f45a881122fc60f30ee7cbd39bbaf6469a163068a9a9ed3b46bb421875b12bd67aaef671b6f27b2b2377d528fa9a9ab1c8777862d088a6b3f10b2498a7dd55ef057ecdcfedf0c0234d06ed177d381fb6488435bfd6f98a0d40692c591ca888d1246dd34be932b02ba7169f733c4e7b53fb72576ac332a43be9f4a7d40ccf6ea3ebce96014f2d348507f4125297639e3ead9930bda0f8dc133e673e206ccf8ce345eb9a964b16dab2eaca8e83edad55c4413506a9b582120a6dc296341bd20d065d5cfeb69fcb40ca8b613bc3f1e397bd22e9fcf398eb855aff69b78b83ec9beea3715aa3b1919dd1e7dfc97248b8c1f652e5a4db5cd3a6b9da5acb1ae1e2865dda932f4ffe0fcd42f13b011bffe5282f43e495b402706170706c69636174696f6e2f6a736f6e03676465666c617465";
    Ok(hex::decode(meta_string.clone())?)
}
