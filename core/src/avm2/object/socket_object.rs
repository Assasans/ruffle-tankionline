//! Object representation for Socket objects

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use core::fmt;
use gc_arena::{Collect, GcCell, GcWeakCell, MutationContext};
use std::cell::{Ref, RefMut};
use tokio::net::TcpStream;

/// A class instance allocator that allocates Socket objects.
pub fn socket_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(SocketObject(GcCell::new(
        activation.context.gc_context,
        SocketObjectData {
            base,
            recv_queue: None,
            send_queue: None,
            flush_queue: None,
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct SocketObject<'gc>(pub GcCell<'gc, SocketObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct SocketObjectWeak<'gc>(pub GcWeakCell<'gc, SocketObjectData<'gc>>);

impl fmt::Debug for SocketObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SocketObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

impl<'gc> SocketObject<'gc> {
    pub fn recv_queue(self) -> Option<GcCell<'gc, GcRecvQueue>> {
        self.0.read().recv_queue
    }

    pub fn set_recv_queue(self, recv_queue: Option<GcCell<'gc, GcRecvQueue>>, mc: MutationContext<'gc, '_>) {
        self.0.write(mc).recv_queue = recv_queue;
    }

    pub fn send_queue(self) -> Option<GcCell<'gc, GcSendQueue>> {
        self.0.read().send_queue
    }

    pub fn set_send_queue(self, send_queue: Option<GcCell<'gc, GcSendQueue>>, mc: MutationContext<'gc, '_>) {
        self.0.write(mc).send_queue = send_queue;
    }

    pub fn flush_queue(self) -> Option<GcCell<'gc, GcFlushQueue>> {
        self.0.read().flush_queue
    }

    pub fn set_flush_queue(self, flush_queue: Option<GcCell<'gc, GcFlushQueue>>, mc: MutationContext<'gc, '_>) {
        self.0.write(mc).flush_queue = flush_queue;
    }
}

#[derive(Collect)]
#[collect(require_static)]
pub struct GcRecvQueue(pub &'static flume::Receiver<Vec<u8>>);

#[derive(Collect)]
#[collect(require_static)]
pub struct GcSendQueue(pub &'static flume::Sender<Vec<u8>>);

#[derive(Collect)]
#[collect(require_static)]
pub struct GcFlushQueue(pub &'static flume::Sender<()>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct SocketObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    recv_queue: Option<GcCell<'gc, GcRecvQueue>>,
    send_queue: Option<GcCell<'gc, GcSendQueue>>,
    flush_queue: Option<GcCell<'gc, GcFlushQueue>>,
}

impl<'gc> TObject<'gc> for SocketObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_socket(&self) -> Option<SocketObject<'gc>> {
        Some(*self)
    }
}
