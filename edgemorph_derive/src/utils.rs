use std::boxed::Box;
use std::rc::Weak;
use std::cell::RefCell;

pub trait Containerize<T> {
    
    fn to_box_vec(self) -> Box<Vec<T>> {
        Box::new(Vec::new(self))
    }

    fn to_weak_refcell(self) -> RefCell<Weak<T>> {
        RefCell::new(Weak::new(self))
    }

    fn to_box(self) -> Box<T> {
        Box::new(self)
    }

}


