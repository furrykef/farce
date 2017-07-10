#[macro_use] extern crate lazy_static;
extern crate regex;

mod color;
mod engine_thread_manager;
mod piece;
mod position;

fn main() {
    let engine_mgr = engine_thread_manager::EngineThreadManager::new();
    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).expect("Error reading line");
        let line = line;
        let mut tokens = line.split_whitespace();

        // NOTE: If the command is not recognized, the whole line is ignored.
        // This violates the UCI spec (which wants us to continue parsing the line), but it is what
        // Stockfish does, and is IMO the only sane thing to do.
        match tokens.next() {
            None                => (),          // Line was empty, I guess
            Some("debug")       => (),
            Some("go")          => engine_mgr.cmd_go(&mut tokens),
            Some("isready")     => engine_mgr.cmd_isready(),
            Some("ponderhit")   => engine_mgr.cmd_ponderhit(),
            Some("position")    => engine_mgr.cmd_position(&mut tokens),
            Some("quit")        => return,
            Some("register")    => (),
            Some("setoption")   => engine_mgr.cmd_setoption(&mut tokens),
            Some("stop")        => engine_mgr.cmd_stop(),
            Some("uci")         => identify(),
            Some("ucinewgame")  => (),
            Some(_)             => ()           // TODO: unrecognized command; log it
        }
    }
}

fn identify() {
    println!("id name farce");
    println!("id author Kef Schecter");
    println!("uciok");
}
