use crate::{
    crc,
    hardware::{Instruction, ID},
};

pub struct Command {
    pub id: ID,
    pub instruction: Instruction,
}

impl Command {
    pub fn new(id: ID, instruction: Instruction) -> Self {
        Self { id, instruction }
    }

    pub(crate) fn build(self) -> Vec<u8> {
        let data = self.instruction.data();
        let length = (data.len() + 1)
            .try_into()
            .expect("Packet data must be within size limits.");

        let mut packet = Vec::from([0xff, 0xff, self.id.into(), length]);
        packet.extend(data);

        packet.push(crc(&packet[2..]));

        packet
    }
}
