use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;

use crate::mpmc::{ChannelError, ChannelReceiver, ChannelSender};

use super::ComputeNode;

/// At this time, this is intended to serve as a possible template for a macro to create a series
/// of generic compute nodes that need only take a function and the appropriate channel connections
/// and then can handle the rest of the boilerplate.
#[derive(Clone)]
pub struct GenericComputeNode_1_1<I1, O1, R1, S1, F> {
    /// f will always be called with at least one `Some` value and will only start passing `None`
    /// values once that input is exhausted.
    f: F,
    rx1: R1,
    tx1: S1,
    name: String,
    _phantom_i: PhantomData<I1>,
    _phantom_o: PhantomData<O1>,
}

impl<I1, O1, S1, R1, F> Debug for GenericComputeNode_1_1<I1, O1, S1, R1, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl<I1, O1, S1, R1, F> ComputeNode for GenericComputeNode_1_1<I1, O1, R1, S1, F>
where
    I1: Clone + Send,
    O1: Clone + Send,
    S1: ChannelSender<Item = O1> + Send,
    R1: ChannelReceiver<Item = I1> + Send,
    F: Fn(Option<I1>) -> (Option<O1>) + Send,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self) {
        loop {
            let i1 = match self.rx1.recv() {
                Ok(i1) => Some(i1),
                Err(ChannelError::IsCorked) => None,
                Err(ChannelError::Poisoned) => panic!("Thread was poisoned"),
            };
            if i1.is_none() {
                // all inputs have been exhausted
                break;
            }
            let (o1) = (self.f)(i1);
            if let Some(o1) = o1 {
                self.tx1.send(o1);
            }
        }
    }
}

impl<I1, O1, S1, R1, F> GenericComputeNode_1_1<I1, O1, R1, S1, F>
where
    I1: Clone,
    O1: Clone,
    S1: ChannelSender<Item = O1>,
    R1: ChannelReceiver<Item = I1>,
    F: Fn(Option<I1>) -> (Option<O1>),
{
    pub fn new(name: String, rx: (R1), tx: (S1), f: F) -> Self {
        let (rx1) = rx;
        let (tx1) = tx;
        Self {
            f,
            tx1,
            rx1,
            name,
            _phantom_i: PhantomData::default(),
            _phantom_o: PhantomData::default(),
        }
    }
}
