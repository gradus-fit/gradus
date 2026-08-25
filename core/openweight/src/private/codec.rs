// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::api::{DocumentKind, ParseError, SerializationError, validate};
use serde::{Serialize, de::DeserializeOwned};

pub(crate) fn parse_document<T>(kind: DocumentKind, json: &str) -> Result<T, ParseError>
where
    T: DeserializeOwned,
{
    let document = serde_json::from_str(json).map_err(|error| ParseError::InvalidJson {
        message: error.to_string(),
    })?;
    let result = validate(kind, &document);

    if !result.valid {
        return Err(ParseError::SchemaValidation {
            errors: result.errors,
        });
    }

    serde_json::from_value(document).map_err(|error| ParseError::ModelDeserialization {
        message: error.to_string(),
    })
}

pub(crate) fn serialize_document<T>(
    kind: DocumentKind,
    data: &T,
    pretty: bool,
) -> Result<String, SerializationError>
where
    T: Serialize,
{
    let document =
        serde_json::to_value(data).map_err(|error| SerializationError::JsonEncoding {
            message: error.to_string(),
        })?;
    let result = validate(kind, &document);

    if !result.valid {
        return Err(SerializationError::SchemaValidation {
            errors: result.errors,
        });
    }

    if pretty {
        serde_json::to_string_pretty(&document)
    } else {
        serde_json::to_string(&document)
    }
    .map_err(|error| SerializationError::JsonEncoding {
        message: error.to_string(),
    })
}
