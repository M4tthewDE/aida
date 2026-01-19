use core::f32;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::mpsc::{Receiver, Sender, TryRecvError},
};

use chrono::{DateTime, Utc};
use eframe::egui::{self, Color32, RichText};
use ipc_channel::ipc::IpcOneShotServer;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Confirm exit",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
    .unwrap();
}

struct App {
    rx: Receiver<shared::AgentMessage>,
    tx: Sender<shared::AgentMessage>,
    command: String,
    stdout: Vec<String>,
    stderr: Vec<String>,
    class_load_events: Vec<shared::ClassLoadEvent>,
}

impl App {
    fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            rx,
            tx,
            command: "java -jar agent/jars/hello_world.jar".to_owned(),
            stdout: Vec::new(),
            stderr: Vec::new(),
            class_load_events: Vec::new(),
        }
    }

    fn run_command(&mut self) {
        self.stdout = Vec::new();
        self.class_load_events = Vec::new();

        let (server, server_name): (IpcOneShotServer<shared::AgentMessage>, String) =
            IpcOneShotServer::new().unwrap();

        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let (rx, msg) = server.accept().unwrap();

            if matches!(msg, shared::AgentMessage::Unload) {
                return;
            }

            loop {
                let msg = rx.recv().unwrap();
                match msg {
                    shared::AgentMessage::Unload => break,
                    msg => tx.send(msg).unwrap(),
                }
            }
        });

        let mut parts = self.command.split_whitespace();
        let program = parts.next().expect("no command");

        let agent_path = format!("-agentpath:target/debug/libaida.so={}", server_name);

        let mut args = vec![agent_path.as_str()];
        args.extend(parts);

        let mut cmd = Command::new(program)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to execute");

        let stdout = cmd.stdout.take().expect("failed to capture stdout");
        let stderr = cmd.stderr.take().expect("failed to capture stdout");
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        for line in stdout_reader.lines() {
            self.stdout.push(line.unwrap());
        }

        for line in stderr_reader.lines() {
            self.stderr.push(line.unwrap());
        }

        cmd.wait().expect("failed to wait on command");
    }

    fn receive_agent_msg(&mut self, ctx: &egui::Context) {
        match self.rx.try_recv() {
            Ok(msg) => {
                match msg {
                    shared::AgentMessage::ClassLoad(event) => self.class_load_events.push(event),
                    _ => {}
                };

                ctx.request_repaint();
            }
            Err(err) => {
                if matches!(err, TryRecvError::Disconnected) {
                    panic!("{}", err);
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.receive_agent_msg(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Aida");
            ui.add(egui::TextEdit::singleline(&mut self.command).desired_width(f32::INFINITY));

            if ui.button("Run").clicked() {
                self.run_command();
            }

            if !self.stdout.is_empty() {
                ui.collapsing("Stdout", |ui| {
                    let mut text = self.stdout.join("\n");
                    ui.add(egui::TextEdit::multiline(&mut text).desired_width(f32::INFINITY));
                });
            }

            if !self.stderr.is_empty() {
                ui.collapsing("Stderr", |ui| {
                    let mut text = self.stderr.join("\n");
                    ui.add(egui::TextEdit::multiline(&mut text).desired_width(f32::INFINITY));
                });
            }

            if !self.class_load_events.is_empty() {
                ui.collapsing("Class load events", |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for class_load_event in &self.class_load_events {
                            let timestamp: DateTime<Utc> =
                                DateTime::from_timestamp_millis(class_load_event.timestamp)
                                    .unwrap();
                            ui.horizontal(|ui| {
                                ui.label(timestamp.to_rfc3339());
                                ui.label(
                                    RichText::new(&class_load_event.name).color(Color32::WHITE),
                                );
                            });
                        }
                    });
                });
            }
        });
    }
}
