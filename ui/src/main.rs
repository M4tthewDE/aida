use core::f32;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use eframe::egui;
use ipc_channel::ipc::IpcOneShotServer;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let (server, server_name): (IpcOneShotServer<shared::AgentMessage>, String) =
        IpcOneShotServer::new().unwrap();

    std::thread::spawn(|| {
        let (rx, msg) = server.accept().unwrap();
        dbg!(&msg);

        if matches!(msg, shared::AgentMessage::Unload) {
            return;
        }

        loop {
            let msg = rx.recv().unwrap();
            dbg!(&msg);
            match msg {
                shared::AgentMessage::Unload => break,
                _ => {}
            }
        }
    });

    eframe::run_native(
        "Confirm exit",
        options,
        Box::new(|_cc| Ok(Box::new(App::new(server_name)))),
    )
    .unwrap();
}

struct App {
    server_name: String,
    command: String,
    stdout: Vec<String>,
}

impl App {
    fn new(server_name: String) -> Self {
        Self {
            server_name,
            command: "java -jar agent/jars/hello_world.jar".to_owned(),
            stdout: Vec::new(),
        }
    }

    fn run_command(&mut self) {
        let mut parts = self.command.split_whitespace();
        let program = parts.next().expect("no command");

        let agent_path = format!("-agentpath:target/debug/libaida.so={}", self.server_name);

        let mut args = vec![agent_path.as_str()];
        args.extend(parts);

        let mut cmd = Command::new(program)
            .args(&args)
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute");

        let stdout = cmd.stdout.take().expect("faield to capture stdout");
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            self.stdout.push(line.unwrap());
        }

        cmd.wait().expect("failed to wait on command");
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.text_edit_singleline(&mut self.command);

            if ui.button("Run").clicked() {
                self.run_command();
            }

            if !self.stdout.is_empty() {
                ui.separator();
                ui.heading("Stdout");

                let mut text = self.stdout.join("\n");
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .desired_width(f32::INFINITY)
                        .interactive(true),
                );
            }
        });
    }
}
