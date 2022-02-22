use {
    libc::c_void,
    objc::{
        declare::ClassDecl,
        runtime::{Class, Object, Sel},
        sel, sel_impl, Message,
    },
    objc_foundation::{INSObject, NSObject},
    objc_id::Id,
    std::mem,
};

pub(crate) enum Callback {}
unsafe impl Message for Callback {}

// SO.. some explanation is in order here.  We want to allow closure callbacks that
// can modify their environment.  But we can't keep them on the $name object because
// that is really just a stateless proxy for the objc object.  So we store them
// as numeric pointers values in "ivar" fields on that object.  But, if we store a pointer to the
// closure object, we'll run into issues with thin/fat pointer conversions (because
// closure objects are trait objects and thus fat pointers).  So we wrap the closure in
// another boxed object ($cbs_name), which, since it doesn't use traits, is actually a
// regular "thin" pointer, and store THAT pointer in the ivar.  But...so...oy.
pub(crate) struct CallbackState {
    cb: Box<dyn Fn() -> ()>,
}

impl Callback {
    pub(crate) fn from(cb: Box<dyn Fn() -> ()>) -> Id<Self> {
        let cbs = CallbackState { cb };
        let bcbs = Box::new(cbs);

        let ptr = Box::into_raw(bcbs);
        let ptr = ptr as *mut c_void as usize;
        let mut oid = <Callback as INSObject>::new();
        (*oid).setptr(ptr);
        oid
    }

    pub(crate) fn setptr(&mut self, uptr: usize) {
        unsafe {
            let obj = &mut *(self as *mut _ as *mut ::objc::runtime::Object);
            obj.set_ivar("_cbptr", uptr);
        }
    }
}

// TODO: Drop for $name doesn't get called, probably because objc manages the memory and
// releases it for us.  so we leak the boxed callback right now.

impl INSObject for Callback {
    fn class() -> &'static Class {
        let cname = "Callback";

        let mut klass = Class::get(cname);
        if klass.is_none() {
            let superclass = NSObject::class();
            let mut decl = ClassDecl::new(&cname, superclass).unwrap();
            decl.add_ivar::<usize>("_cbptr");

            extern "C" fn sysbar_callback_call(this: &Object, _cmd: Sel) {
                unsafe {
                    let pval: usize = *this.get_ivar("_cbptr");
                    let ptr = pval as *mut c_void;
                    let ptr = ptr as *mut CallbackState;
                    let bcbs: Box<CallbackState> = Box::from_raw(ptr);
                    {
                        (*bcbs.cb)();
                    }
                    mem::forget(bcbs);
                }
            }

            unsafe {
                decl.add_method(
                    sel!(call),
                    sysbar_callback_call as extern "C" fn(&Object, Sel),
                );
            }

            decl.register();
            klass = Class::get(cname);
        }
        klass.unwrap()
    }
}
