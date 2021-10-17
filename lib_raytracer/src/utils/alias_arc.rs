use std::sync::Arc;
use std::ops::Deref;

pub struct AliasArc<Parent, Child>
    where
        Parent: ?Sized,
        Child: ?Sized {
    parent: Arc<Parent>,
    child: *const Child,
}

impl<Parent, Child> AliasArc<Parent, Child>
    where
        Parent: ?Sized,
        Child: ?Sized {
    pub fn new(parent: Arc<Parent>,
               child_from_parent: impl Fn(&Parent)->&Child) -> Self {
        let child = child_from_parent(parent.deref()) as _;

        Self {
            parent,
            child
        }
    }

    pub fn get_child(this: &Self) -> &Child {
        // TODO: Safety
        unsafe {
            &*this.child
        }
    }

    pub fn get_parent(this: &Self) -> &Parent {
        this.parent.deref()
    }

    pub fn into_parent(this: Self) -> Arc<Parent> {
        this.parent
    }
}

unsafe impl<Parent, Child> Send for AliasArc<Parent, Child>
    where
        Parent: ?Sized + Sync + Send,
        Child: ?Sized + Sync + Send {}

unsafe impl<Parent, Child> Sync for AliasArc<Parent, Child>
    where
        Parent: ?Sized + Sync + Send,
        Child: ?Sized + Sync + Send {}

impl<Parent, Child> Deref for AliasArc<Parent, Child>
    where
        Parent: ?Sized,
        Child: ?Sized {
    type Target = Child;

    fn deref(&self) -> &Self::Target {
        Self::get_child(self)
    }
}

impl<Parent, Child> Clone for AliasArc<Parent, Child>
    where
        Parent: ?Sized,
        Child: ?Sized {
    fn clone(&self) -> Self {
        Self {
            parent: self.parent.clone(),
            child: self.child
        }
    }
}

