// SPDX-License-Identifier: AGPL-3.0-or-later

use gradus_module_template::{Template, create};

#[test]
fn create_returns_the_public_template_type() {
    assert_eq!(create(), Template);
}
