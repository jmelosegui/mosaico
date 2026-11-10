//! `subscribe`: a session kept open, its messages printed as they come.

use std::io::Write;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::api::ErrorCode;
use crate::ipc::framing::write_message;

use super::link::{
    Event, disconnected, hello, not_connected, read_daemon, read_input, refused, session_error,
    spawn,
};
use super::output::Printer;
use super::queue::{Received, Receiver, Sender, queue};
use super::space::{Context, Space};
use super::{ApiClient, Outcome, Output, Streams, Subscription};

/// Follows the state as `name` until the session, the input, the output
/// or the parent ends.
pub fn subscribe(client: &dyn ApiClient, name: &str, sub: &Subscription, io: Streams) -> Outcome {
    let (sender, events) = queue();
    let process = io.parent.as_ref().and_then(|parent| parent.pid());
    if let Some(parent) = io.parent {
        let sender = sender.clone();
        spawn("parent-watch", move || {
            parent.wait();
            let _ = sender.send(Event::ParentExited);
        });
    }
    if sub.requests {
        let sender = sender.clone();
        let input = io.input;
        spawn("subscribe-input", move || read_input(input, &sender));
    }
    let mut printer = Printer::new(sub.output.clone(), sub.monitor.clone(), sub.requests);
    printer.follow(sub.topics.clone());
    let mut subscriber = Subscriber {
        client,
        name,
        sub,
        out: io.out,
        printer,
        sender,
        space: Space::new(sub.reserve.clone()),
        process,
        topics: sub.topics.clone(),
        generation: 0,
        writer: None,
        in_flight: Vec::new(),
        ready: false,
        held: Vec::new(),
        input_ended: false,
        bye: None,
        last_input: Instant::now(),
        next_try: Instant::now(),
    };
    subscriber.steps(&events).err().unwrap_or(Outcome::Done)
}

struct Subscriber<'a> {
    client: &'a dyn ApiClient,
    name: &'a str,
    sub: &'a Subscription,
    out: &'a mut dyn Write,
    printer: Printer,
    sender: Sender<Event>,
    space: Space,
    /// The process that started this one, the bar's, said in `hello`.
    process: Option<u32>,
    /// The topics the next `hello` asks for: the last `subscribe` forwarded.
    topics: Option<Vec<String>>,
    /// Counts sessions, so the messages of a lost one are dropped.
    generation: u64,
    writer: Option<Box<dyn Write + Send>>,
    /// The requests forwarded and not yet answered.
    in_flight: Vec<u64>,
    /// Whether the session's snapshot came, so the monitors are known.
    ready: bool,
    /// The requests read before it, sent once it comes.
    held: Vec<Value>,
    /// Whether the input ended, so the command ends once all is answered.
    input_ended: bool,
    /// Why the daemon said the session ends, if it did.
    bye: Option<String>,
    last_input: Instant,
    /// When to try to connect next, so a daemon that is not running, or
    /// that ends every session at once, is not asked again and again.
    next_try: Instant,
}

/// How a step of the loop went: on, or ended with an outcome.
type Step = Result<(), Outcome>;

impl Subscriber<'_> {
    fn steps(&mut self, events: &Receiver<Event>) -> Step {
        loop {
            if self.writer.is_none() && Instant::now() >= self.next_try {
                self.connect()?;
            }
            let wait = self.next_wait();
            match events.recv_timeout(wait) {
                Received::Item(event) => self.handle(event)?,
                Received::Timeout => self.check_idle()?,
                Received::Closed => return Ok(()),
            }
        }
    }

    /// How long to wait for the next event.
    fn next_wait(&self) -> Duration {
        let idle = if self.sub.requests {
            self.sub.idle.saturating_sub(self.last_input.elapsed())
        } else {
            Duration::MAX
        };
        match self.writer {
            None => idle.min(self.next_try.saturating_duration_since(Instant::now())),
            Some(_) => idle.min(Duration::from_secs(3600)),
        }
    }

    fn check_idle(&self) -> Step {
        let quiet = self.sub.requests && self.last_input.elapsed() >= self.sub.idle;
        if quiet { Err(Outcome::Done) } else { Ok(()) }
    }

    /// Opens a session, unless no daemon runs and `wait` says to try again.
    fn connect(&mut self) -> Step {
        self.next_try = Instant::now() + self.sub.retry;
        let connection = match self.client.connect() {
            Ok(connection) => connection,
            Err(error) if !self.sub.wait => return Err(not_connected(error)),
            Err(_) => return Ok(()),
        };
        self.generation += 1;
        self.bye = None;
        self.ready = false;
        read_daemon(connection.reader, self.generation, &self.sender);
        let mut writer = connection.writer;
        let bar = self.sub.bar.as_deref();
        let opening = hello(self.name, bar, self.topics.clone(), self.process);
        // A failed write shows as the session ending, read on the other half.
        let _ = write_message(&mut writer, &opening);
        self.writer = Some(writer);
        Ok(())
    }

    fn handle(&mut self, event: Event) -> Step {
        match event {
            Event::Daemon(generation, message) if generation == self.generation => {
                self.on_daemon(&message)
            }
            Event::Ended(generation) if generation == self.generation => self.ended(),
            Event::Daemon(..) | Event::Ended(_) => Ok(()),
            Event::Input(line) => self.on_input(&line),
            Event::InputEnded => {
                self.input_ended = true;
                self.check_answered()
            }
            Event::ParentExited => Err(Outcome::Done),
        }
    }

    fn on_daemon(&mut self, message: &Value) -> Step {
        if let Some(error) = session_error(message)
            && error.code == ErrorCode::UnsupportedApi
        {
            return Err(refused(error));
        }
        if !self.space.answered(message) {
            return Ok(());
        }
        if let Some(id) = message["id"].as_u64() {
            self.in_flight.retain(|waiting| *waiting != id);
        }
        if let Some(line) = self.printer.take(message) {
            self.print(&line)?;
        }
        if let Some(bye) = message.get("bye") {
            if !self.sub.wait {
                return Err(Outcome::Done);
            }
            self.bye = Some(bye["reason"].as_str().unwrap_or("").to_string());
        }
        if !self.ready && message.get("snapshot").is_some() {
            self.ready = true;
            for request in std::mem::take(&mut self.held) {
                self.send(request)?;
            }
        }
        if self.ready {
            let at = Context::new(self.printer.state(), self.sub.monitor.as_ref());
            let due = self.space.due(&at);
            if let Some(writer) = self.writer.as_mut() {
                // A failed write shows as the session ending.
                due.iter()
                    .for_each(|request| drop(write_message(writer, request)));
            }
        }
        self.check_answered()
    }

    /// Ends the command once the input ended and every request it held
    /// was answered.
    fn check_answered(&self) -> Step {
        let answered = self.in_flight.is_empty() && self.held.is_empty();
        if self.input_ended && answered {
            Err(Outcome::Done)
        } else {
            Ok(())
        }
    }

    /// The session ended: ends the command, or, with `wait`, fails what
    /// was in flight and connects again.
    fn ended(&mut self) -> Step {
        if !self.sub.wait {
            return Err(Outcome::Lost);
        }
        self.writer = None;
        self.printer.lost();
        self.space.lost();
        if matches!(self.sub.output, Output::Messages | Output::Full) {
            let reason = self.bye.take().unwrap_or_else(|| "lost".to_string());
            self.print(&json!({"disconnected": {"reason": reason}}).to_string())?;
        }
        let held = std::mem::take(&mut self.held);
        let held = held.iter().filter_map(|request| request["id"].as_u64());
        for id in std::mem::take(&mut self.in_flight).into_iter().chain(held) {
            self.print(&disconnected(id))?;
        }
        self.check_answered()
    }

    fn on_input(&mut self, line: &str) -> Step {
        self.last_input = Instant::now();
        let Ok(request) = serde_json::from_str::<Value>(line) else {
            let error = json!({"error": {"code": "malformed", "message": "not JSON"}});
            return self.print(&error.to_string());
        };
        // Held until the snapshot, which `--monitor` is resolved against.
        if self.writer.is_some() && !self.ready {
            self.held.push(request);
            return Ok(());
        }
        self.send(request)
    }

    fn send(&mut self, mut request: Value) -> Step {
        self.name_monitor(&mut request);
        if let Some(topics) = request.pointer("/subscribe/topics") {
            self.topics = serde_json::from_value(topics.clone()).ok();
            self.printer.follow(self.topics.clone());
        }
        let id = request["id"].as_u64();
        let Some(writer) = self.writer.as_mut() else {
            return match id {
                Some(id) => self.print(&disconnected(id)),
                None => Ok(()),
            };
        };
        if let Some(id) = id
            && !self.in_flight.contains(&id)
        {
            self.in_flight.push(id);
        }
        let at = Context::new(self.printer.state(), self.sub.monitor.as_ref());
        self.space.forwarded(&mut request, &at);
        // A failed write shows as the session ending, which fails the request.
        let _ = write_message(writer, &request);
        Ok(())
    }

    /// Gives a `run` that names no monitor the one `--monitor` names.
    fn name_monitor(&self, request: &mut Value) {
        let Some(monitor) = &self.sub.monitor else {
            return;
        };
        let Some(Value::Object(run)) = request.get_mut("run") else {
            return;
        };
        if run.contains_key("monitor") {
            return;
        }
        if let Some(id) = monitor.resolve(self.printer.state()) {
            run.insert("monitor".to_string(), Value::String(id));
        }
    }

    fn print(&mut self, line: &str) -> Step {
        let written = writeln!(self.out, "{line}").and_then(|()| self.out.flush());
        written.map_err(|_| Outcome::Done)
    }
}
