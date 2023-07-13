//! `flash.net.Socket` native function definitions

use std::io::{Read, Write};
use tokio::net::TcpStream;
use gc_arena::GcCell;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::avm2::activation::Activation;
use crate::avm2::object::{GcSendQueue, GcTcpStream, TObject};
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error, EventObject, Multiname, Object};
use crate::avm2::bytearray::ByteArrayStorage;
use crate::loader::{DataFormat, Loader};

pub use crate::avm2::object::socket_allocator;

/// Native function definition for `Socket.connect`
pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    tracing::debug!("Socket.connect: start");

    let old_this = this;
    if let Some(this) = this.as_socket() {
        let host = match args.get(0) {
            Some(Value::String(host)) => host,
            // This should never actually happen
            _ => panic!("host fucked up")
        };
        let port = match args.get(1) {
            Some(Value::Integer(port)) => *port as u16,
            // This should never actually happen
            _ => panic!("port fucked up")
        };

        let addr = (host.to_string(), port);

        let future = activation.context
            .load_manager
            .load_socket(activation.context.player.clone(), this, addr);
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
    tracing::debug!("Socket.writeBytes: start");

    let old_this = this;
    if let Some(this) = this.as_socket() {
        let bytes = match args.get(0) {
            Some(Value::Object(bytes)) => bytes.as_bytearray().unwrap(),
            // This should never actually happen
            _ => panic!("bytes fucked up")
        };
        let offset = match args.get(1) {
            Some(Value::Integer(offset)) => *offset as usize,
            // This should never actually happen
            _ => panic!("offset fucked up")
        };
        let length = match args.get(2) {
            Some(Value::Integer(length)) => *length as usize,
            // This should never actually happen
            _ => panic!("length fucked up")
        };
        tracing::debug!("Socket.writeBytes: {:?} offset={} len={}", bytes, offset, length);

        let mut socket: GcCell<'gc, GcSendQueue> = this.send_queue().unwrap();
        let socket = socket.read();
        let socket: &tokio::sync::mpsc::Sender<Vec<u8>> = socket.0;
        socket.blocking_send(ByteArrayStorage::bytes(&bytes).to_vec()).unwrap();
        // socket.write_all(ByteArrayStorage::bytes(&bytes)).unwrap();

        return Ok(Value::Undefined);
    }
    Err("Socket.prototype.writeBytes has been called on an incompatible object".into())
}

/// Native function definition for `Socket.flush`
pub fn flush<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    tracing::debug!("Socket.flush: start");

    let old_this = this;
    if let Some(this) = this.as_socket() {
        let mut socket: GcCell<'gc, GcTcpStream> = this.socket().unwrap();
        let mut socket = socket.write(activation.context.gc_context);
        let socket: &mut TcpStream = socket.0;
        socket.flush(); // TODO: Wtf

        return Ok(Value::Undefined);
    }
    Err("Socket.prototype.flush has been called on an incompatible object".into())
}

/// Native function definition for `Socket.readBytes`
pub fn read_bytes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    tracing::debug!("Socket.readBytes: start");

    let old_this = this;
    if let Some(this) = this.as_socket() {
        let mut bytes = match args.get(0) {
            Some(Value::Object(bytes)) => bytes.as_bytearray_mut(activation.context.gc_context).unwrap(),
            // This should never actually happen
            _ => panic!("bytes fucked up")
        };
        let offset = match args.get(1) {
            Some(Value::Integer(offset)) => *offset as usize,
            // This should never actually happen
            _ => panic!("offset fucked up")
        };
        let length = match args.get(2) {
            Some(Value::Integer(length)) => *length as usize,
            // This should never actually happen
            _ => panic!("length fucked up")
        };
        tracing::debug!("Socket.readBytes: {:?} offset={} len={}", bytes, offset, length);

        let mut socket: GcCell<'gc, GcTcpStream> = this.socket().unwrap();
        let mut socket = socket.write(activation.context.gc_context);
        let socket: &mut TcpStream = socket.0;

        let length = if length == 0 { 1024 } else { length };

        let mut buffer = Vec::new();
        buffer.resize(length, 0);
        let read = socket.try_read(&mut buffer).unwrap();

        bytes.write_bytes(&buffer[..read]).unwrap();
        tracing::error!("read {} bytes out of {}", read, length);

        return Ok(Value::Undefined);
    }
    Err("Socket.prototype.readBytes has been called on an incompatible object".into())
}
