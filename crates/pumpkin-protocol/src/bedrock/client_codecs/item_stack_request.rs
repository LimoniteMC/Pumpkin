use std::io::{Error, ErrorKind, Write};

use crate::{
    bedrock::server::item_stack_request::{
        ItemStackRequest, ItemStackRequestAction, SItemStackRequest, StackRequestItem,
    },
    codec::var_uint::VarUInt,
    serial::PacketWrite,
};

/// Writes an action's type twice: as a `VarUInt`, then as the byte Pumpkin's
/// reader skips as the legacy action type.
///
/// The byte keeps the numbering from before Bedrock dropped two actions, so
/// from 7 on it's 2 more than the type. The official client sends 0, 1 and 2
/// twice for take, place and swap, and 12 then 14 for the creative take, 18
/// then 20 for its results (the M5 and Step 12 captures).
fn action_type<W: Write>(writer: &mut W, action_type: u8) -> Result<(), Error> {
    VarUInt(u32::from(action_type)).write(writer)?;
    let legacy = if action_type < 7 {
        action_type
    } else {
        action_type + 2
    };
    legacy.write(writer)
}

/// Mirrors `PacketRead for StackRequestItem`: the descriptor's type as a
/// `VarUInt` and again as a byte (1 with an identifier, 0 without); the
/// identifier and the metadata, if there's an identifier; the count as an
/// `i16`, the block runtime id, and the extra data with a `VarUInt` length.
impl PacketWrite for StackRequestItem {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let descriptor_type = u8::from(self.identifier.is_some());
        VarUInt(u32::from(descriptor_type)).write(writer)?;
        descriptor_type.write(writer)?;
        if let Some(identifier) = &self.identifier {
            identifier.write(writer)?;
            self.metadata_value.write(writer)?;
        }
        (self.count as i16).write(writer)?;
        self.block_runtime_id.write(writer)?;
        VarUInt(self.extra_data.len() as u32).write(writer)?;
        writer.write_all(&self.extra_data)
    }
}

/// Mirrors `PacketRead for ItemStackRequestAction`, for the actions the client
/// sends: take, place, swap, drop, destroy, consume, create, and the creative
/// take (`CraftCreative`) with the results the official client sends after it
/// (`CraftResultsDeprecated`).
///
/// The others are refused: the client doesn't send them, and for some the
/// reader drops data, such as an auto-craft's ingredients, so no writer could
/// match it.
impl PacketWrite for ItemStackRequestAction {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        match self {
            Self::Take {
                count,
                source,
                destination,
            } => {
                action_type(writer, 0)?;
                count.write(writer)?;
                source.write(writer)?;
                destination.write(writer)
            }
            Self::Place {
                count,
                source,
                destination,
            } => {
                action_type(writer, 1)?;
                count.write(writer)?;
                source.write(writer)?;
                destination.write(writer)
            }
            Self::Swap { slot1, slot2 } => {
                action_type(writer, 2)?;
                slot1.write(writer)?;
                slot2.write(writer)
            }
            Self::Drop {
                count,
                source,
                randomly,
            } => {
                action_type(writer, 3)?;
                count.write(writer)?;
                source.write(writer)?;
                randomly.write(writer)
            }
            Self::Destroy { count, source } => {
                action_type(writer, 4)?;
                count.write(writer)?;
                source.write(writer)
            }
            Self::Consume { count, source } => {
                action_type(writer, 5)?;
                count.write(writer)?;
                source.write(writer)
            }
            Self::Create { result_index } => {
                action_type(writer, 6)?;
                result_index.write(writer)
            }
            Self::CraftCreative {
                creative_item_id,
                repetitions,
            } => {
                action_type(writer, 12)?;
                creative_item_id.write(writer)?;
                repetitions.write(writer)
            }
            Self::CraftResultsDeprecated {
                result_items,
                times_crafted,
            } => {
                action_type(writer, 18)?;
                VarUInt(result_items.len() as u32).write(writer)?;
                for item in result_items {
                    item.write(writer)?;
                }
                times_crafted.write(writer)
            }
            other => Err(Error::new(
                ErrorKind::Unsupported,
                format!("the client can't send the item stack request action {other:?}"),
            )),
        }
    }
}

/// Mirrors `PacketRead for ItemStackRequest`: the request id; a `VarUInt`
/// count and the actions; a `VarUInt` count and the filter strings; then the
/// filter cause as a little-endian `i32`.
impl PacketWrite for ItemStackRequest {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.request_id.write(writer)?;
        VarUInt(self.actions.len() as u32).write(writer)?;
        for action in &self.actions {
            action.write(writer)?;
        }
        VarUInt(self.filter_strings.len() as u32).write(writer)?;
        for filter in &self.filter_strings {
            filter.write(writer)?;
        }
        self.filter_cause.write(writer)
    }
}

/// Mirrors `PacketRead for SItemStackRequest`: a `VarUInt` count, then the
/// requests.
impl PacketWrite for SItemStackRequest {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(self.requests.len() as u32).write(writer)?;
        for request in &self.requests {
            request.write(writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bedrock::{
            network_item::{ContainerName, FullContainerName},
            server::item_stack_request::ItemStackRequestSlotInfo,
        },
        codec::{var_int::VarInt, var_uint::VarUInt},
        serial::PacketRead,
    };

    /// Reads a capture with Pumpkin's reader, then writes it back with ours.
    fn golden(fixture: &[u8]) -> SItemStackRequest {
        let mut reader = fixture;
        let packet = SItemStackRequest::read(&mut reader).unwrap();
        assert!(reader.is_empty(), "{} bytes left over", reader.len());

        let mut written = Vec::new();
        packet.write(&mut written).unwrap();
        assert_eq!(written, fixture);
        packet
    }

    /// The one request in a capture, which is all the official client ever sent in one.
    fn only(packet: &SItemStackRequest) -> &ItemStackRequest {
        assert_eq!(packet.requests.len(), 1);
        &packet.requests[0]
    }

    fn slot(container_name: ContainerName, slot_id: u8, stack_id: i32) -> ItemStackRequestSlotInfo {
        ItemStackRequestSlotInfo {
            container_name: FullContainerName {
                container_name,
                dynamic_id: None,
            },
            slot_id,
            stack_id,
        }
    }

    /// A slot's container, its slot and its stack id, to compare in one go.
    fn at(slot: &ItemStackRequestSlotInfo) -> (ContainerName, u8, i32) {
        (slot.container_name.container_name, slot.slot_id, slot.stack_id)
    }

    #[test]
    fn a_take_matches_the_official_client() {
        let packet = golden(include_bytes!("fixtures/item_stack_request-take.bin"));
        let request = only(&packet);
        assert_eq!(request.request_id, VarInt(-3), "the first request of a session");
        let ItemStackRequestAction::Take {
            count,
            source,
            destination,
        } = &request.actions[0]
        else {
            panic!("expected a take");
        };
        assert_eq!(*count, 16, "the whole stack");
        assert_eq!(at(source), (ContainerName::HotBar, 0, 8));
        assert_eq!(at(destination), (ContainerName::Cursor, 0, 0));
        assert!(request.filter_strings.is_empty());
        assert_eq!(request.filter_cause, -1);
    }

    #[test]
    fn a_place_matches_the_official_client() {
        let packet = golden(include_bytes!("fixtures/item_stack_request-place.bin"));
        let request = only(&packet);
        assert_eq!(request.request_id, VarInt(-5), "ids fall by 2");
        let ItemStackRequestAction::Place {
            count,
            source,
            destination,
        } = &request.actions[0]
        else {
            panic!("expected a place");
        };
        assert_eq!(*count, 16);
        // The cursor's stack id is the one the server gave it in its response.
        assert_eq!(at(source), (ContainerName::Cursor, 0, 15));
        // The 27 slots above the hotbar are slots 9 to 35 of `Inventory`.
        assert_eq!(at(destination), (ContainerName::Inventory, 27, 0));
    }

    #[test]
    fn taking_half_matches_the_official_client() {
        let packet = golden(include_bytes!("fixtures/item_stack_request-take_half.bin"));
        assert!(matches!(
            &only(&packet).actions[0],
            ItemStackRequestAction::Take { count: 8, source, .. } if at(source) == (ContainerName::HotBar, 1, 10)
        ));
    }

    #[test]
    fn a_swap_matches_the_official_client() {
        let packet = golden(include_bytes!("fixtures/item_stack_request-swap.bin"));
        let ItemStackRequestAction::Swap { slot1, slot2 } = &only(&packet).actions[0] else {
            panic!("expected a swap");
        };
        assert_eq!(at(slot1), (ContainerName::Cursor, 0, 19));
        assert_eq!(at(slot2), (ContainerName::Inventory, 27, 16));
    }

    #[test]
    fn a_shift_click_matches_the_official_client() {
        let packet = golden(include_bytes!("fixtures/item_stack_request-shift_click.bin"));
        // From the hotbar to the first empty slot above it, in one place: no cursor.
        let ItemStackRequestAction::Place {
            count,
            source,
            destination,
        } = &only(&packet).actions[0]
        else {
            panic!("expected a place");
        };
        assert_eq!(*count, 16);
        assert_eq!(at(source), (ContainerName::HotBar, 0, 20));
        assert_eq!(at(destination), (ContainerName::Inventory, 9, 0));
    }

    #[test]
    fn a_creative_take_matches_the_official_client() {
        let packet = golden(include_bytes!("fixtures/item_stack_request-creative_take.bin"));
        let request = only(&packet);
        assert_eq!(request.request_id, VarInt(-19));
        let [craft, results, take] = &request.actions[..] else {
            panic!("expected three actions");
        };
        assert!(matches!(
            craft,
            ItemStackRequestAction::CraftCreative {
                creative_item_id: VarUInt(604),
                repetitions: 1
            }
        ));
        let ItemStackRequestAction::CraftResultsDeprecated {
            result_items,
            times_crafted: 1,
        } = results
        else {
            panic!("expected the results");
        };
        let item = &result_items[..];
        assert_eq!(item.len(), 1);
        assert_eq!(item[0].identifier.as_deref(), Some("minecraft:stone"));
        assert_eq!((item[0].metadata_value, item[0].count), (VarInt(0), 1));
        // No NBT, and nothing it can be placed on or break: 10 bytes of zeros.
        assert_eq!(item[0].extra_data, [0; 10]);
        let ItemStackRequestAction::Take {
            count,
            source,
            destination,
        } = take
        else {
            panic!("expected a take");
        };
        assert_eq!(*count, 64, "a whole stack");
        // The created item's stack id is the request's own id: it has no other yet.
        assert_eq!(at(source), (ContainerName::CreatedOutput, 50, -19));
        assert_eq!(at(destination), (ContainerName::Cursor, 0, 0));
    }

    #[test]
    fn a_creative_shift_click_on_the_hotbar_destroys_it() {
        let packet = golden(include_bytes!("fixtures/item_stack_request-creative_shift_click.bin"));
        assert!(matches!(
            &only(&packet).actions[0],
            ItemStackRequestAction::Destroy { count: 8, source } if at(source) == (ContainerName::HotBar, 1, 10)
        ));
    }

    #[test]
    fn dropping_the_cursor_on_the_creative_list_destroys_it() {
        let packet = golden(include_bytes!("fixtures/item_stack_request-destroy.bin"));
        assert!(matches!(
            &only(&packet).actions[0],
            ItemStackRequestAction::Destroy { count: 64, source } if at(source) == (ContainerName::Cursor, 0, 25)
        ));
    }

    #[test]
    fn the_other_sent_actions_round_trip() {
        let packet = SItemStackRequest {
            requests: vec![ItemStackRequest {
                request_id: VarInt(-21),
                actions: vec![
                    ItemStackRequestAction::CraftCreative {
                        creative_item_id: VarUInt(117),
                        repetitions: 1,
                    },
                    ItemStackRequestAction::CraftResultsDeprecated {
                        result_items: vec![StackRequestItem {
                            identifier: None,
                            metadata_value: VarInt(0),
                            count: 3,
                            block_runtime_id: VarUInt(0),
                            extra_data: vec![1, 2],
                        }],
                        times_crafted: 2,
                    },
                    ItemStackRequestAction::Drop {
                        count: 2,
                        source: slot(ContainerName::HotBar, 3, 40),
                        randomly: true,
                    },
                    ItemStackRequestAction::Consume {
                        count: 1,
                        source: slot(ContainerName::Inventory, 9, 42),
                    },
                    ItemStackRequestAction::Create { result_index: 2 },
                ],
                filter_strings: vec!["a filter".to_string()],
                filter_cause: 3,
            }],
        };
        let mut first = Vec::new();
        packet.write(&mut first).unwrap();
        let mut reader = first.as_slice();
        let read = SItemStackRequest::read(&mut reader).unwrap();
        assert!(reader.is_empty(), "{} bytes left over", reader.len());

        let request = &read.requests[0];
        assert_eq!(request.request_id, VarInt(-21));
        assert!(matches!(
            &request.actions[1],
            ItemStackRequestAction::CraftResultsDeprecated { result_items, times_crafted: 2 }
                if result_items[0].identifier.is_none() && result_items[0].extra_data == [1, 2]
        ));
        assert!(matches!(
            &request.actions[2],
            ItemStackRequestAction::Drop { count: 2, randomly: true, source } if source.stack_id == 40
        ));
        assert!(matches!(
            request.actions[4],
            ItemStackRequestAction::Create { result_index: 2 }
        ));
        assert_eq!(request.filter_strings, ["a filter"]);
        assert_eq!(request.filter_cause, 3);

        let mut second = Vec::new();
        read.write(&mut second).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn an_action_the_client_never_sends_is_refused() {
        let error = ItemStackRequestAction::LabTableCombine
            .write(&mut Vec::new())
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported);
    }
}
