// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use std::borrow::Borrow;

use knxkit::{
    core::{address::GroupAddress, DataPoint},
    project::{Project, ProjectExt},
};

pub trait ProjectExtDPT: ProjectExt {
    fn group_dpt_name(&self, address: impl Borrow<GroupAddress>) -> Option<&str>;
    fn group_dpt_unit(&self, address: impl Borrow<GroupAddress>) -> Option<&str>;
    fn group_value(&self, address: GroupAddress, dp: &DataPoint, unit: bool) -> Option<String>;
    fn group_json(&self, address: GroupAddress, dp: &DataPoint) -> Option<serde_json::Value>;
}

impl ProjectExtDPT for Option<&Project> {
    fn group_dpt_name(&self, address: impl Borrow<GroupAddress>) -> Option<&str> {
        self.and_then(|project| {
            project
                .groups
                .by_address(address)
                .and_then(|group| group.dpt.and_then(|dpt| crate::typeinfo::lookup(dpt)))
                .map(|type_info| type_info.text.unwrap_or(type_info.name))
        })
    }

    fn group_dpt_unit(&self, address: impl Borrow<GroupAddress>) -> Option<&str> {
        self.and_then(|project| {
            project
                .groups
                .by_address(address)
                .and_then(|group| group.dpt.and_then(|dpt| crate::typeinfo::lookup(dpt)))
                .and_then(|type_info| type_info.unit)
        })
    }

    fn group_json(&self, address: GroupAddress, dp: &DataPoint) -> Option<serde_json::Value> {
        self.and_then(|project| project.groups.by_address(address).and_then(|g| g.dpt))
            .and_then(|dpt| {
                crate::generic::try_decode(dpt, dp)
                    .map(|d| d.to_json_value())
                    .ok()
            })
    }

    fn group_value(&self, address: GroupAddress, dp: &DataPoint, unit: bool) -> Option<String> {
        self.and_then(|project| project.groups.by_address(address).and_then(|g| g.dpt))
            .and_then(|dpt| crate::generic::try_decode(dpt, dp).ok())
            .map(|data| {
                if unit {
                    format!(
                        "{}{}",
                        data,
                        self.group_dpt_unit(address)
                            .map(|s| s.to_string())
                            .unwrap_or(String::default())
                    )
                } else {
                    format!("{}", data)
                }
            })
    }
}
