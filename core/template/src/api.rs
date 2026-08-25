// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::private::template;

/// Placeholder public type to replace with the module's domain type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Template;

/// Placeholder public operation to replace with the module's public API.
pub fn create() -> Template {
    template()
}
