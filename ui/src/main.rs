use std::process::Command;

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
}

impl App {
    fn new(server_name: String) -> Self {
        Self {
            server_name,
            command: "java -jar agent/jars/hello_world.jar".to_owned(),
        }
    }

    fn run_command(&mut self) {
        let mut parts = self.command.split_whitespace();
        let program = parts.next().expect("nbo command");

        let agent_path = format!("-agentpath:target/debug/libaida.so={}", self.server_name);

        let mut args = vec![agent_path.as_str()];
        args.extend(parts);

        Command::new(program)
            .args(&args)
            .status()
            .expect("failed to execute");
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.text_edit_singleline(&mut self.command);

            if ui.button("Run").clicked() {
                self.run_command();
            }
        });
    }
}
