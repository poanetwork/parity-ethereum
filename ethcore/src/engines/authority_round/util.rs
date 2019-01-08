//! Utility functions.
//!
//! Contains small functions used by the AuRa engine that are not strictly limited to that scope.

use std::fmt;

use ethabi;
use ethereum_types::Address;

use client::{BlockId, EngineClient};
use types::transaction;

/// A contract bound to a client and block number.
///
/// A bound contract is a combination of a `Client` reference, a `BlockId` and a contract `Address`.
/// These three parts are enough to call a contract's function; return values are automatically
/// decoded.
pub struct BoundContract<'a> {
	client: &'a EngineClient,
	block_id: BlockId,
	contract_addr: Address,
}

/// Contract call failed error.
#[derive(Debug)]
pub enum CallError {
	/// The call itself failed.
	CallFailed(String),
	/// Decoding the return value failed or the decoded value was a failure.
	DecodeFailed(ethabi::Error),
	/// The passed in client reference could not be upgraded to a `BlockchainClient`.
	NotFullClient,
	/// The transaction required to make a call could not be scheduled.
	TransactionFailed(transaction::Error),
}

impl<'a> fmt::Debug for BoundContract<'a> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("BoundContract")
			.field("client", &(self.client as *const EngineClient))
			.field("block_id", &self.block_id)
			.field("contract_addr", &self.contract_addr)
			.finish()
	}
}

impl<'a> BoundContract<'a> {
	/// Create a new `BoundContract`.
	#[inline]
	pub fn bind(client: &EngineClient, block_id: BlockId, contract_addr: Address) -> BoundContract {
		BoundContract {
			client,
			block_id,
			contract_addr,
		}
	}

	/// Perform a function call to an ethereum machine.
	///
	/// Runs a constant function call on `client`. The `call` value can be serialized by calling any
	/// api function generated by the `use_contract!` macro.
	pub fn call_const<D>(&self, call: (ethabi::Bytes, D)) -> Result<D::Output, CallError>
	where
		D: ethabi::FunctionOutputDecoder,
	{
		let (data, output_decoder) = call;

		let call_return = self
			.client
			.as_full_client()
			.ok_or(CallError::NotFullClient)?
			.call_contract(self.block_id, self.contract_addr, data)
			.map_err(CallError::CallFailed)?;

		// Decode the result and return it.
		output_decoder
			.decode(call_return.as_slice())
			.map_err(CallError::DecodeFailed)
	}

	/// Schedules a transaction that calls a contract.
	///
	/// Causes `client` to schedule a call to the bound contract. The `call` value can be serialized
	/// by calling any api function generated by the `use_contract!` macro.
	pub fn schedule_call_transaction<D>(&self, call: (ethabi::Bytes, D)) -> Result<(), CallError> {
		// NOTE: The second item of `call` is actually meaningless, since the function will only be
		//       executed later on when the transaction is processed. For this reason, there is no
		//       `ethabi::FunctionOutputDecoder` trait bound on it, even though the `use_contract`
		//       macro generates these for constant and non-constant functions the same way.
		let (data, _) = call;

		let cl = self
			.client
			.as_full_client()
			.ok_or(CallError::NotFullClient)?;

		cl.transact_contract(self.contract_addr, data)
			.map_err(CallError::TransactionFailed)
	}
}
