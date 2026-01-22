use core::f32;
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    sync::mpsc::{Receiver, Sender, TryRecvError},
};

use chrono::{DateTime, Utc};
use eframe::egui::{self, Color32, RichText};
use ipc_channel::ipc::IpcOneShotServer;

fn main() {
    let config_arg = std::env::args().nth(1).unwrap();
    let config_path = PathBuf::from(config_arg.clone());
    let config = shared::load_config(config_path);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Confirm exit",
        options,
        Box::new(|_cc| Ok(Box::new(App::new(config, config_arg)))),
    )
    .unwrap();
}

struct App {
    rx: Receiver<shared::AgentMessage>,
    tx: Sender<shared::AgentMessage>,
    config: shared::Config,
    config_arg: String,
    stdout: Vec<String>,
    stderr: Vec<String>,
    class_load_events: Vec<shared::ClassLoadEvent>,
    method_events: Vec<shared::MethodEvent>,
    running_command: bool,
    done_command: bool,
}

impl App {
    fn new(config: shared::Config, config_arg: String) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            rx,
            tx,
            config,
            config_arg,
            stdout: Vec::new(),
            stderr: Vec::new(),
            class_load_events: Vec::new(),
            method_events: Vec::new(),
            running_command: false,
            done_command: false,
        }
    }

    fn run_command(&mut self) {
        let (server, server_name): (IpcOneShotServer<shared::AgentMessage>, String) =
            IpcOneShotServer::new().unwrap();

        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let (rx, msg) = server.accept().unwrap();

            if matches!(msg, shared::AgentMessage::Unload) {
                tx.send(msg).unwrap();
                return;
            } else {
                tx.send(msg).unwrap();
            }

            loop {
                let msg = rx.recv().unwrap();
                match msg {
                    shared::AgentMessage::Unload => {
                        tx.send(msg).unwrap();
                        break;
                    }
                    msg => tx.send(msg).unwrap(),
                }
            }
        });

        let agent_path = format!(
            "-agentpath:target/debug/libaida.so={},{}",
            server_name, self.config_arg
        );

        let args = vec![agent_path.as_str(), "-jar", &self.config.jar];

        self.running_command = true;

        let mut cmd = Command::new("java")
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
                    shared::AgentMessage::MethodEvent(event) => self.method_events.push(event),
                    shared::AgentMessage::Unload => {
                        self.running_command = false;
                        self.done_command = true;
                    }
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
        if !self.done_command && !self.running_command {
            self.run_command();
        }

        self.receive_agent_msg(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Aida");
                if self.running_command {
                    ui.label(RichText::new("Running...").color(Color32::YELLOW));
                }

                if self.done_command {
                    ui.label(RichText::new("Done").color(Color32::GREEN));
                }
            });

            if !self.stdout.is_empty() {
                egui::CollapsingHeader::new("Stdout")
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut text = self.stdout.join("\n");
                        ui.add(egui::TextEdit::multiline(&mut text).desired_width(f32::INFINITY));
                    });
            }

            if !self.stderr.is_empty() {
                egui::CollapsingHeader::new("Stderr")
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut text = self.stderr.join("\n");
                        ui.add(egui::TextEdit::multiline(&mut text).desired_width(f32::INFINITY));
                    });
            }

            if !self.class_load_events.is_empty() {
                egui::CollapsingHeader::new("Class load events")
                    .default_open(true)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                for class_load_event in &self.class_load_events {
                                    let timestamp: DateTime<Utc> =
                                        DateTime::from_timestamp_micros(class_load_event.timestamp)
                                            .unwrap();
                                    ui.horizontal(|ui| {
                                        ui.label(timestamp.to_rfc3339());
                                        ui.label(
                                            RichText::new(&class_load_event.name)
                                                .color(Color32::WHITE),
                                        );
                                    });
                                }
                            });
                    });
            }

            if !self.method_events.is_empty() {
                egui::CollapsingHeader::new("Method events")
                    .default_open(true)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                for method_event in &self.method_events {
                                    let timestamp: DateTime<Utc> =
                                        DateTime::from_timestamp_micros(method_event.timestamp())
                                            .unwrap();
                                    ui.horizontal(|ui| {
                                        ui.label(timestamp.to_rfc3339());
                                        match method_event {
                                            shared::MethodEvent::Entry { .. } => {
                                                ui.label(RichText::new("->").color(Color32::GREEN))
                                            }
                                            shared::MethodEvent::Exit { .. } => {
                                                ui.label(RichText::new("<-").color(Color32::RED))
                                            }
                                        };
                                        ui.label(
                                            RichText::new(method_event.class_name())
                                                .color(Color32::WHITE),
                                        );
                                        ui.label(
                                            RichText::new(method_event.name())
                                                .color(Color32::WHITE),
                                        );
                                    });
                                }
                            });
                    });
            }
        });
    }
}
