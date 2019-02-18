// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Storage requests

use super::{Field, NoSuchOutput, OutputKind, Output};
use ethereum_types::H256;
use bytes::Bytes;

/// Potentially incomplete request for an storage proof.
#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct IncompleteRequest {
	/// Block hash to request state proof for.
	pub block_hash: Field<H256>,
	/// Hash of the account's address.
	pub address_hash: Field<H256>,
	/// Hash of the storage key.
	pub key_hash: Field<H256>,
}

impl super::IncompleteRequest for IncompleteRequest {
	type Complete = CompleteRequest;
	type Response = Response;

	fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
	where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
	{
		if let Field::BackReference(req, idx) = self.block_hash {
			f(req, idx, OutputKind::Hash)?
		}

		if let Field::BackReference(req, idx) = self.address_hash {
			f(req, idx, OutputKind::Hash)?
		}

		if let Field::BackReference(req, idx) = self.key_hash {
			f(req, idx, OutputKind::Hash)?
		}

		Ok(())
	}

	fn note_outputs<F>(&self, mut f: F) where F: FnMut(usize, OutputKind) {
		f(0, OutputKind::Hash);
	}

	fn fill<F>(&mut self, oracle: F) where F: Fn(usize, usize) -> Result<Output, NoSuchOutput> {
		if let Field::BackReference(req, idx) = self.block_hash {
			self.block_hash = match oracle(req, idx) {
				Ok(Output::Hash(block_hash)) => Field::Scalar(block_hash),
				_ => Field::BackReference(req, idx),
			}
		}

		if let Field::BackReference(req, idx) = self.address_hash {
			self.address_hash = match oracle(req, idx) {
				Ok(Output::Hash(address_hash)) => Field::Scalar(address_hash),
				_ => Field::BackReference(req, idx),
			}
		}

		if let Field::BackReference(req, idx) = self.key_hash {
			self.key_hash = match oracle(req, idx) {
				Ok(Output::Hash(key_hash)) => Field::Scalar(key_hash),
				_ => Field::BackReference(req, idx),
			}
		}
	}

	fn complete(self) -> Result<Self::Complete, NoSuchOutput> {
		Ok(CompleteRequest {
			block_hash: self.block_hash.into_scalar()?,
			address_hash: self.address_hash.into_scalar()?,
			key_hash: self.key_hash.into_scalar()?,
		})
	}

	fn adjust_refs<F>(&mut self, mut mapping: F) where F: FnMut(usize) -> usize {
		self.block_hash.adjust_req(&mut mapping);
		self.address_hash.adjust_req(&mut mapping);
		self.key_hash.adjust_req(&mut mapping);
	}
}

/// A complete request for a storage proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteRequest {
	/// Block hash to request state proof for.
	pub block_hash: H256,
	/// Hash of the account's address.
	pub address_hash: H256,
	/// Storage key hash.
	pub key_hash: H256,
}

/// The output of a request for an account state proof.
#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct Response {
	/// Inclusion/exclusion proof
	pub proof: Vec<Bytes>,
	/// Storage value.
	pub value: H256,
}

impl super::ResponseLike for Response {
	/// Fill reusable outputs by providing them to the function.
	fn fill_outputs<F>(&self, mut f: F) where F: FnMut(usize, Output) {
		f(0, Output::Hash(self.value));
	}
}