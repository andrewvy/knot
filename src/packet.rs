use nom::{be_u8, be_u16, be_u32,};

#[derive(Debug)]
pub struct Packet {
    pub protocol_id: u32,
    pub sender_peer_id: u16,
    pub channel: u8,
    pub base_packet: BasePacket,
    pub data_packet: Option<DataPacket>,
}

#[derive(Debug)]
pub enum BasePacketType {
    CONTROL,
    ORIGINAL,
    SPLIT,
    RELIABLE,
    UNKNOWN,
}

impl From<u8> for BasePacketType {
    fn from(byte: u8) -> Self {
        match byte {
            0 => BasePacketType::CONTROL,
            1 => BasePacketType::ORIGINAL,
            2 => BasePacketType::SPLIT,
            3 => BasePacketType::RELIABLE,
            _ => BasePacketType::UNKNOWN,
        }
    }
}

#[derive(Debug)]
pub enum BasePacket {
    ControlPacket {
        base_packet_type: BasePacketType,
        controltype: ControlPacketType,
        seqnum: Option<u16>,
        peer_id_new: Option<u16>,
    },
    OriginalPacket {
        base_packet_type: BasePacketType,
    },
    SplitPacket {
        base_packet_type: BasePacketType,
        seqnum: u16,
        chunk_count: u16,
        chunk_num: u16,
    },
    ReliablePacket {
        base_packet_type: BasePacketType,
        seqnum: u16,
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ControlPacketType {
    CONTROLTYPE_ACK,
    CONTROLTYPE_SET_PEER_ID,
    CONTROLTYPE_PING,
    CONTROLTYPE_DISCO,
    CONTROLTYPE_UNKNOWN,
}

impl From<u8> for ControlPacketType {
    fn from(byte: u8) -> Self {
        match byte {
            0 => ControlPacketType::CONTROLTYPE_ACK,
            1 => ControlPacketType::CONTROLTYPE_SET_PEER_ID,
            2 => ControlPacketType::CONTROLTYPE_PING,
            3 => ControlPacketType::CONTROLTYPE_DISCO,
            _ => ControlPacketType::CONTROLTYPE_UNKNOWN,
        }
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum DataPacket {
    TOSERVER_INIT {
        max_client_serialization_version: u8,
        supp_compr_modes: u16,
        min_net_proto_version: u16,
        max_net_proto_version: u16,
        player_name: String,
    }
}

named!(protocol_id<u32>, call!(be_u32));
named!(sender_peer_id<u16>, call!(be_u16));
named!(channel<u8>, call!(be_u8));

named!(control_packet_ack<BasePacket>, do_parse!(
    tag!([0x00, 0x00])
    >> seqnum: be_u16
    >> (BasePacket::ControlPacket {
        base_packet_type: BasePacketType::CONTROL,
        controltype: ControlPacketType::CONTROLTYPE_ACK,
        peer_id_new: None,
        seqnum: Some(seqnum),
    })
));

named!(control_packet_set_peer_id<BasePacket>, do_parse!(
    tag!([0x00, 0x01])
    >> peer_id_new: be_u16
    >> (BasePacket::ControlPacket {
        base_packet_type: BasePacketType::CONTROL,
        controltype: ControlPacketType::CONTROLTYPE_SET_PEER_ID,
        peer_id_new: Some(peer_id_new),
        seqnum: None,
    })
));

named!(control_packet_ping<BasePacket>, do_parse!(
    tag!([0x00, 0x02])
    >> (BasePacket::ControlPacket {
        base_packet_type: BasePacketType::CONTROL,
        controltype: ControlPacketType::CONTROLTYPE_PING,
        peer_id_new: None,
        seqnum: None,
    })
));

named!(control_packet_disco<BasePacket>, do_parse!(
    tag!([0x00, 0x03])
    >> (BasePacket::ControlPacket {
        base_packet_type: BasePacketType::CONTROL,
        controltype: ControlPacketType::CONTROLTYPE_DISCO,
        peer_id_new: None,
        seqnum: None,
    })
));

named!(control_packet<BasePacket>, alt!(
    control_packet_ack |
    control_packet_set_peer_id |
    control_packet_ping |
    control_packet_disco
));

named!(original_packet<BasePacket>, do_parse!(
    tag!([0x01])
    >> (BasePacket::OriginalPacket {
        base_packet_type: BasePacketType::ORIGINAL,
    })
));

named!(split_packet<BasePacket>, do_parse!(
    tag!([0x02])
    >> seqnum: be_u16
    >> chunk_count: be_u16
    >> chunk_num: be_u16
    >> (BasePacket::SplitPacket {
        base_packet_type: BasePacketType::SPLIT,
        seqnum,
        chunk_count,
        chunk_num,
    })
));

named!(reliable_packet<BasePacket>, do_parse!(
    tag!([0x03])
    >> seqnum: be_u16
    >> (BasePacket::ReliablePacket {
        base_packet_type: BasePacketType::RELIABLE,
        seqnum
    })
));

named!(base_packet<BasePacket>, alt!(
    control_packet |
    original_packet |
    split_packet |
    reliable_packet
));

named!(data_packet<DataPacket>, do_parse!(
    tag!([0x00, 0x02])
    >> max_client_serialization_version: be_u8
    >> supp_compr_modes: be_u16
    >> min_net_proto_version: be_u16
    >> max_net_proto_version: be_u16
    >> player_name: map!(length_bytes!(be_u16), |name| String::from_utf8(name.to_vec()).unwrap_or("".to_string()))
    >> (DataPacket::TOSERVER_INIT {
        max_client_serialization_version,
        supp_compr_modes,
        min_net_proto_version,
        max_net_proto_version,
        player_name,
    })
));

named!(pub packet<Packet>, do_parse!(
    protocol_id: protocol_id
    >> sender_peer_id: sender_peer_id
    >> channel: channel
    >> base_packet: base_packet
    >> data_packet: opt!(data_packet)
    >> (Packet {
        protocol_id,
        sender_peer_id,
        channel,
        base_packet,
        data_packet,
    })
));
