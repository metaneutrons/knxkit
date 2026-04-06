// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use knxkit::{core::DataPoint, project::DPT};

/// Errors that can occur when encoding or decoding datapoints.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The binary payload does not match the expected DPT encoding.
    #[error("Invalid data point value: {0}")]
    InvalidDataPointValue(DataPoint),

    /// The requested DPT is not supported or unknown.
    #[error("Invalid DPT: {0}")]
    InvalidDPT(DPT),

    /// An I/O error occurred during encoding or decoding.
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    /// A JSON serialization/deserialization error occurred.
    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde_json::Error),
}
