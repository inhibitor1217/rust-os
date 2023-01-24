use core::task::Poll;

use crossbeam_queue::ArrayQueue;
use futures_util::{task::AtomicWaker, Stream, StreamExt};
use lazy_static::lazy_static;
use pc_keyboard::{layouts::Us104Key, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

use crate::{print, println};

lazy_static! {
    pub static ref SCANCODE_QUEUE: ArrayQueue<u8> = ArrayQueue::new(100);
}

static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    if SCANCODE_QUEUE.push(scancode).is_err() {
        println!("WARNING: scancode queue full; dropping keyboard input");
    } else {
        WAKER.wake();
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    #[must_use]
    pub fn new() -> Self {
        ScancodeStream { _private: () }
    }
}

impl Default for ScancodeStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        if let Some(scancode) = SCANCODE_QUEUE.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(cx.waker());
        if let Some(scancode) = SCANCODE_QUEUE.pop() {
            WAKER.take();
            Poll::Ready(Some(scancode))
        } else {
            Poll::Pending
        }
    }
}

pub async fn print_keypress() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard: Keyboard<Us104Key, ScancodeSet1> = Keyboard::new(HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(char) => print!("{char}"),
                    DecodedKey::RawKey(key) => print!("{key:?}"),
                }
            }
        }
    }
}
