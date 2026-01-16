use std::rc::Rc;

#[derive(Clone)]
pub struct TaskHandle {
    inner: Rc<dyn Task>,
}

impl TaskHandle {
    // TODO: future errors
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        let inner = Rc::new(TaskInner { future });
        Self { inner }
    }
}

struct TaskInner<F> {
    future: F,
}

trait Task {}

impl<F: Future<Output = ()>> Task for TaskInner<F> {}
