## Serverbound Packets

Types:

- `u8`
- `u16`
- `u32`
- `bytestring` (std::string, prepended with `u16` for strlen)

---

> `0x02` - `TOSERVER_INIT`

- `max_client_serialization_version` - `u8`
- `suppr_compr_modes` - `u16`
- `min_net_proto_version` - `u16`
- `max_net_proto_version` - `u16`
- `player_name` - `bytestring`

> `0x11` - `TOSERVER_INIT2`

- (optional) `language` - `bytestring`

> `0x17` - `TOSERVER_MODCHANNEL_JOIN`

- `channel_name` - `bytestring`

> `0x18` - `TOSERVER_MODCHANNEL_LEAVE`

- `channel_name` - `bytestring`

> `0x19` - `TOSERVER_MODCHANNEL_MSG`

- `channel_name` - `bytestring`
- `channel_msg` - `bytestring`

> `0x23` - `TOSERVER_PLAYERPOS`

> `0x24` - `TOSERVER_GOTBLOCKS`

> `0x25` - `TOSERVER_DELETEDBLOCKS`

> `0x31` - `TOSERVER_INVENTORY_ACTION`

> `0x32` - `TOSERVER_CHAT_MESSAGE`

- `message` - `bytestring`

> `0x35` - `TOSERVER_DAMAGE`

> `0x37` - `TOSERVER_PLAYERITEM`

> `0x38` - `TOSERVER_RESPAWN`

> `0x39` - `TOSERVER_INTERACT`

> `0x3a` - `TOSERVER_REMOVED_SOUNDS`

> `0x3b` - `TOSERVER_NODEMETA_FIELDS`

> `0x3c` - `TOSERVER_INVENTORY_FIELDS`

> `0x40` - `TOSERVER_REQUEST_MEDIA`

> `0x43` - `TOSERVER_CLIENT_READY`

> `0x50` - `TOSERVER_FIRST_SRP`

> `0x51` - `TOSERVER_SRP_BYTES_A`

> `0x52` - `TOSERVER_SRP_BYTES_M`
