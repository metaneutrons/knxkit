// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

/// KNX/IP device description.
pub mod describe;
/// UDP endpoint for KNX/IP communication.
pub mod endpoint_udp;
/// KNX/IP frame definitions and parsing.
pub mod frames;
/// KNX/IP device search via multicast.
pub mod search;
/// KNX/IP tunneling (point-to-point) connection.
pub mod tunnel;
