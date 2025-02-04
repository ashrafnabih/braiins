// Copyright (C) 2020  Braiins Systems s.r.o.
//
// This file is part of Braiins Open-Source Initiative (BOSI).
//
// BOSI is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// Please, keep in mind that we may also license BOSI or any part thereof
// under a proprietary license. For more information on the terms and conditions
// of such proprietary license or if you have any other questions, please
// contact us at opensource@braiins.com.

//! Adapter module for connecting to stratum V2 endpoints that are secured by noise protocol
//! according to Stratum V2 specification

use std::pin::Pin;
use tokio::net::TcpStream;

use ii_async_compat::prelude::*;
use ii_logging::macros::*;
use ii_stratum::v2;

use crate::error;

/// Wrapper that establishes Stratum V2 connection
/// Note: explicitely derive Copy, so that the instance can be consumed and moved into the future.
/// All is really being copied is the public key
#[derive(Copy, Clone)]
pub(crate) struct Connector {
    /// Upstream authority public key that will be used to authenticate the endpoint
    upstream_authority_public_key: v2::noise::AuthorityPublicKey,
}

impl Connector {
    pub fn new(upstream_authority_public_key: v2::noise::AuthorityPublicKey) -> Self {
        Self {
            upstream_authority_public_key,
        }
    }

    pub async fn connect(
        self,
        connection: TcpStream,
    ) -> error::Result<(v2::DynFramedSink, v2::DynFramedStream)> {
        let noise_initiator =
            ii_stratum::v2::noise::Initiator::new(self.upstream_authority_public_key);
        trace!(
            "Stratum V2 noise connector: {:?}, {:?}",
            connection,
            noise_initiator
        );
        // Successful noise initiator handshake results in a stream/sink of V2 frames
        let (noise_sink, noise_stream) = noise_initiator.connect(connection).await?.split();
        Ok((Pin::new(Box::new(noise_sink)), noise_stream.boxed()))
    }

    /// Converts the connector into a closure that provides the connect future for later
    /// evaluation once an actual connection has been established
    pub fn into_connector_fn(self) -> super::DynConnectFn {
        Box::new(move |connection| self.connect(connection).boxed())
    }
}
