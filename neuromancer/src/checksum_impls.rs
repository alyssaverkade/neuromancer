use std::mem;

use bytes::{BufMut, Bytes, BytesMut};
use prost::Message;

use super::{base, executor, Hashable};
use crate::errors::*;

trait EncodeIntoBuffer {
    fn encode_into_buffer(&self, buffer: &mut BytesMut) -> Result<()>;
}

// Vec<T>, Option<T>, and T blanket impls don't fully overlap, forcing us to use a marker trait
// to please the trait solver
trait NeuromancerMessage: Message {}

impl<T> EncodeIntoBuffer for T
where
    T: NeuromancerMessage,
{
    fn encode_into_buffer(&self, buffer: &mut BytesMut) -> Result<()> {
        let mut result: Vec<u8> = Vec::with_capacity(mem::size_of_val(self));
        self.encode(&mut result).context(ProtobufEncodeError)?;
        buffer.extend_from_slice(&result);
        Ok(())
    }
}

impl<T> EncodeIntoBuffer for Option<T>
where
    T: NeuromancerMessage,
{
    fn encode_into_buffer(&self, buffer: &mut BytesMut) -> Result<()> {
        let mut result: Vec<u8> = Vec::with_capacity(mem::size_of_val(self));
        match &self {
            Some(message) => message.encode(&mut result).context(ProtobufEncodeError)?,
            None => result.extend_from_slice(&[0u8; 0]),
        }
        buffer.extend_from_slice(&result);
        Ok(())
    }
}

impl<T> EncodeIntoBuffer for Vec<T>
where
    T: NeuromancerMessage,
{
    fn encode_into_buffer(&self, buffer: &mut BytesMut) -> Result<()> {
        let mut result = BytesMut::new();
        for message in self.iter() {
            message.encode(&mut result).context(ProtobufEncodeError)?;
            buffer.extend_from_slice(&result);
            result.clear(); // allow for next iteration
        }
        Ok(())
    }
}

impl NeuromancerMessage for base::Identifier {}
impl NeuromancerMessage for base::Map {}
impl NeuromancerMessage for executor::ExecutionCommand {}

impl Hashable for base::Map {
    fn bytes(&self) -> Result<Bytes> {
        let mut result = BytesMut::new();
        result.extend_from_slice(self.key.as_bytes());
        result.extend_from_slice(self.value.as_bytes());
        Ok(result.freeze())
    }
}

impl Hashable for base::Reduction {
    fn bytes(&self) -> Result<Bytes> {
        let mut result = BytesMut::new();
        result.extend(self.key.as_bytes());
        result.extend(self.values.iter().flat_map(|s| s.as_bytes()));
        Ok(result.freeze())
    }
}

impl Hashable for base::RunIdentifiers {
    fn bytes(&self) -> Result<Bytes> {
        let mut result = BytesMut::new();
        self.run_ids.encode_into_buffer(&mut result)?;
        Ok(result.freeze())
    }
}

impl Hashable for executor::ExecutionCommand {
    fn bytes(&self) -> Result<Bytes> {
        let mut result = BytesMut::new();
        self.run_id.encode_into_buffer(&mut result)?;
        result.extend_from_slice(&self.program);
        Ok(result.freeze())
    }
}

impl Hashable for executor::MapRequest {
    fn bytes(&self) -> Result<Bytes> {
        let mut result = BytesMut::new();
        self.command.encode_into_buffer(&mut result)?;
        self.data.encode_into_buffer(&mut result)?;
        self.job.encode_into_buffer(&mut result)?;
        Ok(result.freeze())
    }
}

impl Hashable for executor::ReductionResult {
    fn bytes(&self) -> Result<Bytes> {
        let mut result = BytesMut::new();
        self.run_id.encode_into_buffer(&mut result)?;
        result.extend_from_slice(self.output.as_bytes());
        Ok(result.freeze())
    }
}

impl Hashable for executor::RunProgression {
    fn bytes(&self) -> Result<Bytes> {
        let mut result = BytesMut::new();
        result.put_i32_le(self.status);
        result.put_u64_le(self.time_taken);
        Ok(result.freeze())
    }
}
