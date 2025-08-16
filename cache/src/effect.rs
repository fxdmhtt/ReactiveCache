use std::rc::Rc;

pub struct Effect {
    f: Box<dyn Fn()>,
}

impl Effect {
    pub fn new(f: impl Fn() + 'static) -> Rc<Self> {
        let e = Rc::new(Effect { f: Box::new(f) });

        crate::current_effect_push(e.clone());
        e.run();
        crate::current_effect_pop();

        e
    }

    pub(crate) fn run(&self) {
        (self.f)()
    }
}
