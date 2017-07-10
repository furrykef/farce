use std::str::SplitWhitespace;
use std::sync::mpsc;
use std::thread;

pub struct EngineThreadManager {
    tx: mpsc::Sender<MessageToEngine>
}

pub enum MessageToEngine {
    IsReady,
    PonderHit,
    Stop
}

impl EngineThreadManager {
    pub fn new() -> EngineThreadManager {
        let (tx, rx) = mpsc::channel::<MessageToEngine>();
        // XXX create the thread
        EngineThreadManager {
            tx: tx
        }
    }

    pub fn cmd_go(&self, tokens: &mut SplitWhitespace) {
        // XXX
    }

    pub fn cmd_isready(&self) {
        self.tx.send(MessageToEngine::IsReady).expect("Error sending message");
    }

    pub fn cmd_ponderhit(&self) {
        self.tx.send(MessageToEngine::PonderHit).expect("Error sending message");
    }

    pub fn cmd_position(&self, tokens: &mut SplitWhitespace) {
        // XXX
    }

    pub fn cmd_setoption(&self, tokens: &mut SplitWhitespace) {
        // XXX
    }

    pub fn cmd_stop(&self) {
        self.tx.send(MessageToEngine::Stop).expect("Error sending message");
    }
}
