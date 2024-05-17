use borsh::BorshSerialize;
pub use l1x_sdk_macros::contract;
pub use l1x_sys as sys;
use std::panic as std_panic;
use types::{Address, Balance, BlockHash, BlockNumber, Gas, TimeStamp};

pub mod contract_interaction;
pub mod store;
pub mod types;
use contract_interaction::ContractCall;
pub mod utils;
pub(crate) use crate::utils::*;

const EVICTED_REGISTER: u64 = std::u64::MAX - 1;
const ATOMIC_OP_REGISTER: u64 = std::u64::MAX - 2;

#[derive(Debug)]
pub enum TransferError {
    TransferFailed,
    InsufficientFunds,
}

macro_rules! try_method_into_register {
    ( $method:ident ) => {{
        unsafe { l1x_sys::$method(ATOMIC_OP_REGISTER) };
        read_register(ATOMIC_OP_REGISTER)
    }};
}

macro_rules! method_into_register {
    ( $method:ident ) => {{
        expect_register(try_method_into_register!($method))
    }};
}

/// Returns the size of the register. If register is not used returns `None`.
fn register_len(register_id: u64) -> Option<u64> {
    let len = unsafe { l1x_sys::register_len(register_id) };
    if len == std::u64::MAX {
        None
    } else {
        Some(len)
    }
}

/// Reads the content of the `register_id`. If register is not used returns `None`.
fn read_register(register_id: u64) -> Option<Vec<u8>> {
    let len: usize = register_len(register_id)?
        .try_into()
        .unwrap_or_else(|_| abort());

    let mut buffer = Vec::with_capacity(len);

    unsafe {
        l1x_sys::read_register(register_id, buffer.as_mut_ptr() as u64);

        buffer.set_len(len);
    }
    Some(buffer)
}

fn expect_register<T>(option: Option<T>) -> T {
    option.unwrap_or_else(|| abort())
}

/// Implements panic hook that converts `PanicInfo` into a string and provides it through the
/// blockchain interface.
fn panic_hook_impl(info: &std_panic::PanicInfo) {
    panic(&info.to_string());
}

/// Setups panic hook to expose error info to the blockchain.
pub fn setup_panic_hook() {
    std_panic::set_hook(Box::new(panic_hook_impl));
}

/// Aborts the current contract execution without a custom message.
/// To include a message, use [`crate::panic`].
pub fn abort() -> ! {
    #[cfg(test)]
    std::panic!("Mocked panic function called!");
    #[cfg(not(test))]
    unsafe {
        l1x_sys::panic()
    }
}

/// Terminates the execution of the program with the message.
pub fn panic(message: &str) -> ! {
    msg(message);

    #[cfg(test)]
    std::panic!("Mocked panic function called!");
    #[cfg(not(test))]
    unsafe {
        l1x_sys::panic_msg(message.as_ptr() as _, message.len() as _)
    }
}

/// The input to the contract call serialized as bytes. If input is not provided returns `None`.
pub fn input() -> Option<Vec<u8>> {
    #[cfg(test)]
    {
        return tests::input();
    }
    #[cfg(not(test))]
    try_method_into_register!(input)
}

/// Writes `data` to 'output' register
pub fn output(data: &[u8]) {
    #[cfg(test)]
    {
        return tests::output(data);
    }
    #[cfg(not(test))]
    unsafe {
        sys::output(data.as_ptr() as _, data.len() as _)
    }
}

pub fn msg(message: &str) {
    #[cfg(test)]
    {
        return tests::msg(message);
    }
    #[cfg(not(test))]
    {
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        eprintln!("{}", message);

        unsafe { l1x_sys::msg(message.as_ptr() as _, message.len() as _) }
    }
}

/// Writes key-value into storage.
///
/// If the the storage did not have this key present, `false` is returned.
///
/// If the map did have this key present, the value is updated, and `true` is returned.
pub fn storage_write(key: &[u8], value: &[u8]) -> bool {
    #[cfg(test)]
    {
        return tests::storage_write(key, value);
    }
    #[cfg(not(test))]
    match unsafe {
        sys::storage_write(
            key.as_ptr() as _,
            key.len() as _,
            value.as_ptr() as _,
            value.len() as _,
            EVICTED_REGISTER,
        )
    } {
        0 => false,
        1 => true,
        _ => abort(),
    }
}

/// Removes the value stored under the given key.
///
/// If key-value existed returns `true`, otherwise `false`.
pub fn storage_remove(key: &[u8]) -> bool {
    #[cfg(test)]
    {
        return tests::storage_remove(key);
    }

    #[cfg(not(test))]
    match unsafe { sys::storage_remove(key.as_ptr() as _, key.len() as _, EVICTED_REGISTER) } {
        0 => false,
        1 => true,
        _ => abort(),
    }
}

/// Reads the value stored under the given key.
///
/// If the storage doesn't have the key present, returns `None`
pub fn storage_read(key: &[u8]) -> Option<Vec<u8>> {
    #[cfg(test)]
    {
        return tests::storage_read(key);
    }

    #[cfg(not(test))]
    match unsafe { sys::storage_read(key.as_ptr() as _, key.len() as _, ATOMIC_OP_REGISTER) } {
        0 => None,
        1 => Some(expect_register(read_register(ATOMIC_OP_REGISTER))),
        _ => abort(),
    }
}

/// Returns `true` if the contract has write permissions and `false` if it doesn't.
pub fn storage_write_perm() -> bool {
    match unsafe { sys::storage_write_perm() } {
        0 => false,
        1 => true,
        _ => abort(),
    }
}

/// Returns the address of the account that owns the current contract.
pub fn contract_owner_address() -> Address {
    #[cfg(test)]
    {
        return tests::contract_owner_address();
    }
    #[cfg(not(test))]
    method_into_register!(contract_owner_address)
        .try_into()
        .unwrap_or_else(|_| abort())
}

/// Returns the address of the account or the contract that called the current contract.
pub fn caller_address() -> Address {
    #[cfg(test)]
    {
        return tests::caller_address();
    }
    #[cfg(not(test))]
    method_into_register!(caller_address)
        .try_into()
        .unwrap_or_else(|_| abort())
}

/// Returns the address of the current contract's instance.
pub fn contract_instance_address() -> Address {
    #[cfg(test)]
    {
        return tests::contract_instance_address();
    }
    #[cfg(not(test))]
    method_into_register!(contract_instance_address)
        .try_into()
        .unwrap_or_else(|_| abort())
}

/// Returns the address of the account that owns the given contract instance
///
/// # Panics
///
/// If the contract instance is not found by the given address
pub fn contract_owner_address_of(instance_address: Address) -> Address {
    let addr = instance_address.as_bytes();
    unsafe {
        l1x_sys::contract_owner_address_of(addr.as_ptr() as _, addr.len() as _, ATOMIC_OP_REGISTER);
    }
    let maybe_addr = expect_register(read_register(ATOMIC_OP_REGISTER));
    Address::try_from(maybe_addr).expect("VM returned an incorrect address")
}

/// Returns the address of the account that owns the given contract code
///
/// # Panics
///
/// If the contract code is not found by the given address
pub fn contract_code_owner_address_of(code_address: Address) -> Address {
    let addr = code_address.as_bytes();
    unsafe {
        l1x_sys::contract_code_owner_address_of(
            addr.as_ptr() as _,
            addr.len() as _,
            ATOMIC_OP_REGISTER,
        );
    }
    let maybe_addr = expect_register(read_register(ATOMIC_OP_REGISTER));
    Address::try_from(maybe_addr).expect("VM returned an incorrect address")
}

/// Returns the address of the contract code that is used for the given contract instance
///
/// # Panics
///
/// If the contract instance is not found by the given address
pub fn contract_code_address_of(instance_address: Address) -> Address {
    let addr = instance_address.as_bytes();
    unsafe {
        l1x_sys::contract_code_address_of(addr.as_ptr() as _, addr.len() as _, ATOMIC_OP_REGISTER);
    }
    let maybe_addr = expect_register(read_register(ATOMIC_OP_REGISTER));
    Address::try_from(maybe_addr).expect("VM returned an incorrect address")
}

/// Returns `Balance` of the given `Address`
///
/// If `Address` not found, returns `0`
pub fn address_balance(address: &Address) -> Balance {
    let address_vec = address.to_vec();
    unsafe {
        l1x_sys::address_balance(
            address_vec.as_ptr() as _,
            address_vec.len() as _,
            ATOMIC_OP_REGISTER,
        )
    };
    let bytes = expect_register(read_register(ATOMIC_OP_REGISTER));

    u128::from_le_bytes(bytes.try_into().unwrap_or_else(|_| abort()))
}

/// Transfers `amount` of L1X tokens from [`contract_instance_address`] to the specified address
///
/// # Panics
///
/// Panics if transfer failed
pub fn transfer_to(to: &Address, amount: Balance) {
    let to_address_vec = to.to_vec();
    let amount = amount.to_le_bytes();
    match unsafe {
        l1x_sys::transfer_to(
            to_address_vec.as_ptr() as _,
            to_address_vec.len() as _,
            amount.as_ptr() as _,
            amount.len() as _,
        )
    } {
        1 => (),
        0 => crate::panic("Transfer tokens from the contract balance failed"),
        _ => abort(),
    };
}

/// Transfers `amount` of L1X tokens from [`caller_address`] to [`contract_instance_address`]
///
/// # Panics
///
/// Panics if transfer failed
pub fn transfer_from_caller(amount: Balance) {
    let amount = amount.to_le_bytes();
    match unsafe { l1x_sys::transfer_from_caller(amount.as_ptr() as _, amount.len() as _) } {
        1 => (),
        0 => crate::panic("Transfer tokens from the caller balance failed"),
        _ => abort(),
    }
}

/// Returns the hash of the current block
pub fn block_hash() -> BlockHash {
    let mut buf = BlockHash::default();

    unsafe { l1x_sys::block_hash(buf.as_mut_ptr() as _, buf.len() as _) };

    buf
}

/// Returns the number of the current block
pub fn block_number() -> BlockNumber {
    let mut buf = [0u8; std::mem::size_of::<BlockNumber>()];

    unsafe { l1x_sys::block_number(buf.as_mut_ptr() as _, buf.len() as _) };

    BlockNumber::from_le_bytes(buf)
}

/// Returns the timestamp of the current block
pub fn block_timestamp() -> TimeStamp {
    let mut buf = [0u8; std::mem::size_of::<TimeStamp>()];

    unsafe { l1x_sys::block_timestamp(buf.as_mut_ptr() as _, buf.len() as _) };

    TimeStamp::from_le_bytes(buf)
}

/// Returns the total amount of `Gas` that is allowed the contract to burn out
pub fn gas_limit() -> Gas {
    unsafe { l1x_sys::gas_limit() }
}

/// Returns the amount of available `Gas`
pub fn gas_left() -> Gas {
    unsafe { l1x_sys::gas_left() }
}

/// Returns `Balance` of the current contract's instance.
pub fn contract_instance_balance() -> Balance {
    address_balance(&contract_instance_address())
}

/// Calls another contract
///
/// # Panics
///
/// - If deserialization of `call` failed
/// - If `call.read_only` is `false` but `call_contract` is called from read-only context
/// - If there is not enough `Gas` to satisfy `gas_limit`
pub fn call_contract(call: &ContractCall) -> Result<Vec<u8>, String> {
    let call = call
        .try_to_vec()
        .expect("Can't serialize the function arguments");
    match unsafe { sys::call_contract2(call.as_ptr() as _, call.len() as _, ATOMIC_OP_REGISTER) } {
        0 => Err(
            String::from_utf8_lossy(&expect_register(read_register(ATOMIC_OP_REGISTER)))
                .to_string(),
        ),
        1 => Ok(expect_register(read_register(ATOMIC_OP_REGISTER))),
        _ => abort(),
    }
}

/// Emits the event. This `event` is stored on chain.
pub fn emit_event_experimental<T>(event: T)
where
    T: BorshSerialize,
{
    let event_data = event.try_to_vec().expect("Can't serialize the event");
    match unsafe { sys::emit_event_experimental(event_data.as_ptr() as _, event_data.len() as _) } {
        0 => abort(),
        _ => (),
    }
}

#[cfg(test)]
mod tests {

    use crate::types::Address;
    use std::cell::RefCell;
    use std::collections::HashMap;

    thread_local! {
        static MOCK_DATA: RefCell<MockData> = RefCell::new(MockData::new());
    }

    const CONTRACT_OWNER_ADDRESS: &[u8; 20] = b"mock_owner_address11";
    const CONTRACT_INSTANCE_ADDRESS: &[u8; 20] = b"mock_instance_addres";
    const CALLER_ADDRESS: &[u8; 20] = b"mock_caller_address1";

    pub struct MockData {
        storage: HashMap<Vec<u8>, Vec<u8>>,
        input: Option<Vec<u8>>,
        output: Vec<u8>,
        messages: Vec<String>,
        contract_owner_address: Address,
        caller_address: Address,
        contract_instance_address: Address,
    }

    impl MockData {
        pub fn new() -> Self {
            Self {
                storage: HashMap::new(),
                input: Some(Vec::new()),
                output: Vec::new(),
                messages: Vec::new(),
                contract_owner_address: Address::test_create_address(
                    &CONTRACT_OWNER_ADDRESS.to_vec(),
                ),
                caller_address: Address::test_create_address(&CALLER_ADDRESS.to_vec()),
                contract_instance_address: Address::test_create_address(
                    &CONTRACT_INSTANCE_ADDRESS.to_vec(),
                ),
            }
        }
    }

    pub fn storage_write(key: &[u8], value: &[u8]) -> bool {
        MOCK_DATA.with(|data| {
            let mut mock_data = data.borrow_mut();
            // Check if the key is already in the storage
            let is_new_insertion = !mock_data.storage.contains_key(key);
            mock_data.storage.insert(key.to_vec(), value.to_vec());
            is_new_insertion
        })
    }

    pub fn storage_read(key: &[u8]) -> Option<Vec<u8>> {
        MOCK_DATA.with(|data| data.borrow().storage.get(key).cloned())
    }

    pub fn storage_remove(key: &[u8]) -> bool {
        MOCK_DATA.with(|data| data.borrow_mut().storage.remove(key).is_some())
    }

    pub fn contract_owner_address() -> Address {
        MOCK_DATA.with(|data| data.borrow().contract_owner_address.clone())
    }

    pub fn caller_address() -> Address {
        MOCK_DATA.with(|data| data.borrow().caller_address.clone())
    }

    pub fn contract_instance_address() -> Address {
        MOCK_DATA.with(|data| data.borrow().contract_instance_address.clone())
    }

    pub fn remove_from_mock_storage(key: &[u8]) -> bool {
        MOCK_DATA.with(|data| data.borrow_mut().storage.remove(key).is_some())
    }

    pub fn input() -> Option<Vec<u8>> {
        MOCK_DATA.with(|data| data.borrow().input.clone())
    }

    pub fn output(data: &[u8]) {
        MOCK_DATA.with(|data_refcell| {
            let mut data_inside = data_refcell.borrow_mut();
            data_inside.output = data.to_vec();
        })
    }

    pub fn msg(message: &str) {
        MOCK_DATA.with(|data| data.borrow_mut().messages.push(message.to_owned()))
    }

    pub fn set_mock_input(data: Vec<u8>) {
        MOCK_DATA.with(|data_refcell| {
            let mut data_inside = data_refcell.borrow_mut();
            data_inside.input = Some(data);
        });
    }

    pub fn get_mock_output() -> Vec<u8> {
        MOCK_DATA.with(|data| data.borrow().output.clone())
    }

    pub fn get_mock_msgs() -> Vec<String> {
        MOCK_DATA.with(|data| data.borrow().messages.clone())
    }

    pub fn clear_mock_io() {
        MOCK_DATA.with(|data| {
            let mut data = data.borrow_mut();
            data.input = None;
            data.output = Vec::new();
            data.messages = Vec::new();
        })
    }

    pub fn set_mock_contract_owner_address(owner_address: Vec<u8>) {
        MOCK_DATA.with(|data| {
            data.borrow_mut().contract_owner_address = Address::test_create_address(&owner_address)
        })
    }

    pub fn set_mock_caller_address(caller_address: Vec<u8>) {
        MOCK_DATA.with(|data| {
            data.borrow_mut().caller_address = Address::test_create_address(&caller_address)
        })
    }

    pub fn set_mock_contract_instance_address(contract_instance_address: Vec<u8>) {
        MOCK_DATA.with(|data| {
            data.borrow_mut().contract_instance_address =
                Address::test_create_address(&contract_instance_address)
        })
    }

    ////////////////////////////////////////////// TESTS ////////////////////////////////////////////////////////////
    #[test]
    fn test_storage() {
        // Prepare key-value
        let key = b"key";
        let value = b"value";

        // Write to storage
        assert!(storage_write(key, value));

        // Read from storage
        let stored_value = storage_read(key).unwrap();
        assert_eq!(stored_value, value);

        // Remove from storage
        assert!(storage_remove(key));

        // Try to read removed key
        assert!(storage_read(key).is_none());
    }

    #[test]
    fn test_msg() {
        let message = "Test message";
        msg(message);

        let mock_messages = get_mock_msgs();
        assert_eq!(mock_messages.len(), 1);
        assert_eq!(mock_messages[0], message);
    }

    #[test]
    fn test_input_output() {
        let data = vec![1, 2, 3, 4];

        set_mock_input(data.clone());

        // Check input
        let input_data = input().unwrap();
        assert_eq!(input_data, data);

        // Output
        output(&data);

        // Check output
        let output_data = get_mock_output();
        assert_eq!(output_data, data);

        // Clear
        clear_mock_io();

        // Check input and output are cleared
        assert!(input().is_none());
        assert!(get_mock_output().is_empty());
    }

    #[test]
    fn test_storage_write_and_read() {
        let key = vec![1, 2, 3];
        let value = vec![4, 5, 6];

        // Write to storage
        storage_write(&key, &value);

        // Read from storage and check value
        let stored_value = storage_read(&key).unwrap();
        assert_eq!(stored_value, value);
    }

    #[test]
    fn test_remove_from_mock_storage() {
        let key = vec![1, 2, 3];
        let value = vec![4, 5, 6];

        // Write to storage and then remove
        storage_write(&key, &value);
        remove_from_mock_storage(&key);

        // Check value is removed
        let stored_value = storage_read(&key);
        assert!(stored_value.is_none());
    }

    #[test]
    fn test_contract_owner_address_and_caller_address() {
        let mock_owner_address = b"current_address12345".to_vec();
        let mock_caller_address = b"caller_address123456".to_vec();
        let mock_instance_address = b"instance_address3456".to_vec();

        // Set mock data
        set_mock_contract_owner_address(mock_owner_address.clone());
        set_mock_caller_address(mock_caller_address.clone());
        set_mock_contract_instance_address(mock_instance_address.clone());

        // Test contract_owner_address
        assert_eq!(
            contract_owner_address(),
            Address::test_create_address(&mock_owner_address)
        );

        // Test caller_address
        assert_eq!(
            caller_address(),
            Address::test_create_address(&mock_caller_address)
        );

        assert_eq!(
            contract_instance_address(),
            Address::test_create_address(&mock_instance_address)
        );
    }

    #[test]
    fn test_input_and_output() {
        let data = vec![1, 2, 3];

        // Set mock input and verify it
        set_mock_input(data.clone());
        assert_eq!(input().unwrap(), data);

        // Write to output
        output(&data);

        // Verify output
        assert_eq!(get_mock_output(), data);
    }

    #[test]
    fn test_clear_mock_io() {
        // Set some mock input/output data and a message
        set_mock_input(vec![1, 2, 3]);
        output(&vec![4, 5, 6]);
        msg("Hello, world!");

        // Clear the mock I/O data
        clear_mock_io();

        // Verify everything was cleared
        assert!(input().is_none());
        assert_eq!(get_mock_output(), vec![] as Vec<u8>);
        assert_eq!(get_mock_msgs(), Vec::<String>::new());
    }
}
