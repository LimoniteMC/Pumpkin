// Last verified for v2169

#[cfg(feature = "client")]
use crate::serial::PacketWrite;
use crate::{codec::var_int::VarInt, serial::PacketRead};
use pumpkin_macros::packet;

/// Sent by the client to sync inventory UI layout options and selected crafting tabs.
///
/// Limonite: the client writes it too, when the creative screen changes tab, as
/// the official client does.
#[derive(PacketRead)]
#[cfg_attr(feature = "client", derive(Debug, PacketWrite))]
#[packet(307)]
pub struct SSetPlayerInventoryOptions {
    // TODO: enum InventoryLeftTabIndex
    /// Selected left inventory category tab.
    pub left_inventory_tab: VarInt,
    // TODO: enum InventoryRightTabIndex
    /// Selected right inventory category tab.
    pub right_inventory_tab: VarInt,

    /// Whether recipe filtering (craftable only) is toggled on.
    pub filtering: bool,

    // TODO: enum InventoryLayout
    /// Layout style of the inventory grid.
    pub layout_inv: VarInt,
    // TODO: enum InventoryLayout
    /// Layout style of the crafting section.
    pub layout_craft: VarInt,
}
