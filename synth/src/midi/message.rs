#[derive(Debug, Copy, Clone)]
pub enum MessageKind {
    NoteOn,
    NoteOff,
    ControlChange,
    ProgramChange,
}

#[derive(Debug, Copy, Clone)]
pub struct Message {
    pub channel: u8,
    pub kind: MessageKind,
    pub identifier: u8,
    pub value: u8,
}

impl Message {
    pub fn frequency(&self) -> Result<f32, String> {
        match self.kind {
            MessageKind::NoteOn => Ok(to_freq(self.identifier)),
            MessageKind::NoteOff => Ok(to_freq(self.identifier)),
            _ => Err(format!(
                "MIDI message type has no frequency: {:?}",
                self.kind
            )),
        }
    }
}

pub fn try_parse(data: &[u8; 3]) -> Result<Message, String> {
    let [status, identifier, value] = *data;
    let raw_kind = status & 0xF0;
    let kind = match raw_kind {
        0x80 => MessageKind::NoteOff,
        0x90 => MessageKind::NoteOn,
        0xB0 => MessageKind::ControlChange,
        0xC0 => MessageKind::ProgramChange,
        _ => return Err(format!("Unsupported MIDI message type: {:#X}", raw_kind)),
    };
    let channel = status & 0x0F;
    Ok(Message {
        channel,
        kind,
        identifier,
        value,
    })
}

fn to_freq(identifier: u8) -> f32 {
    440.0 * (2.0f32).powf((identifier as f32 - 69.0) / 12.0)
}
