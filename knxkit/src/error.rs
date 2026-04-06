// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

/// Top-level error type for KNX operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// KNX protocol parsing or encoding error.
    #[error("protocol error: {0}")]
    ProtocolError(#[from] crate::core::util::Error),

    /// KNXnet/IP tunneling connection error.
    #[error("tunnel error: {0}")]
    TunnelError(String),

    /// Invalid user-supplied input value.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Underlying I/O error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Operation timed out.
    #[error("timeout")]
    Timeout,

    /// General unclassified error.
    #[error("general Error: {0}")]
    General(String),
}
