// Last verified for v2225

#[cfg(feature = "client")]
use crate::serial::PacketWrite;
use crate::serial::PacketRead;
use pumpkin_macros::packet;

pub const CURSOR_ITEM_DRAG_START: u8 = 0;
pub const CURSOR_ITEM_DRAG_STOP: u8 = 1;

/// Sent by the client to signal the start and stop of a cursor item drag operation across inventory slots.
///
/// Limonite: the client writes it too, around each click that puts the cursor's
/// stack down, as the official client does.
#[derive(PacketRead)]
#[cfg_attr(feature = "client", derive(Debug, PacketWrite))]
#[packet(358)]
pub struct SCursorItemDrag {
    /// Drag operation state (`0` for start, `1` for stop).
    pub state: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_item_drag_reads_state() {
        let mut buf = &[CURSOR_ITEM_DRAG_START][..];
        let packet = SCursorItemDrag::read(&mut buf).unwrap();
        assert_eq!(packet.state, CURSOR_ITEM_DRAG_START);
    }
}
