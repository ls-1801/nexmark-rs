use crate::event::{Event};
use byteorder::ByteOrder;
use std::io::Write;

#[derive(bytemuck::NoUninit, Clone, Copy)]
#[repr(C)]
pub struct BinaryBid {
    creation_ts: u64,
    auction_id: u64,
    bidder_id: u64,
    timestamp: u64,
    price: f64,
}

impl From<Event> for BinaryBid {
    fn from(event: Event) -> Self {
        match event {
            Event::Person(_) => todo!(),
            Event::Auction(_) => todo!(),
            Event::Bid(value) => BinaryBid {
                creation_ts: value.date_time,
                auction_id: value.auction as u64,
                bidder_id: value.bidder as u64,
                timestamp: value.timestamp as u64,
                price: value.price,
            },
        }
    }
}

pub struct BinaryWriter<'a, T: Write> {
    writer: &'a mut T,
    buffer: Vec<u8>,
    current: usize,
}

impl<'a, T: Write> BinaryWriter<'a, T> {
    fn any_as_u8_slice<D: bytemuck::NoUninit>(p: &D) -> &[u8] {
        bytemuck::bytes_of(p)
    }

    pub fn new(t: &'a mut T, buffer_size: u64) -> Self {
        Self {
            writer: t,
            buffer: vec![0u8; buffer_size as usize],
            current: 0,
        }
    }

    pub fn write_buffer<D: bytemuck::NoUninit>(&mut self, pod: &D) -> std::io::Result<()> {
        let new_data_buffer = Self::any_as_u8_slice(pod);
        if self.current + new_data_buffer.len() < self.buffer.len() {
            self.buffer.as_mut_slice()[self.current..new_data_buffer.len() + self.current]
                .clone_from_slice(new_data_buffer);
            self.current += new_data_buffer.len();
            Ok(())
        } else {
            self.flush()?;
            self.write_buffer(pod)
        }
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        if self.current == 0 {
            return Ok(());
        }

        let mut number_of_bytes = [0u8; 8];
        byteorder::LittleEndian::write_u64_into(&[self.current as u64], &mut number_of_bytes);
        self.writer.write_all(&number_of_bytes)?;
        self.writer.write_all(self.buffer.as_slice())?;
        self.writer.flush()?;
        self.current = 0;

        Ok(())
    }
}
