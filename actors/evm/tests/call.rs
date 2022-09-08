mod asm;

use evm::interpreter::address::Address as EVMAddress;
use evm::interpreter::U256;
use fil_actor_evm as evm;
use fil_actors_runtime::test_utils::*;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address as FILAddress;
use fvm_shared::bigint::Zero;
use fvm_shared::econ::TokenAmount;
use fvm_shared::error::ExitCode;

#[allow(dead_code)]
pub fn call_proxy_contract() -> Vec<u8> {
    let init = "";
    let body = r#"
# this contract takes an address and the call payload and proxies a call to that address
# get call payload size
push1 0x20
calldatasize
sub
# store payload to mem 0x00
push1 0x20
push1 0x00
calldatacopy

# prepare the proxy call
# output offset and size -- 0 in this case, we use returndata
push2 0x00
push1 0x00
# input offset and size
push1 0x20
calldatasize
sub
push1 0x00
# value
push1 0x00
# dest address
push1 0x00
calldataload
# gas
push1 0x00
# do the call
call

# return result through
returndatasize
push1 0x00
push1 0x00
returndatacopy
returndatasize
push1 0x00
return
"#;

    asm::new_contract("call-proxy", init, body).unwrap()
}

#[test]
fn test_call() {
    let mut rt = MockRuntime::default();

    // construct the proxy
    let contract = call_proxy_contract();
    rt.expect_validate_caller_any();
    let params =
        evm::ConstructorParams { bytecode: contract.into(), input_data: RawBytes::default() };
    let result = rt
        .call::<evm::EvmContractActor>(
            evm::Method::Constructor as u64,
            &RawBytes::serialize(params).unwrap(),
        )
        .unwrap();
    expect_empty(result);
    rt.verify();

    // create a mock target and proxy a call through the proxy
    let target = FILAddress::new_id(0x100);
    rt.actor_code_cids.insert(target, *EVM_ACTOR_CODE_ID);

    let evm_target = EVMAddress::from_id_address(&target).unwrap();
    let evm_target_word = evm_target.as_evm_word();

    // dest + method 0 with no data
    let mut contract_params = vec![0u8; 36];
    evm_target_word.to_big_endian(&mut contract_params[..32]);
    let params = evm::InvokeParams { input_data: RawBytes::from(contract_params) };

    let proxy_call_contract_params = vec![0u8; 4];
    let proxy_call_params =
        evm::InvokeParams { input_data: RawBytes::from(proxy_call_contract_params) };

    // expected return data
    let mut return_data = vec![0u8; 32];
    return_data[31] = 0x42;

    rt.expect_validate_caller_any();
    rt.expect_send(
        target,
        evm::Method::InvokeContract as u64,
        RawBytes::serialize(proxy_call_params).unwrap(),
        TokenAmount::zero(),
        RawBytes::from(return_data),
        ExitCode::OK,
    );
    let result = rt
        .call::<evm::EvmContractActor>(
            evm::Method::InvokeContract as u64,
            &RawBytes::serialize(params).unwrap(),
        )
        .unwrap();

    assert_eq!(U256::from_big_endian(&result), U256::from(0x42));
}