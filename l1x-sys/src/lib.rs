#![no_std]

// Registers are a nice abstraction that allows developers to store data without moving it outside of VM.
type RegisterId = u64;
// An address in virtual memory.
type MemoryAddress = u64;
type ReturnCode = u64;

extern "C" {
    /*
     * Register API
     */
    pub fn read_register(register_id: RegisterId, result_addr: MemoryAddress);
    pub fn register_len(register_id: RegisterId) -> u64;
    pub fn write_register(register_id: RegisterId, data_addr: MemoryAddress, data_len: u64);
    /*
     * Storage API
     */
    // 0 or 1 depending on whether anything was replaced
    pub fn storage_write(
        key_addr: MemoryAddress,
        key_len: u64,
        value_addr: MemoryAddress,
        value_len: u64,
        evicted_register_id: RegisterId,
    ) -> ReturnCode;
    // 0 or 1 depending on whether anything was read
    pub fn storage_read(
        key_addr: MemoryAddress,
        key_len: u64,
        register_id: RegisterId,
    ) -> ReturnCode;
    // 0 or 1 depending on whether anything was removed
    pub fn storage_remove(
        key_addr: MemoryAddress,
        key_len: u64,
        register_id: RegisterId,
    ) -> ReturnCode;
    // 0 or 1 depending on whether the contract has write permissions
    pub fn storage_write_perm() -> ReturnCode;
    /*
     * Context API
     */
    pub fn current_runtime_version() -> u64;
    pub fn input(result_register_id: RegisterId);
    pub fn output(output_addr: MemoryAddress, output_len: u64);
    pub fn contract_owner_address(register_id: u64);
    pub fn caller_address(register_id: u64);
    pub fn contract_instance_address(register_id: u64);
    pub fn contract_owner_address_of(
        address_ptr: MemoryAddress,
        len: u64,
        result_register_id: RegisterId,
    );
    pub fn contract_code_owner_address_of(
        address_ptr: MemoryAddress,
        len: u64,
        result_register_id: RegisterId,
    );
    pub fn contract_code_address_of(
        address_ptr: MemoryAddress,
        len: u64,
        result_register_id: RegisterId,
    );
    pub fn block_hash(output_addr: MemoryAddress, output_len: u64);
    pub fn block_number(output_addr: MemoryAddress, output_len: u64);
    pub fn block_timestamp(output_addr: MemoryAddress, output_len: u64);
    pub fn gas_limit() -> u64;
    pub fn gas_left() -> u64;

    /*
     * Economics API
     */
    pub fn address_balance(
        address_ptr: MemoryAddress,
        address_len: u64,
        result_register_id: RegisterId,
    );
    pub fn transfer_to(
        to_address_ptr: MemoryAddress,
        to_address_len: u64,
        amount_ptr: MemoryAddress,
        amount_len: u64,
    ) -> ReturnCode;
    pub fn transfer_from_caller(amount_ptr: MemoryAddress, amount_len: u64) -> ReturnCode;
    /*
     * Misc API
     */
    pub fn panic() -> !;
    pub fn panic_msg(msg_addr: MemoryAddress, msg_len: u64) -> !;
    pub fn msg(addr: MemoryAddress, len: u64);

    pub fn call_contract(
        call_addr: MemoryAddress,
        len: u64,
        result_register_id: RegisterId,
    ) -> ReturnCode;
    pub fn call_contract2(
        call_addr: MemoryAddress,
        len: u64,
        result_register_id: RegisterId,
    ) -> ReturnCode;

    pub fn emit_event_experimental(data_addr: MemoryAddress, len: u64) -> ReturnCode;
}
