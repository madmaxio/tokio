use crate::codec::RecvError;
use crate::frame;
use crate::proto::*;
use std::task::{Context, Poll};

#[derive(Debug)]
pub(crate) struct Settings {
    /// Received SETTINGS frame pending processing. The ACK must be written to
    /// the socket first then the settings applied **before** receiving any
    /// further frames.
    pending: Option<frame::Settings>,
}

impl Settings {
    pub fn new() -> Self {
        Settings { pending: None }
    }

    pub fn recv_settings(&mut self, frame: frame::Settings) {
        if frame.is_ack() {
            log::debug!("received remote settings ack");
        // TODO: handle acks
        } else {
            assert!(self.pending.is_none());
            self.pending = Some(frame);
        }
    }

    pub fn send_pending_ack<T, B, C, P>(
        &mut self,
        cx: &mut Context,
        dst: &mut Codec<T, B>,
        streams: &mut Streams<C, P>,
    ) -> Poll<Result<(), RecvError>>
    where
        T: AsyncWrite + Unpin,
        B: Buf + Unpin,
        C: Buf + Unpin,
        P: Peer,
    {
        log::trace!("send_pending_ack; pending={:?}", self.pending);

        if let Some(settings) = &self.pending {
            if !dst.poll_ready(cx)?.is_ready() {
                log::trace!("failed to send ACK");
                return Poll::Pending;
            }

            // Create an ACK settings frame
            let frame = frame::Settings::ack();

            // Buffer the settings frame
            dst.buffer(frame.into()).expect("invalid settings frame");

            log::trace!("ACK sent; applying settings");

            if let Some(val) = settings.max_frame_size() {
                dst.set_max_send_frame_size(val as usize);
            }

            streams.apply_remote_settings(settings)?;
        }

        self.pending = None;

        Poll::Ready(Ok(()))
    }
}
