use std::rc::Rc;
use std::ops::Deref;

/// A safe version of C++ std::shared_ptr's aliasing constructor for Rust [`std::Rc<T>`][std::Rc].
/// Derefs to a borrow of the Rc's content by caching a user-generated pointer.
/// This brings convenience and performance
/// - **convenience**: When ownership isn't shared for [Child] directly, it must be borrowed from [Parent]. [AliasRc<Parent, Child>][AliasRc] does this access once at construction.
/// - **performance**: Cache misses due to indirect access to [Child] through [Parent] and the resulting Safety/bound checks are done once at construction, not once per Deref.
///
/// usage: Safety/bound checks///
/// # Examples
///
/// ```
/// # use std::ops::Deref;
/// use std::rc::Rc;
/// use lib_raytracer::utils::AliasRc;
///
/// let strings = Rc::new(vec!["first", "second"]);
/// let first_str = AliasRc::new(strings, |vec| vec[0]);
/// assert_eq!(first_str.deref(), "first");
/// ```
pub struct AliasRc<Parent, Child>
    where
        Parent: ?Sized,
        Child: ?Sized {
    parent: Rc<Parent>,
    child: *const Child,
}

impl<Parent, Child> AliasRc<Parent, Child>
    where
        Parent: ?Sized,
        Child: ?Sized {
    pub fn new(parent: Rc<Parent>,
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

    pub fn into_parent(this: Self) -> Rc<Parent> {
        this.parent
    }
}

impl<Parent, Child> Deref for AliasRc<Parent, Child>
    where
        Parent: ?Sized,
        Child: ?Sized {
    type Target = Child;

    fn deref(&self) -> &Self::Target {
        Self::get_child(self)
    }
}

impl<Parent, Child> Clone for AliasRc<Parent, Child>
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