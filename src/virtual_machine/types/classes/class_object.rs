use crate::virtual_machine::{
    traits::member_accessible::IMemberAccessible, types::classes::class::TClass, value::Value,
    vm::VM,
};
use bincode::{Decode, Encode};
use std::{any::Any, cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

#[derive(Clone)]
pub struct TClassObject {
    pub base: Rc<RefCell<TClass>>,
    pub values: Rc<RefCell<HashMap<u64, (Value, bool)>>>,
    pub native_data: Option<Box<dyn NativeData>>,
}

impl Encode for TClassObject {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.base.encode(encoder)?;
        self.values.encode(encoder)?;
        Ok(())
    }
}

impl<Context> Decode<Context> for TClassObject {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            base: Decode::decode(decoder)?,
            values: Decode::decode(decoder)?,
            native_data: None,
        })
    }
}

impl<'de, Context> bincode::BorrowDecode<'de, Context> for TClassObject {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            base: bincode::BorrowDecode::borrow_decode(decoder)?,
            values: bincode::BorrowDecode::borrow_decode(decoder)?,
            native_data: None,
        })
    }
}

pub trait NativeData: Any {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn NativeData>;
}
impl<T> NativeData for T
where
    T: Any + Clone + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

	fn clone_box(&self) -> Box<dyn NativeData> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn NativeData> {
	fn clone(&self) -> Self {
		(**self).clone_box()
	}
}

impl PartialEq for TClassObject {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.base, &other.base) && *self.values.borrow() == *other.values.borrow()
    }
}

impl PartialOrd for TClassObject {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl TClassObject {
    pub fn new(base: Rc<RefCell<TClass>>) -> Self {
        let values = rc!(RefCell::new(base.borrow().values.borrow().clone()));
        Self {
            base,
            values,
            native_data: None,
        }
    }

    pub fn with_native<T: NativeData>(base: Rc<RefCell<TClass>>, native: T) -> Self {
        let values = rc!(RefCell::new(base.borrow().values.borrow().clone()));
        Self {
            base,
            values,
            native_data: Some(Box::new(native)),
        }
    }
}

impl Debug for TClassObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Object:{}", self.base.borrow().name))
            .unwrap();
        Ok(())
    }
}

// MEMBER ACCESS
impl IMemberAccessible for TClassObject {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Value::String(member) = member {
            if let Some((v, _is_const)) = self.values.borrow().get(&hash_u64!(&member.0)) {
                return v.clone();
            }

            if let Some(v) = self
                .base
                .borrow()
                .functions
                .borrow_mut()
                .get_mut(&hash_u64!(&member.0))
            {
                if let Value::Function(f) = v {
                    f.target = Some(boxed!(Value::ClassObject(self.clone())))
                }

                return v.clone();
            }
        }

        panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
    }

    fn get_member_id(&self, vm: &mut VM, member: &u64) -> Value {
        if let Some((v, _is_const)) = self.values.borrow().get(member) {
            return v.clone();
        }

        if let Some(v) = self.base.borrow().functions.borrow_mut().get_mut(member) {
            if let Value::Function(f) = v {
                f.target = Some(boxed!(Value::ClassObject(self.clone())))
            }

            return v.clone();
        }

        panic!(
            "Cannot get member id `{}` on {self:?}",
            vm.lookup_intern(*member)
        );
    }

    fn set_member(&mut self, member: &Value, value: Value) {
        if let Value::String(member) = member {
            if let Some((v, is_const)) = self.values.borrow_mut().get_mut(&hash_u64!(&member.0)) {
                if *is_const {
                    panic!("Cannot set constant member `{}` on {self:?}", member.0);
                }
                *v = value;
                return;
            } else {
                self.values
                    .borrow_mut()
                    .insert(hash_u64!(&member.0), (value, false));
                return;
            }
        }

        panic!("Cannot set member `{}` on {self:?}", member.to_string(true));
    }

    fn set_member_id(&mut self, vm: &mut VM, member: &u64, value: Value) {
        if let Some((v, is_const)) = self.values.borrow_mut().get_mut(member) {
            if *is_const {
                panic!(
                    "Cannot set constant member `{}` on {self:?}",
                    vm.lookup_intern(*member)
                );
            }
            *v = value;
            return;
        } else {
            self.values.borrow_mut().insert(*member, (value, false));
            return;
        }
    }
}
