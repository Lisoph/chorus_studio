use std::cell::RefCell;

pub struct Event<'a> {
    handlers: RefCell<Vec<Box<FnMut() + 'a>>>,
}

impl<'a> Event<'a> {
    pub fn new() -> Self {
        Self {
            handlers: RefCell::new(Vec::new()),
        }
    }

    pub fn invoke(&self) {
        for h in self.handlers.borrow_mut().iter_mut() {
            h();
        }
    }

    pub fn add_handler<H: FnMut() + 'a>(&mut self, handler: H) {
        self.handlers.borrow_mut().push(Box::new(handler));
    }
}
