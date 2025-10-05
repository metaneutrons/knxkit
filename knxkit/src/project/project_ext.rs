// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use std::borrow::Borrow;

use super::{SharedString, DPT};
use crate::{
    core::address::{GroupAddress, IndividualAddress},
    project::Project,
};

pub trait ProjectExt {
    fn device_name(&self, address: impl Borrow<IndividualAddress>) -> Option<SharedString>;
    fn group(&self, address: impl Borrow<GroupAddress>) -> Option<SharedString>;
    fn group_dpt(&self, address: GroupAddress) -> Option<DPT>;
    //    fn decode_hex(&self, address: GroupAddress, hex: &str) -> Result<DataPoint, Error>;
}

impl ProjectExt for Option<&Project> {
    fn device_name(&self, address: impl Borrow<IndividualAddress>) -> Option<SharedString> {
        self.and_then(|project| project.devices.by_address(address).map(|d| d.name.clone()))
    }

    fn group(&self, address: impl Borrow<GroupAddress>) -> Option<SharedString> {
        self.and_then(|project| project.groups.by_address(address).map(|g| g.name.clone()))
    }

    fn group_dpt(&self, address: GroupAddress) -> Option<DPT> {
        self.and_then(|project| project.groups.by_address(address).and_then(|g| g.dpt))
    }

    /*
    fn decode_hex(&self, address: GroupAddress, hex: &str) -> Result<DataPoint, Error> {
        // self.and_then(|project|)

        unimplemented!();
    }*/
}
