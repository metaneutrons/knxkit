// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use std::{future::Future, sync::Arc};

use tokio::sync::Notify;

use crate::{
    core::{address::IndividualAddress, cemi::CEMI},
    error::Error,
};

/// Connection multiplexing support.
pub mod multiplex;
/// Group and filter operations on KNX bus connections.
pub mod ops;
/// Remote connection target parsing and connection establishment.
pub mod remote;

pub use remote::{connect, parse_remote, RemoteSpec};

/// Trait for KNX bus connections (tunneling, routing, etc.).
pub trait KnxBusConnection {
    /// Send a CEMI frame to the bus, returning a notify handle for acknowledgement.
    fn send(&self, cemi: CEMI) -> impl Future<Output = Result<Arc<Notify>, Error>> + Send;
    /// Receive the next incoming CEMI frame, or `None` if the connection is closed.
    fn recv(&mut self) -> impl Future<Output = Option<Arc<CEMI>>> + Send;
    /// Gracefully shut down the connection.
    fn terminate(self) -> impl Future<Output = ()> + Send;
    /// Return the individual address assigned to this connection.
    fn address(&self) -> IndividualAddress;
}
