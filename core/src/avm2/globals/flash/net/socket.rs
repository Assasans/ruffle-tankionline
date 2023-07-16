//! `flash.net.Socket` native function definitions

use crate::avm2::activation::Activation;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::object::{GcOutgoingQueue, GcRecvQueue, OutgoingSocketAction, TObject};
use crate::avm2::value::Value;
use crate::avm2::{Error, Object};
use gc_arena::GcCell;

pub use crate::avm2::object::socket_allocator;

/// Native function definition for `Socket.connect`
pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    tracing::debug!("Socket.connect: start");

    if let Some(this) = this.as_socket() {
        let host = match args.get(0) {
            Some(Value::String(host)) => host,
            // This should never actually happen
            _ => panic!("host fucked up"),
        };
        let port = match args.get(1) {
            Some(Value::Integer(port)) => *port as u16,
            // This should never actually happen
            _ => panic!("port fucked up"),
        };

        let addr = (host.to_string(), port);

        let future = activation.context.load_manager.load_socket(
            activation.context.player.clone(),
            this,
            addr,
        );
        activation.context.navigator.spawn_future(future);

        return Ok(Value::Undefined);
    }
    Err("Socket.prototype.connect has been called on an incompatible object".into())
}

/// Native function definition for `Socket.writeBytes`
pub fn write_bytes<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    tracing::debug!("Socket.writeBytes: start");

    if let Some(this) = this.as_socket() {
        let bytes = match args.get(0) {
            Some(Value::Object(bytes)) => bytes.as_bytearray().unwrap(),
            // This should never actually happen
            _ => panic!("bytes fucked up"),
        };
        let offset = match args.get(1) {
            Some(Value::Integer(offset)) => *offset as usize,
            // This should never actually happen
            _ => panic!("offset fucked up"),
        };
        let length = match args.get(2) {
            Some(Value::Integer(length)) => *length as usize,
            // This should never actually happen
            _ => panic!("length fucked up"),
        };
        tracing::debug!(
            "Socket.writeBytes: {:?} offset={} len={}",
            bytes,
            offset,
            length
        );

        let mut queue: GcCell<'gc, GcOutgoingQueue> = this.outgoing_queue().unwrap();
        let queue = queue.read().0;
        queue
            .send(OutgoingSocketAction::Send(
                ByteArrayStorage::bytes(&bytes).to_vec(),
            ))
            .unwrap();

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
    tracing::debug!("Socket.writeByte: start");

    if let Some(this) = this.as_socket() {
        let byte = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)
            .unwrap();
        tracing::debug!("Socket.writeByte: {:?}", byte);

        let mut queue: GcCell<'gc, GcOutgoingQueue> = this.outgoing_queue().unwrap();
        let queue = queue.read().0;
        queue
            .send(OutgoingSocketAction::Send(vec![byte as u8]))
            .unwrap();

        return Ok(Value::Undefined);
    }
    Err("Socket.prototype.writeByte has been called on an incompatible object".into())
}

/// Native function definition for `Socket.flush`
pub fn flush<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    tracing::debug!("Socket.flush: start");

    if let Some(this) = this.as_socket() {
        let mut queue: GcCell<'gc, GcOutgoingQueue> = this.outgoing_queue().unwrap();
        let queue = queue.read().0;
        if !queue.is_disconnected() {
            queue.send(OutgoingSocketAction::Flush).unwrap();
        }

        return Ok(Value::Undefined);
    }
    Err("Socket.prototype.flush has been called on an incompatible object".into())
}

/// Native function definition for `Socket.close`
pub fn close<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    tracing::debug!("Socket.flush: start");

    if let Some(this) = this.as_socket() {
        let mut queue: GcCell<'gc, GcOutgoingQueue> = this.outgoing_queue().unwrap();
        let queue = queue.read().0;
        if !queue.is_disconnected() {
            queue.send(OutgoingSocketAction::Close).unwrap();
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
    tracing::debug!("Socket.readBytes: start");

    if let Some(this) = this.as_socket() {
        let mut bytes = match args.get(0) {
            Some(Value::Object(bytes)) => bytes
                .as_bytearray_mut(activation.context.gc_context)
                .unwrap(),
            // This should never actually happen
            _ => panic!("bytes fucked up"),
        };
        let offset = match args.get(1) {
            Some(Value::Integer(offset)) => *offset as usize,
            // This should never actually happen
            _ => panic!("offset fucked up"),
        };
        let length = match args.get(2) {
            Some(Value::Integer(length)) => *length as usize,
            // This should never actually happen
            _ => panic!("length fucked up"),
        };
        tracing::debug!(
            "Socket.readBytes: {:?} offset={} len={}",
            bytes,
            offset,
            length
        );

        // let length = if length == 0 { 1024 } else { length };

        let mut socket: GcCell<'gc, GcRecvQueue> = this.recv_queue().unwrap();
        let socket = socket.read();
        let socket = socket.0;

        let buffer = if length == 0 {
            tracing::debug!("Socket.readBytes: reading unbounded");
            let mut buffer = Vec::with_capacity(1024);
            while socket.len() > 0 {
                let chunk = socket.recv().unwrap();
                buffer.extend(chunk);
            }
            tracing::debug!("Socket.readBytes: read unbounded: {} bytes", buffer.len());

            buffer
        } else {
            tracing::debug!("Socket.readBytes: reading bounded");
            let mut buffer = Vec::with_capacity(length);
            while buffer.len() < length {
                let chunk = socket.recv().unwrap();
                buffer.extend(chunk);
            }
            tracing::debug!("Socket.readBytes: read bounded: {} bytes", buffer.len());

            buffer
        };

        // self.position.set(self.position.get() + buf.len());
        tracing::error!("byte output position: {}", bytes.position());
        let position = bytes.position();
        bytes.write_at(&buffer, position).unwrap();
        // bytes.write_bytes(&buffer).unwrap();
        tracing::error!(
            "read {} bytes out of {}, pos: {}",
            buffer.len(),
            length,
            position
        );
        tracing::error!("{:?}", buffer);

        return Ok(Value::Undefined);
    }
    Err("Socket.prototype.readBytes has been called on an incompatible object".into())
}
