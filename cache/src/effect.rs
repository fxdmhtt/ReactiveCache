use std::rc::Rc;

pub struct Effect {
    f: Box<dyn Fn()>,
}

impl Effect {
    pub fn new(f: impl Fn() + 'static) -> Rc<Self> {
        let e = Rc::new(Effect { f: Box::new(f) });

        crate::creating_effect_push(Rc::downgrade(&e));
        e.run();
        crate::creating_effect_pop();

        e
    }

    pub(crate) fn run(&self) {
        (self.f)()
    }
}
