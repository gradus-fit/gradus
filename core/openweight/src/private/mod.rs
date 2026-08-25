// SPDX-License-Identifier: AGPL-3.0-or-later

mod codec;
mod validation;

pub(crate) use codec::{parse_document, serialize_document};
pub(crate) use validation::validate_document;
