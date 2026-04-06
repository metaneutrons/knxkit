// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use std::fmt::Display;

use anyhow::Result;

use tokio::signal::unix::{signal, Signal, SignalKind};

use knxkit::connection::{remote::RemoteSpec, KnxBusConnection};

use crate::CLI;

pub async fn interrupt() -> std::io::Result<Signal> {
    signal(SignalKind::interrupt())
}

pub async fn connect(remote: &RemoteSpec) -> Result<impl KnxBusConnection> {
    Ok(knxkit::connection::remote::connect(CLI.globals.local_address, remote).await?)
}

pub trait Defaults {
    /// Format as display or "-" if None.
    fn unwrap_or_missing(self) -> impl Display;
}

impl<T: Display> Defaults for Option<T> {
    fn unwrap_or_missing(self) -> impl Display {
        self.map(|s| s.to_string()).unwrap_or("-".to_string())
    }
}

#[macro_export]
macro_rules! match_variant {
    ($p:pat = $e:expr => $s:stmt) => {
        #[allow(irrefutable_let_patterns)]
        if let $p = $e {
            $s
        } else {
            panic!("invalid pattern")
        }
    };
}
