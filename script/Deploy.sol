// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {Script} from "forge-std/Script.sol";
import {OrderBook} from "src/concrete/ob/OrderBook.sol";
import {OrderBookSubParser} from "src/concrete/parser/OrderBookSubParser.sol";
import {GenericPoolOrderBookV3ArbOrderTaker} from "src/concrete/arb/GenericPoolOrderBookV3ArbOrderTaker.sol";
import {RouteProcessorOrderBookV3ArbOrderTaker} from "src/concrete/arb/RouteProcessorOrderBookV3ArbOrderTaker.sol";
import {GenericPoolOrderBookV3FlashBorrower} from "src/concrete/arb/GenericPoolOrderBookV3FlashBorrower.sol";
import {EvaluableConfigV3, IExpressionDeployerV3} from "rain.orderbook.interface/interface/IOrderBookV3.sol";
import {OrderBookV3ArbConfigV1} from "src/abstract/OrderBookV3ArbCommon.sol";
import {IMetaBoardV1} from "rain.metadata/interface/IMetaBoardV1.sol";
import {LibDescribedByMeta} from "rain.metadata/lib/LibDescribedByMeta.sol";

bytes32 constant DEPLOYMENT_SUITE_ALL = keccak256("all");
bytes32 constant DEPLOYMENT_SUITE_RAINDEX = keccak256("raindex");
bytes32 constant DEPLOYMENT_SUITE_SUBPARSER = keccak256("subparser");
bytes32 constant DEPLOYMENT_SUITE_ROUTE_PROCESSOR = keccak256("route-processor");
bytes32 constant DEPLOYMENT_SUITE_ARB = keccak256("arb");

/// @dev Exact bytecode taken from sushiswap deployments list in github.
/// https://github.com/sushiswap/sushiswap/blob/master/protocols/route-processor/deployments/ethereum/RouteProcessor4.json#L406
///
/// Cross referenced against deployment on etherscan.
/// https://etherscan.io/address/0xe43ca1dee3f0fc1e2df73a0745674545f11a59f5#code
///
/// Includes constructor args found on etherscan which translate to `address(0)`
/// for the bento (i.e. no bento) and no owner addresses.
bytes constant ROUTE_PROCESSOR_4_CREATION_CODE =
    hex"60a06040526002805461ffff60a01b191661010160a01b1790553480156200002657600080fd5b50604051620040483803806200404883398101604081905262000049916200016e565b6200005433620000eb565b6001600160a01b038216608052600280546001600160a01b031916600117905560005b8151811015620000e25760018060008484815181106200009b576200009b62000257565b6020908102919091018101516001600160a01b03168252810191909152604001600020805460ff191691151591909117905580620000d9816200026d565b91505062000077565b50505062000297565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b80516001600160a01b03811681146200015357600080fd5b919050565b634e487b7160e01b600052604160045260246000fd5b600080604083850312156200018257600080fd5b6200018d836200013b565b602084810151919350906001600160401b0380821115620001ad57600080fd5b818601915086601f830112620001c257600080fd5b815181811115620001d757620001d762000158565b8060051b604051601f19603f83011681018181108582111715620001ff57620001ff62000158565b6040529182528482019250838101850191898311156200021e57600080fd5b938501935b82851015620002475762000237856200013b565b8452938501939285019262000223565b8096505050505050509250929050565b634e487b7160e01b600052603260045260246000fd5b60006000198214156200029057634e487b7160e01b600052601160045260246000fd5b5060010190565b608051613d486200030060003960008181610151015281816115060152818161247b015281816124e00152818161254a0152818161260f015281816126bc015281816127a0015281816128a70152818161295301528181612a220152612b340152613d486000f3fe6080604052600436106100d65760003560e01c80638456cb591161007f5780639a1f3406116100595780639a1f340614610200578063cd0fb7a714610220578063f2fde38b14610260578063fa461e331461028057600080fd5b80638456cb59146101ad5780638da5cb5b146101c257806393b3774c146101ed57600080fd5b80632c8958f6116100b05780632c8958f6146100f95780636b2ace871461013f578063715018a61461019857600080fd5b8063046f7da2146100e257806323a69e75146100f95780632646478b1461011957600080fd5b366100dd57005b600080fd5b3480156100ee57600080fd5b506100f76102a0565b005b34801561010557600080fd5b506100f7610114366004613579565b6103a8565b61012c6101273660046136f5565b6103ba565b6040519081526020015b60405180910390f35b34801561014b57600080fd5b506101737f000000000000000000000000000000000000000000000000000000000000000081565b60405173ffffffffffffffffffffffffffffffffffffffff9091168152602001610136565b3480156101a457600080fd5b506100f7610564565b3480156101b957600080fd5b506100f7610578565b3480156101ce57600080fd5b5060005473ffffffffffffffffffffffffffffffffffffffff16610173565b61012c6101fb36600461377c565b61067b565b34801561020c57600080fd5b506100f761021b36600461382f565b61089a565b34801561022c57600080fd5b5061025061023b366004613868565b60016020526000908152604090205460ff1681565b6040519015158152602001610136565b34801561026c57600080fd5b506100f761027b366004613868565b6108f8565b34801561028c57600080fd5b506100f761029b366004613579565b6109af565b60005473ffffffffffffffffffffffffffffffffffffffff163314806102d557503360009081526001602052604090205460ff165b610366576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152603060248201527f52503a2063616c6c6572206973206e6f7420746865206f776e6572206f72206160448201527f2070726976696c6567656420757365720000000000000000000000000000000060648201526084015b60405180910390fd5b600280547fffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffffff167501000000000000000000000000000000000000000000179055565b6103b4848484846109af565b50505050565b60025460009074010000000000000000000000000000000000000000900460ff16600114610444576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601860248201527f526f75746550726f636573736f72206973206c6f636b65640000000000000000604482015260640161035d565b6002547501000000000000000000000000000000000000000000900460ff166001146104cc576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601860248201527f526f75746550726f636573736f72206973207061757365640000000000000000604482015260640161035d565b600280547fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1674020000000000000000000000000000000000000000179055610519878787878787610b5d565b9050600280547fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff16740100000000000000000000000000000000000000001790559695505050505050565b61056c6111a3565b6105766000611224565b565b60005473ffffffffffffffffffffffffffffffffffffffff163314806105ad57503360009081526001602052604090205460ff165b610639576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152603060248201527f52503a2063616c6c6572206973206e6f7420746865206f776e6572206f72206160448201527f2070726976696c65676564207573657200000000000000000000000000000000606482015260840161035d565b600280547fffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffffff167502000000000000000000000000000000000000000000179055565b60025460009074010000000000000000000000000000000000000000900460ff16600114610705576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601860248201527f526f75746550726f636573736f72206973206c6f636b65640000000000000000604482015260640161035d565b6002547501000000000000000000000000000000000000000000900460ff1660011461078d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601860248201527f526f75746550726f636573736f72206973207061757365640000000000000000604482015260640161035d565b600280547fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff1674020000000000000000000000000000000000000000179055604051600090819073ffffffffffffffffffffffffffffffffffffffff8c16908b908381818185875af1925050503d8060008114610826576040519150601f19603f3d011682016040523d82523d6000602084013e61082b565b606091505b50915091508161083d57805181602001fd5b61084b898989898989610b5d565b92505050600280547fffffffffffffffffffffff00ffffffffffffffffffffffffffffffffffffffff167401000000000000000000000000000000000000000017905598975050505050505050565b6108a26111a3565b73ffffffffffffffffffffffffffffffffffffffff91909116600090815260016020526040902080547fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0016911515919091179055565b6109006111a3565b73ffffffffffffffffffffffffffffffffffffffff81166109a3576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201527f6464726573730000000000000000000000000000000000000000000000000000606482015260840161035d565b6109ac81611224565b50565b60025473ffffffffffffffffffffffffffffffffffffffff163314610a56576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152603e60248201527f526f75746550726f636573736f722e756e697377617056335377617043616c6c60448201527f6261636b3a2063616c6c2066726f6d20756e6b6e6f776e20736f757263650000606482015260840161035d565b6000808513610a655783610a67565b845b905060008113610af9576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152603960248201527f526f75746550726f636573736f722e756e697377617056335377617043616c6c60448201527f6261636b3a206e6f7420706f73697469766520616d6f756e7400000000000000606482015260840161035d565b600280547fffffffffffffffffffffffff00000000000000000000000000000000000000001660011790556000610b3283850185613868565b9050610b5573ffffffffffffffffffffffffffffffffffffffff82163384611299565b505050505050565b60008073ffffffffffffffffffffffffffffffffffffffff881673eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee14610c24576040517f70a0823100000000000000000000000000000000000000000000000000000000815233600482015273ffffffffffffffffffffffffffffffffffffffff8916906370a0823190602401602060405180830381865afa158015610bfb573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610c1f919061388c565b610c27565b60005b9050600073ffffffffffffffffffffffffffffffffffffffff871673eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee14610cf1576040517f70a0823100000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff86811660048301528816906370a0823190602401602060405180830381865afa158015610cc8573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610cec919061388c565b610d0a565b8473ffffffffffffffffffffffffffffffffffffffff16315b905087600080610d2e87604080518082019091528181528151909101602082015290565b90505b805160208201511115610e84576000610d508280516001018051915290565b90508060ff1660011415610d7a576000610d6983611372565b905083610d74578094505b50610e73565b8060ff1660021415610d9557610d90828d611437565b610e73565b8060ff1660031415610dac576000610d6983611457565b8060ff1660041415610dc157610d908261147d565b8060ff1660051415610dd657610d90826114a3565b8060ff1660061415610dec57610d908d836115a8565b6040517f08c379a0000000000000000000000000000000000000000000000000000000008152602060048201526024808201527f526f75746550726f636573736f723a20556e6b6e6f776e20636f6d6d616e642060448201527f636f646500000000000000000000000000000000000000000000000000000000606482015260840161035d565b610e7c836138d4565b925050610d31565b506000905073ffffffffffffffffffffffffffffffffffffffff8b1673eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee14610f4d576040517f70a0823100000000000000000000000000000000000000000000000000000000815233600482015273ffffffffffffffffffffffffffffffffffffffff8c16906370a0823190602401602060405180830381865afa158015610f24573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610f48919061388c565b610f50565b60005b905083610f5d8b8361390d565b1015610feb576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602f60248201527f526f75746550726f636573736f723a204d696e696d616c20696e70757420626160448201527f6c616e63652076696f6c6174696f6e0000000000000000000000000000000000606482015260840161035d565b600073ffffffffffffffffffffffffffffffffffffffff8a1673eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee146110b3576040517f70a0823100000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff89811660048301528b16906370a0823190602401602060405180830381865afa15801561108a573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906110ae919061388c565b6110cc565b8773ffffffffffffffffffffffffffffffffffffffff16315b90506110d8898561390d565b81101561111e576110e98482613925565b6040517f963b34a500000000000000000000000000000000000000000000000000000000815260040161035d91815260200190565b6111288482613925565b6040805173ffffffffffffffffffffffffffffffffffffffff8b81168252602082018790529181018c905260608101839052919750808c1691908e169033907f2db5ddd0b42bdbca0d69ea16f234a870a485854ae0d91f16643d6f317d8b89949060800160405180910390a450505050509695505050505050565b60005473ffffffffffffffffffffffffffffffffffffffff163314610576576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820181905260248201527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572604482015260640161035d565b6000805473ffffffffffffffffffffffffffffffffffffffff8381167fffffffffffffffffffffffff0000000000000000000000000000000000000000831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b60405173ffffffffffffffffffffffffffffffffffffffff831660248201526044810182905261136d9084907fa9059cbb00000000000000000000000000000000000000000000000000000000906064015b604080517fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe08184030181529190526020810180517bffffffffffffffffffffffffffffffffffffffffffffffffffffffff167fffffffff000000000000000000000000000000000000000000000000000000009093169290921790915261163b565b505050565b6000806113858380516014018051915290565b6040517f70a0823100000000000000000000000000000000000000000000000000000000815230600482015290915073ffffffffffffffffffffffffffffffffffffffff8216906370a0823190602401602060405180830381865afa1580156113f2573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190611416919061388c565b91508115611425576001820391505b61143183308385611747565b50919050565b60006114498380516014018051915290565b905061136d83338385611747565b47611478823073eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee84611747565b919050565b600061148f8280516014018051915290565b905061149f8260008360006117a2565b5050565b60006114b58280516014018051915290565b6040517ff7888aec00000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff80831660048301523060248301529192506000917f0000000000000000000000000000000000000000000000000000000000000000169063f7888aec90604401602060405180830381865afa15801561154d573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190611571919061388c565b9050801561159c577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff015b61136d83308484611747565b60006115ba8280516020018051915290565b905060006115ce8380516020018051915290565b905060006115e28480516001018051915290565b905060006115f68580516020018051915290565b9050600061160a8680516020018051915290565b905061163273ffffffffffffffffffffffffffffffffffffffff8816333088888888886118d6565b50505050505050565b600061169d826040518060400160405280602081526020017f5361666545524332303a206c6f772d6c6576656c2063616c6c206661696c65648152508573ffffffffffffffffffffffffffffffffffffffff16611b569092919063ffffffff16565b80519091501561136d57808060200190518101906116bb919061393c565b61136d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602a60248201527f5361666545524332303a204552433230206f7065726174696f6e20646964206e60448201527f6f74207375636365656400000000000000000000000000000000000000000000606482015260840161035d565b60006117598580516001018051915290565b905060005b8160ff16811015610b5557600061177b8780516002018051915290565b61ffff8082168602049485900394909150611798888888846117a2565b505060010161175e565b60006117b48580516001018051915290565b905060ff81166117cf576117ca85858585611b6d565b6118cf565b8060ff16600114156117e7576117ca85858585611f22565b8060ff16600214156117ff576117ca85858585612162565b8060ff1660031415611817576117ca85858585612410565b8060ff166004141561182f576117ca85858585612a98565b8060ff1660051415611847576117ca85858585612c26565b6040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602160248201527f526f75746550726f636573736f723a20556e6b6e6f776e20706f6f6c2074797060448201527f6500000000000000000000000000000000000000000000000000000000000000606482015260840161035d565b5050505050565b6040517f7ecebe0000000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff8881166004830152600091908a1690637ecebe0090602401602060405180830381865afa158015611946573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061196a919061388c565b6040517fd505accf00000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff8a811660048301528981166024830152604482018990526064820188905260ff8716608483015260a4820186905260c48201859052919250908a169063d505accf9060e401600060405180830381600087803b158015611a0457600080fd5b505af1158015611a18573d6000803e3d6000fd5b50506040517f7ecebe0000000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff8b81166004830152600093508c169150637ecebe0090602401602060405180830381865afa158015611a8b573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190611aaf919061388c565b9050611abc82600161390d565b8114611b4a576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602160248201527f5361666545524332303a207065726d697420646964206e6f742073756363656560448201527f6400000000000000000000000000000000000000000000000000000000000000606482015260840161035d565b50505050505050505050565b6060611b6584846000856131e1565b949350505050565b6000611b7f8580516014018051915290565b90506000611b938680516001018051915290565b90506000611ba78780516014018051915290565b90506000611bbb8880516003018051915290565b905073ffffffffffffffffffffffffffffffffffffffff8716301415611c0157611bfc73ffffffffffffffffffffffffffffffffffffffff87168587611299565b611c41565b73ffffffffffffffffffffffffffffffffffffffff8716331415611c4157611c4173ffffffffffffffffffffffffffffffffffffffff87163386886132fa565b6000808573ffffffffffffffffffffffffffffffffffffffff16630902f1ac6040518163ffffffff1660e01b8152600401606060405180830381865afa158015611c8f573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190611cb39190613977565b506dffffffffffffffffffffffffffff1691506dffffffffffffffffffffffffffff169150600082118015611ce85750600081115b611d4e576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601360248201527f57726f6e6720706f6f6c20726573657276657300000000000000000000000000604482015260640161035d565b6000808660ff16600114611d63578284611d66565b83835b6040517f70a0823100000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff8b8116600483015292945090925083918c16906370a0823190602401602060405180830381865afa158015611dda573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190611dfe919061388c565b611e089190613925565b98506000611e1986620f42406139c7565b611e289062ffffff168b6139eb565b9050600081611e3a85620f42406139eb565b611e44919061390d565b611e4e84846139eb565b611e589190613a28565b90506000808a60ff16600114611e7057826000611e74565b6000835b604080516000815260208101918290527f022c0d9f00000000000000000000000000000000000000000000000000000000909152919350915073ffffffffffffffffffffffffffffffffffffffff8d169063022c0d9f90611ede90859085908f9060248101613ad9565b600060405180830381600087803b158015611ef857600080fd5b505af1158015611f0c573d6000803e3d6000fd5b5050505050505050505050505050505050505050565b6000611f348580516014018051915290565b9050600080611f498780516001018051915290565b60ff161190506000611f618780516014018051915290565b905073ffffffffffffffffffffffffffffffffffffffff8616331415611fa357611fa373ffffffffffffffffffffffffffffffffffffffff86163330876132fa565b600280547fffffffffffffffffffffffff00000000000000000000000000000000000000001673ffffffffffffffffffffffffffffffffffffffff851690811790915563128acb088284878161201757612012600173fffd8963efd1fc6a506488495d951d5263988d26613b14565b612027565b6120276401000276a36001613b41565b6040805173ffffffffffffffffffffffffffffffffffffffff8d166020820152016040516020818303038152906040526040518663ffffffff1660e01b8152600401612077959493929190613b79565b60408051808303816000875af1158015612095573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906120b99190613bc0565b505060025473ffffffffffffffffffffffffffffffffffffffff16600114611632576040517f08c379a0000000000000000000000000000000000000000000000000000000008152602060048201526024808201527f526f75746550726f636573736f722e73776170556e6956333a20756e6578706560448201527f6374656400000000000000000000000000000000000000000000000000000000606482015260840161035d565b60006121748580516001018051915290565b905060006121888680516014018051915290565b9050600180831614156122575760006121a78780516014018051915290565b905060028316612213578073ffffffffffffffffffffffffffffffffffffffff1663d0e30db0856040518263ffffffff1660e01b81526004016000604051808303818588803b1580156121f957600080fd5b505af115801561220d573d6000803e3d6000fd5b50505050505b73ffffffffffffffffffffffffffffffffffffffff821630146122515761225173ffffffffffffffffffffffffffffffffffffffff82168386611299565b50610b55565b600282166123205773ffffffffffffffffffffffffffffffffffffffff851633141561229f5761229f73ffffffffffffffffffffffffffffffffffffffff85163330866132fa565b6040517f2e1a7d4d0000000000000000000000000000000000000000000000000000000081526004810184905273ffffffffffffffffffffffffffffffffffffffff851690632e1a7d4d90602401600060405180830381600087803b15801561230757600080fd5b505af115801561231b573d6000803e3d6000fd5b505050505b60008173ffffffffffffffffffffffffffffffffffffffff168460405160006040518083038185875af1925050503d806000811461237a576040519150601f19603f3d011682016040523d82523d6000602084013e61237f565b606091505b5050905080611632576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152603760248201527f526f75746550726f636573736f722e777261704e61746976653a204e6174697660448201527f6520746f6b656e207472616e73666572206661696c6564000000000000000000606482015260840161035d565b60006124228580516001018051915290565b905060006124368680516014018051915290565b905060ff8216156128315773ffffffffffffffffffffffffffffffffffffffff85163014156124a5576124a073ffffffffffffffffffffffffffffffffffffffff85167f000000000000000000000000000000000000000000000000000000000000000085611299565b61275b565b73ffffffffffffffffffffffffffffffffffffffff8516331415612505576124a073ffffffffffffffffffffffffffffffffffffffff8516337f0000000000000000000000000000000000000000000000000000000000000000866132fa565b6040517f4ffe34db00000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff85811660048301527f00000000000000000000000000000000000000000000000000000000000000001690634ffe34db906024016040805180830381865afa158015612590573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906125b49190613c04565b516040517fdf23b45b00000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff86811660048301526fffffffffffffffffffffffffffffffff909216917f0000000000000000000000000000000000000000000000000000000000000000169063df23b45b90602401606060405180830381865afa158015612656573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061267a9190613c77565b60409081015190517f70a0823100000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff7f0000000000000000000000000000000000000000000000000000000000000000811660048301526fffffffffffffffffffffffffffffffff909216918716906370a0823190602401602060405180830381865afa158015612720573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190612744919061388c565b61274e919061390d565b6127589190613925565b92505b6040517f02b9446c00000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff85811660048301527f000000000000000000000000000000000000000000000000000000000000000081166024830181905290831660448301526064820185905260006084830152906302b9446c9060a40160408051808303816000875af1158015612806573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061282a9190613bc0565b5050610b55565b73ffffffffffffffffffffffffffffffffffffffff851615612908576040517ff18d03cc00000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff85811660048301528681166024830152306044830152606482018590527f0000000000000000000000000000000000000000000000000000000000000000169063f18d03cc90608401600060405180830381600087803b1580156128eb57600080fd5b505af11580156128ff573d6000803e3d6000fd5b505050506129c1565b6040517ff7888aec00000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff85811660048301523060248301527f0000000000000000000000000000000000000000000000000000000000000000169063f7888aec90604401602060405180830381865afa15801561299a573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906129be919061388c565b92505b6040517f97da6d3000000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff8581166004830152306024830152828116604483015260006064830152608482018590527f000000000000000000000000000000000000000000000000000000000000000016906397da6d309060a40160408051808303816000875af1158015612a6a573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190612a8e9190613bc0565b5050505050505050565b6000612aaa8580516014018051915290565b85516020808201805190920101875290915073ffffffffffffffffffffffffffffffffffffffff851615612b91576040517ff18d03cc00000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff858116600483015286811660248301528381166044830152606482018590527f0000000000000000000000000000000000000000000000000000000000000000169063f18d03cc90608401600060405180830381600087803b158015612b7857600080fd5b505af1158015612b8c573d6000803e3d6000fd5b505050505b6040517f627dd56a00000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff83169063627dd56a90612be3908490600401613ce3565b6020604051808303816000875af1158015612c02573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190611632919061388c565b6000612c388580516014018051915290565b90506000612c4c8680516001018051915290565b90506000612c608780516001018051915290565b60000b90506000612c778880516001018051915290565b60000b90506000612c8e8980516014018051915290565b90506000612ca28a80516014018051915290565b9050600073ffffffffffffffffffffffffffffffffffffffff891673eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee1415612d8b576040517f3df02124000000000000000000000000000000000000000000000000000000008152600f86810b600483015285900b6024820152604481018990526000606482015273ffffffffffffffffffffffffffffffffffffffff881690633df02124908a9060840160206040518083038185885af1158015612d5f573d6000803e3d6000fd5b50505050506040513d601f19601f82011682018060405250810190612d84919061388c565b905061306e565b73ffffffffffffffffffffffffffffffffffffffff8a16331415612dcb57612dcb73ffffffffffffffffffffffffffffffffffffffff8a1633308b6132fa565b612dec73ffffffffffffffffffffffffffffffffffffffff8a16888a613358565b5060ff8616612e9f576040517f3df02124000000000000000000000000000000000000000000000000000000008152600f86810b600483015285900b6024820152604481018990526000606482015273ffffffffffffffffffffffffffffffffffffffff881690633df02124906084016020604051808303816000875af1158015612e7b573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190612d84919061388c565b6040517f70a0823100000000000000000000000000000000000000000000000000000000815230600482015260009073ffffffffffffffffffffffffffffffffffffffff8416906370a0823190602401602060405180830381865afa158015612f0c573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190612f30919061388c565b6040517f3df02124000000000000000000000000000000000000000000000000000000008152600f88810b600483015287900b6024820152604481018b90526000606482015290915073ffffffffffffffffffffffffffffffffffffffff891690633df0212490608401600060405180830381600087803b158015612fb457600080fd5b505af1158015612fc8573d6000803e3d6000fd5b50506040517f70a082310000000000000000000000000000000000000000000000000000000081523060048201526000925073ffffffffffffffffffffffffffffffffffffffff861691506370a0823190602401602060405180830381865afa158015613039573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061305d919061388c565b90506130698282613925565b925050505b73ffffffffffffffffffffffffffffffffffffffff831630146131d45773ffffffffffffffffffffffffffffffffffffffff821673eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee14156131b35760008373ffffffffffffffffffffffffffffffffffffffff168260405160006040518083038185875af1925050503d8060008114613117576040519150601f19603f3d011682016040523d82523d6000602084013e61311c565b606091505b50509050806131ad576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152603660248201527f526f75746550726f636573736f722e7377617043757276653a204e617469766560448201527f20746f6b656e207472616e73666572206661696c656400000000000000000000606482015260840161035d565b506131d4565b6131d473ffffffffffffffffffffffffffffffffffffffff83168483611299565b5050505050505050505050565b606082471015613273576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602660248201527f416464726573733a20696e73756666696369656e742062616c616e636520666f60448201527f722063616c6c0000000000000000000000000000000000000000000000000000606482015260840161035d565b6000808673ffffffffffffffffffffffffffffffffffffffff16858760405161329c9190613cf6565b60006040518083038185875af1925050503d80600081146132d9576040519150601f19603f3d011682016040523d82523d6000602084013e6132de565b606091505b50915091506132ef87838387613389565b979650505050505050565b60405173ffffffffffffffffffffffffffffffffffffffff808516602483015283166044820152606481018290526103b49085907f23b872dd00000000000000000000000000000000000000000000000000000000906084016112eb565b6000613365848484613426565b80611b65575061337784846000613426565b8015611b655750611b65848484613426565b6060831561341c5782516134155773ffffffffffffffffffffffffffffffffffffffff85163b613415576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601d60248201527f416464726573733a2063616c6c20746f206e6f6e2d636f6e7472616374000000604482015260640161035d565b5081611b65565b611b658383613535565b6040805173ffffffffffffffffffffffffffffffffffffffff8481166024830152604480830185905283518084039091018152606490920183526020820180517bffffffffffffffffffffffffffffffffffffffffffffffffffffffff167f095ea7b300000000000000000000000000000000000000000000000000000000179052915160009283928392918816916134bf9190613cf6565b6000604051808303816000865af19150503d80600081146134fc576040519150601f19603f3d011682016040523d82523d6000602084013e613501565b606091505b509150915081801561352b57508051158061352b57508080602001905181019061352b919061393c565b9695505050505050565b8151156135455781518083602001fd5b806040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161035d9190613ce3565b6000806000806060858703121561358f57600080fd5b8435935060208501359250604085013567ffffffffffffffff808211156135b557600080fd5b818701915087601f8301126135c957600080fd5b8135818111156135d857600080fd5b8860208285010111156135ea57600080fd5b95989497505060200194505050565b73ffffffffffffffffffffffffffffffffffffffff811681146109ac57600080fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b600082601f83011261365b57600080fd5b813567ffffffffffffffff808211156136765761367661361b565b604051601f83017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0908116603f011681019082821181831017156136bc576136bc61361b565b816040528381528660208588010111156136d557600080fd5b836020870160208301376000602085830101528094505050505092915050565b60008060008060008060c0878903121561370e57600080fd5b8635613719816135f9565b9550602087013594506040870135613730816135f9565b9350606087013592506080870135613747816135f9565b915060a087013567ffffffffffffffff81111561376357600080fd5b61376f89828a0161364a565b9150509295509295509295565b600080600080600080600080610100898b03121561379957600080fd5b88356137a4816135f9565b97506020890135965060408901356137bb816135f9565b95506060890135945060808901356137d2816135f9565b935060a0890135925060c08901356137e9816135f9565b915060e089013567ffffffffffffffff81111561380557600080fd5b6138118b828c0161364a565b9150509295985092959890939650565b80151581146109ac57600080fd5b6000806040838503121561384257600080fd5b823561384d816135f9565b9150602083013561385d81613821565b809150509250929050565b60006020828403121561387a57600080fd5b8135613885816135f9565b9392505050565b60006020828403121561389e57600080fd5b5051919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b60007fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff821415613906576139066138a5565b5060010190565b60008219821115613920576139206138a5565b500190565b600082821015613937576139376138a5565b500390565b60006020828403121561394e57600080fd5b815161388581613821565b80516dffffffffffffffffffffffffffff8116811461147857600080fd5b60008060006060848603121561398c57600080fd5b61399584613959565b92506139a360208501613959565b9150604084015163ffffffff811681146139bc57600080fd5b809150509250925092565b600062ffffff838116908316818110156139e3576139e36138a5565b039392505050565b6000817fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0483118215151615613a2357613a236138a5565b500290565b600082613a5e577f4e487b7100000000000000000000000000000000000000000000000000000000600052601260045260246000fd5b500490565b60005b83811015613a7e578181015183820152602001613a66565b838111156103b45750506000910152565b60008151808452613aa7816020860160208601613a63565b601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169290920160200192915050565b84815283602082015273ffffffffffffffffffffffffffffffffffffffff8316604082015260806060820152600061352b6080830184613a8f565b600073ffffffffffffffffffffffffffffffffffffffff838116908316818110156139e3576139e36138a5565b600073ffffffffffffffffffffffffffffffffffffffff808316818516808303821115613b7057613b706138a5565b01949350505050565b600073ffffffffffffffffffffffffffffffffffffffff8088168352861515602084015285604084015280851660608401525060a060808301526132ef60a0830184613a8f565b60008060408385031215613bd357600080fd5b505080516020909101519092909150565b80516fffffffffffffffffffffffffffffffff8116811461147857600080fd5b600060408284031215613c1657600080fd5b6040516040810181811067ffffffffffffffff82111715613c3957613c3961361b565b604052613c4583613be4565b8152613c5360208401613be4565b60208201529392505050565b805167ffffffffffffffff8116811461147857600080fd5b600060608284031215613c8957600080fd5b6040516060810181811067ffffffffffffffff82111715613cac57613cac61361b565b604052613cb883613c5f565b8152613cc660208401613c5f565b6020820152613cd760408401613be4565b60408201529392505050565b6020815260006138856020830184613a8f565b60008251613d08818460208701613a63565b919091019291505056fea26469706673582212201ff294929b57776d43429e47ba13e7bb550e4a27be6afbadf8c8a0d6ad6324b364736f6c634300080a0033000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000";

/// @title Deploy
/// @notice A script that deploys all contracts. This is intended to be run on
/// every commit by CI to a testnet such as mumbai.
contract Deploy is Script {
    function deployRaindex() internal returns (OrderBook) {
        return new OrderBook();
    }

    function deploySubParser(IMetaBoardV1 metaboard) internal {
        OrderBookSubParser subParser = new OrderBookSubParser();
        bytes memory subParserDescribedByMeta = vm.readFileBinary("meta/OrderBookSubParser.rain.meta");
        LibDescribedByMeta.emitForDescribedAddress(metaboard, subParser, subParserDescribedByMeta);
    }

    function deployRouter() internal returns (address) {
        bytes memory routeProcessor4Code = ROUTE_PROCESSOR_4_CREATION_CODE;
        address routeProcessor4;
        assembly ("memory-safe") {
            routeProcessor4 := create(0, add(routeProcessor4Code, 0x20), mload(routeProcessor4Code))
        }
        return routeProcessor4;
    }

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");
        IMetaBoardV1 metaboard = IMetaBoardV1(vm.envAddress("DEPLOY_METABOARD_ADDRESS"));
        string memory suiteString = vm.envOr("DEPLOYMENT_SUITE", string("all"));
        bytes32 suite = keccak256(bytes(suiteString));

        vm.startBroadcast(deployerPrivateKey);

        address orderbook = address(0);
        address routeProcessor = address(0);

        if (suite == DEPLOYMENT_SUITE_RAINDEX || suite == DEPLOYMENT_SUITE_ALL) {
            deployRaindex();
        }

        if (suite == DEPLOYMENT_SUITE_SUBPARSER || suite == DEPLOYMENT_SUITE_ALL) {
            deploySubParser(metaboard);
        }

        if (suite == DEPLOYMENT_SUITE_ROUTE_PROCESSOR || suite == DEPLOYMENT_SUITE_ALL) {
            deployRouter();
        }

        if (suite == DEPLOYMENT_SUITE_ARB || suite == DEPLOYMENT_SUITE_ALL) {
            if (orderbook == address(0)) {
                orderbook = vm.envAddress("DEPLOY_ORDERBOOK_ADDRESS");
            }
            if (routeProcessor == address(0)) {
                routeProcessor = vm.envAddress("DEPLOY_ROUTE_PROCESSOR_4_ADDRESS");
            }

            // Order takers.
            new GenericPoolOrderBookV3ArbOrderTaker(
                OrderBookV3ArbConfigV1(
                    orderbook, EvaluableConfigV3(IExpressionDeployerV3(address(0)), "", new uint256[](0)), ""
                )
            );

            new RouteProcessorOrderBookV3ArbOrderTaker(
                OrderBookV3ArbConfigV1(
                    orderbook,
                    EvaluableConfigV3(IExpressionDeployerV3(address(0)), "", new uint256[](0)),
                    abi.encode(routeProcessor)
                )
            );

            // Flash borrowers.
            new GenericPoolOrderBookV3FlashBorrower(
                OrderBookV3ArbConfigV1(
                    orderbook, EvaluableConfigV3(IExpressionDeployerV3(address(0)), "", new uint256[](0)), ""
                )
            );
        }
        vm.stopBroadcast();
    }
}
