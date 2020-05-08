use std::sync::mpsc::{self, Sender, Receiver};

/// Two way channel
/// S is being sent
/// A is being returned
pub struct Duplex<S, A> {
    c1: Sender<S>,
    c2: Receiver<A>
}

impl<S, A> Duplex<S, A> {
    fn new(c1: Sender<S>, c2: Receiver<A>) -> Self {
        Self { c1, c2 }
    }

    pub fn send(&self, data: S) -> Result<(), mpsc::SendError<S>> {
        self.c1.send(data)
    }

    #[allow(dead_code)]
    pub fn try_recv(&self) -> Result<A, mpsc::TryRecvError> {
        self.c2.try_recv()
    }

    pub fn recv(&self) -> Result<A, mpsc::RecvError> {
        self.c2.recv()
    }
}

pub fn duplex<S, A>() -> (Duplex<S, A>, Duplex<A, S>) {
    let (s1, s2) = mpsc::channel::<S>();
    let (a1, a2) = mpsc::channel::<A>();
    (Duplex::<S, A>::new(s1, a2), Duplex::<A, S>::new(a1, s2))
}

#[allow(dead_code)]
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    mpsc::channel()
}


#[test]
fn duplex_same() {
    let (d1, d2) = duplex::<String, String>();
    std::thread::spawn(move || {
        d2.send("Hello From Thread".to_string()).unwrap();
        let val = d2.recv().unwrap();
        assert!(val == "Hello Again");
    });

    let val = d1.recv().unwrap();
    d1.send("Hello Again".to_string()).unwrap();
    assert!(val == "Hello From Thread");
}

#[test]
fn duplex_diff() {
    let (d1, d2) = duplex::<String, i32>();
    std::thread::spawn(move || {
        d2.send(42).unwrap();
        let val = d2.recv().unwrap();
        assert!(val == "Hello Again");
    });

    let val = d1.recv().unwrap();
    d1.send("Hello Again".to_string()).unwrap();
    assert!(val == 42);
}