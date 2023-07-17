//! Object representation for Socket objects

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use core::fmt;
use flume::{Receiver, Sender};
use gc_arena::{Collect, GcCell, GcWeakCell, MutationContext};
use std::cell::{Ref, RefMut};
use std::ops::Deref;

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
            outgoing_queue: None,
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

    pub fn set_recv_queue(
        self,
        recv_queue: Option<GcCell<'gc, GcRecvQueue>>,
        mc: MutationContext<'gc, '_>,
    ) {
        self.0.write(mc).recv_queue = recv_queue;
    }

    pub fn outgoing_queue(self) -> Option<GcCell<'gc, GcOutgoingQueue>> {
        self.0.read().outgoing_queue
    }

    pub fn set_outgoing_queue(
        self,
        outgoing_queue: Option<GcCell<'gc, GcOutgoingQueue>>,
        mc: MutationContext<'gc, '_>,
    ) {
        self.0.write(mc).outgoing_queue = outgoing_queue;
    }
}

#[derive(Collect)]
#[collect(require_static)]
pub struct GcRecvQueue(pub Receiver<Vec<u8>>);

impl Deref for GcRecvQueue {
    type Target = Receiver<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Collect)]
#[collect(require_static)]
pub struct GcOutgoingQueue(pub Sender<OutgoingSocketAction>);

impl Deref for GcOutgoingQueue {
    type Target = Sender<OutgoingSocketAction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub enum OutgoingSocketAction {
    Write(Vec<u8>),
    Flush,
    Close,
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct SocketObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    recv_queue: Option<GcCell<'gc, GcRecvQueue>>,
    outgoing_queue: Option<GcCell<'gc, GcOutgoingQueue>>,
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
