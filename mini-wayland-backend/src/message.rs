use crate::message::reader::Reader;
use crate::registry::object_type::ObjectType;
use crate::registry::{ObjectId, Registry};
use crate::stream::RawMessageHeader;

mod reader;

#[derive(Debug, Clone)]
pub struct Message {
    pub object_id: ObjectId,
    pub contents: MessageContents,
}

impl Message {
    pub fn parse(header: RawMessageHeader, data: &[u8], registry: &Registry) -> Self {
        dbg!(header);
        let object_id = ObjectId::new(header.object_id).unwrap(); // TODO: panics
        let object_type = registry.get_object_type(object_id).unwrap();
        dbg!(object_type);

        let contents = match object_type {
            ObjectType::WlDisplay => {
                MessageContents::WlDisplay(WlDisplayMessage::parse(header, data).unwrap())
            }
            ObjectType::WlRegistry => {
                MessageContents::WlRegistry(WlRegistryMessage::parse(header, data).unwrap())
            }
        };

        Message {
            object_id,
            contents,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MessageContents {
    WlDisplay(WlDisplayMessage),
    WlRegistry(WlRegistryMessage),
}

#[derive(Debug, Clone)]
pub enum WlDisplayMessage {
    Error(WlDisplayEventError),
}

impl WlDisplayMessage {
    pub fn parse(header: RawMessageHeader, data: &[u8]) -> Option<Self> {
        Some(match header.opcode {
            0 => Self::Error(WlDisplayEventError::parse(Reader::new(data))),
            c => todo!("Unsupported opcode: {}", c),
        })
    }
}

#[derive(Debug, Clone)]
pub struct WlDisplayEventError {
    pub object_id: Option<ObjectId>,
    pub error_code: u32,
    pub message: Option<String>,
}

impl WlDisplayEventError {
    pub fn parse(mut reader: Reader) -> Self {
        Self {
            object_id: reader.read_object_id(),
            error_code: reader.read_uint(),
            message: reader.read_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum WlRegistryMessage {
    Global(WlRegistryEventGlobal),
}

impl WlRegistryMessage {
    pub fn parse(header: RawMessageHeader, data: &[u8]) -> Option<Self> {
        Some(match header.opcode {
            0 => Self::Global(WlRegistryEventGlobal::parse(Reader::new(data))),
            c => todo!("Unsupported opcode: {}", c),
        })
    }
}

#[derive(Debug, Clone)]
pub struct WlRegistryEventGlobal {
    name: ObjectId,
    interface: String,
    version: u32,
}

impl WlRegistryEventGlobal {
    pub fn parse(mut reader: Reader) -> Self {
        Self {
            name: reader.read_object_id().unwrap(),
            interface: reader.read_str().unwrap(),
            version: reader.read_uint(),
        }
    }
}
