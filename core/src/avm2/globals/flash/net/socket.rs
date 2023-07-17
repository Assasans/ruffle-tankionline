//! `flash.net.Socket` native function definitions

use crate::avm2::activation::Activation;
use crate::avm2::error::security_error;
use crate::avm2::object::{OutgoingSocketAction, TObject};
use crate::avm2::value::Value;
use crate::avm2::{Error, Object};

pub use crate::avm2::object::socket_allocator;
use crate::avm2::parameters::ParametersExt;

/// Native function definition for `Socket.connect`
pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_socket() {
        let host = args.get_string(activation, 0)?;
        let port = args.get_u32(activation, 1)?;
        if port > 65535 {
            return Err(Error::AvmError(security_error(
                activation,
                "Error #2003: Invalid socket port number specified.",
                2003,
            )?));
        }

        let future = activation.context.load_manager.load_socket(
            activation.context.player.clone(),
            this,
            (host.to_string(), port as u16),
        );
        activation.context.navigator.spawn_future(future);

        return Ok(Value::Undefined);
    }
    Err("Socket.prototype.connect has been called on an incompatible object".into())
}

/// Native function definition for `Socket.writeBytes`
pub fn write_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_socket() {
        let bytes = args.get_object(activation, 0, "bytes")?;
        let bytes = bytes
            .as_bytearray()
            .ok_or("ArgumentError: Parameter must be a ByteArray")?;
        let offset = args.get_u32(activation, 1)?;
        let length = args.get_u32(activation, 2)?;

        tracing::debug!(
            "Socket.writeBytes: {:?} offset={} len={}",
            bytes,
            offset,
            length
        );

        let queue = match this.outgoing_queue() {
            Some(queue) => Ok(queue),
            None => Err(Error::AvmError(security_error(
                activation,
                "Error #2031: Socket Error.",
                2031,
            )?)),
        }?;
        let queue = queue.read();
        match queue.send(OutgoingSocketAction::Write(bytes.bytes().to_vec())) {
            Ok(_) => {}
            Err(_) => {
                return Err(Error::AvmError(security_error(
                    activation,
                    "Error #2031: Socket Error.",
                    2031,
                )?))
            }
        }

        return Ok(Value::Undefined);
    }
    Err("Socket.prototype.writeBytes has been called on an incompatible object".into())
}

/// Native function definition for `Socket.writeByte`
pub fn write_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_socket() {
        let byte = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)?;
        tracing::debug!("Socket.writeByte: {:?}", byte);

        let queue = match this.outgoing_queue() {
            Some(queue) => Ok(queue),
            None => Err(Error::AvmError(security_error(
                activation,
                "Error #2031: Socket Error.",
                2031,
            )?)),
        }?;
        let queue = queue.read();
        match queue.send(OutgoingSocketAction::Write(vec![byte as u8])) {
            Ok(_) => {}
            Err(_) => {
                return Err(Error::AvmError(security_error(
                    activation,
                    "Error #2031: Socket Error.",
                    2031,
                )?))
            }
        }

        return Ok(Value::Undefined);
    }
    Err("Socket.prototype.writeByte has been called on an incompatible object".into())
}

/// Native function definition for `Socket.flush`
pub fn flush<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_socket() {
        let queue = match this.outgoing_queue() {
            Some(queue) => Ok(queue),
            None => Err(Error::AvmError(security_error(
                activation,
                "Error #2031: Socket Error.",
                2031,
            )?)),
        }?;
        let queue = queue.read();
        match queue.send(OutgoingSocketAction::Flush) {
            Ok(_) => {}
            Err(_) => {
                return Err(Error::AvmError(security_error(
                    activation,
                    "Error #2031: Socket Error.",
                    2031,
                )?))
            }
        }

        return Ok(Value::Undefined);
    }
    Err("Socket.prototype.flush has been called on an incompatible object".into())
}

/// Native function definition for `Socket.close`
pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_socket() {
        let queue = match this.outgoing_queue() {
            Some(queue) => Ok(queue),
            None => Err(Error::AvmError(security_error(
                activation,
                "Error #2031: Socket Error.",
                2031,
            )?)),
        }?;
        let queue = queue.read();
        match queue.send(OutgoingSocketAction::Close) {
            Ok(_) => {}
            Err(_) => {
                return Err(Error::AvmError(security_error(
                    activation,
                    "Error #2031: Socket Error.",
                    2031,
                )?))
            }
        }

        return Ok(Value::Undefined);
    }
    Err("Socket.prototype.close has been called on an incompatible object".into())
}

/// Native function definition for `Socket.readBytes`
pub fn read_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_socket() {
        let bytes = args.get_object(activation, 0, "bytes")?;
        let mut bytes = bytes
            .as_bytearray_mut(activation.context.gc_context)
            .ok_or("ArgumentError: Parameter must be a ByteArray")?;
        let offset = args.get_u32(activation, 1)? as usize;
        let length = args.get_u32(activation, 2)? as usize;

        tracing::debug!(
            "Socket.readBytes: {:?} offset={} len={}",
            bytes,
            offset,
            length
        );

        let queue = match this.recv_queue() {
            Some(queue) => Ok(queue),
            None => Err(Error::AvmError(security_error(
                activation,
                "Error #2031: Socket Error.",
                2031,
            )?)),
        }?;
        let queue = queue.read();
        if queue.is_disconnected() {
            return Err(Error::AvmError(security_error(
                activation,
                "Error #2031: Socket Error.",
                2031,
            )?));
        }

        let buffer = if length == 0 {
            tracing::debug!("Socket.readBytes: reading unbounded");
            let mut buffer = Vec::with_capacity(1024);
            while queue.len() > 0 {
                let chunk = queue.recv().unwrap();
                buffer.extend(chunk);
            }
            tracing::debug!("Socket.readBytes: read unbounded: {} bytes", buffer.len());

            buffer
        } else {
            tracing::debug!("Socket.readBytes: reading bounded");
            let mut buffer = Vec::with_capacity(length);
            while buffer.len() < length {
                let chunk = queue.recv().unwrap();
                buffer.extend(chunk);
            }
            tracing::debug!("Socket.readBytes: read bounded: {} bytes", buffer.len());

            buffer
        };

        let position = bytes.position();
        bytes.write_at(&buffer, position).unwrap();

        tracing::debug!(
            "read {} bytes out of {}, pos: {}\n{:?}",
            buffer.len(),
            length,
            position,
            buffer
        );

        return Ok(Value::Undefined);
    }
    Err("Socket.prototype.readBytes has been called on an incompatible object".into())
}
