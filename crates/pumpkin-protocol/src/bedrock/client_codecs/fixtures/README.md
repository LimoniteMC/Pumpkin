# Golden fixtures for the client-side Bedrock codecs

Packets sent by the official Bedrock client, used to check that the `client` feature's writers
produce exactly what the real client sends.

- **Client:** official Bedrock 1.26.52 (Windows), protocol 2193, for the M2 and M5 sessions, and
  1.26.60.5, protocol 2225, for Step 12's.
  - **Since the fork took 1.26.60 (2026-11-04),** a 1.26.52 packet that changed is adjusted in its
    test. That's only `Animate`, which gained the hand at the end: the swing while building is still
    1.26.52's, since Step 12's session placed nothing.
  - **Step 12's session shows these unchanged in 1.26.60,** byte for byte: `Interact`'s open,
    `ContainerClose` and `MobEquipment`. Their files stay the M5 session's.
- **Contents:** each file is one packet body after decompression, with no length prefix and no
  packet header. That is exactly what `PacketRead::read` and `PacketWrite::write` handle.
- **Captured** with a temporary hook in `BedrockClient::process_batch` that was never committed,
  in two sessions:
  - **M2:** 2026-10-20, this fork at `80b19bb` (Limonite), offline mode, on `127.0.0.1:19132` over
    `NetherNet`. Join, stand still, walk forward, jump, quit. No clicks, so no item or block
    actions.
  - **M5:** 2026-10-26, this fork at `443b954`, the same setup, in a world with seed
    `1790672995490159855`. A scripted session of moving, building in survival and creative, the
    hotbar, the inventory, death and respawn.
  - **Step 12:** 2026-11-06, this fork at `8acb34b`, the same setup and seed. The inventory screen in
    survival and creative: taking, putting down, half a stack, swapping, shift-clicks, the creative
    list's take and its tabs, and throwing a stack away on it.

## From M2

| File | Packet | What it shows |
| --- | --- | --- |
| `resource_pack_client_response-have_all_packs.bin` | 8 | status 3, name `downloadingfinished` |
| `resource_pack_client_response-completed.bin` | 8 | status 4, name `resourcepackstackfinished` |
| `request_chunk_radius.bin` | 69 | radius 20, max 28 |
| `set_local_player_as_initialized.bin` | 113 | runtime id 1 |
| `player_auth_input-idle.bin` | 144 | standing on the ground; flags 48, 50 |
| `player_auth_input-walking.bin` | 144 | walking forward (`move_vec` 0, 1); flags 10, 48, 50 |
| `player_auth_input-jumping.bin` | 144 | first tick of a jump; flags 3, 6, 16, 31, 48, 60, 61 |

## From M5

The player's runtime id is 1 throughout.

| File | Packet | What it shows |
| --- | --- | --- |
| `player_auth_input-start_destroy.bin` | 144 | survival: `StartDestroyBlock` on dirt at (0, 200, 29), face 4; flag 35 |
| `player_auth_input-predict_destroy.bin` | 144 | the dirt broken by hand: `ContinueDestroyBlock` and `PredictDestroyBlock` together |
| `player_auth_input-abort_destroy.bin` | 144 | the button let go while on stone: `AbortDestroyBlock`, face 0 |
| `player_auth_input-creative_destroy.bin` | 144 | creative: `StartDestroyBlock` and `PredictDestroyBlock` in one tick |
| `inventory_transaction-place_on_top.bin` | 30 | dirt placed on top of a block: use item, click block (action 0, trigger 1, prediction 1), with the predicted change to hotbar slot 0 as a legacy action |
| `inventory_transaction-click_air.bin` | 30 | the use item, click air (action 1, face 255) the client sends after each placement |
| `inventory_transaction-place_on_side.bin` | 30 | a torch placed on a block's west face (face 4), from hotbar slot 2 |
| `inventory_transaction-drop.bin` | 30 | Q: a normal transaction, the slot's change and a world-source drop |
| `mob_equipment.bin` | 31 | hotbar slot 1 selected, holding oak planks |
| `interact-open_inventory.bin` | 33 | E: open inventory, target runtime id 1 |
| `interact-update.bin` | 33 | `InteractUpdate`, target 0, which often comes before `MobEquipment` |
| `player_action-start_item_use_on.bin` | 36 | the start of a placement: clicked block, placed position, face 1 |
| `player_action-stop_item_use_on.bin` | 36 | the end of a placement: placed position, face 0 |
| `player_action-creative_destroy.bin` | 36 | creative: `CreativeDestroyBlock` at (-4, 199, 30), face 1 |
| `player_action-respawn.bin` | 36 | the death screen's Respawn button: `Respawn`, face -1 |
| `animate-swing_build.bin` | 44 | arm swing, source `Build`, on each placement |
| `block_pick_request.bin` | 34 | middle click on (0, 200, 28): with data, max slots 9 |
| `container_close.bin` | 47 | the inventory screen closed: container 0, type 247, not by the server |
| `respawn.bin` | 45 | `ClientReadyToSpawn`, position (0, 0, 0) |

## From Step 12

The player's runtime id is 1 throughout. A request's stack ids are the ones the server's answers had
given each stack by then.

| File | Packet | What it shows |
| --- | --- | --- |
| `item_stack_request-take.bin` | 147 | take all 16 from hotbar slot 0 to the cursor: request -3, the session's first |
| `item_stack_request-place.bin` | 147 | place the 16 in the first slot of the bottom row, `Inventory` slot 27 (request -5) |
| `item_stack_request-take_half.bin` | 147 | the right button: take 8 of 16 from hotbar slot 1 (request -7) |
| `item_stack_request-swap.bin` | 147 | the cursor's torches put on the dirt in slot 27: a swap, the cursor first (request -13) |
| `item_stack_request-shift_click.bin` | 147 | shift-click: place 16 from hotbar slot 0 in the first empty slot above, 9, with no cursor (request -17) |
| `item_stack_request-creative_take.bin` | 147 | creative: stone from the list, as `CraftCreative` (id 604), the deprecated results, and a take of 64 from the created output, slot 50, whose stack id is the request's own (request -19) |
| `item_stack_request-creative_shift_click.bin` | 147 | creative: shift-click on hotbar slot 1, which destroys its 8 (request -23) |
| `item_stack_request-destroy.bin` | 147 | creative: the cursor's 64 put on the list, which destroys them (request -27) |
| `cursor_item_drag-start.bin` | 358 | state 0, sent with each request that puts the cursor's stack down |
| `cursor_item_drag-stop.bin` | 358 | state 1, when the button comes up after it |
| `set_player_inventory_options.bin` | 307 | the creative screen's nature tab (left tab 4), the crafting tab beside it (2), the creative layout (3) |
| `animate-swing_mine.bin` | 44 | arm swing, source `Mine`, with the hand at the end: 0, the main hand |

**The legacy byte after an action's type** keeps the numbering from before Bedrock dropped two actions:
the same number below 7, and 2 more from there. The creative take's is 12 then 14, and its results' 18
then 20.

## Not here

These packets carry the player's identity, so they are never committed. Their tests are
`#[ignore]`d, and read a local capture from an environment variable:

- **The login** (id 1), with the Xbox token: `LIMONITE_SLOGIN_FIXTURE`.
- **Commands** (id 77), with the player's UUID, and **chat** (id 9), with the Xbox id. The M5
  session's are in `run/captures/m5-official/` in the superproject, which is gitignored.
