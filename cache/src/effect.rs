use std::rc::Rc;

pub struct Effect<F>
where
    F: Fn(),
{
    f: F,
}

impl<F> Effect<F>
where
    F: Fn(),
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new(f: F) -> Rc<dyn IEffect>
    where
        F: 'static,
    {
        let e: Rc<dyn IEffect> = Rc::new(Effect { f });

        crate::creating_effect_push(Rc::downgrade(&e));
        e.run();
        crate::creating_effect_pop();

        e
    }
}

pub trait IEffect {
    fn run(&self);
}

impl<F> IEffect for Effect<F>
where
    F: Fn(),
{
    fn run(&self) {
        (self.f)()
    }
}
