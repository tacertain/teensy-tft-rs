/// A simple blocking executor for async hardware examples.
pub mod blocking {
    use core::{future::Future, pin::Pin, task::Poll};

    /// Poll a future with a dummy waker.
    ///
    /// Use `poll_no_wake` when you want to drive a future to completion, but you
    /// don't care about the future waking an executor. It may be used to initiate
    /// a DMA transfer that will later be awaited with [`block`].
    ///
    /// Do not use `poll_no_wake` if you want an executor to be woken when the DMA
    /// transfer completes.
    fn poll_no_wake<F>(future: Pin<&mut F>) -> Poll<F::Output>
    where
        F: Future,
    {
        use core::task::{Context, RawWaker, RawWakerVTable, Waker};
        const VTABLE: RawWakerVTable = RawWakerVTable::new(|_| RAW_WAKER, |_| {}, |_| {}, |_| {});

        const RAW_WAKER: RawWaker = RawWaker::new(core::ptr::null(), &VTABLE);
        // Safety: raw waker meets documented requirements.
        let waker = unsafe { Waker::from_raw(RAW_WAKER) };
        let mut context = Context::from_waker(&waker);
        future.poll(&mut context)
    }

    /// Block until the future returns a result.
    ///
    /// `block` invokes `poll_no_wake()` in a loop until the future
    /// returns a result. Consider using `block` after starting a transfer
    /// with `poll_no_wake`, and after doing other work.
    pub fn run<F>(mut future: Pin<&mut F>) -> F::Output
    where
        F: Future,
    {
        loop {
            match poll_no_wake(future.as_mut()) {
                Poll::Ready(result) => return result,
                Poll::Pending => {}
            }
        }
    }
}