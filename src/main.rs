mod legends;
mod ui;

use ui::App;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--dump") {
        match hekit::dump::run() {
            Ok(text) => {
                println!("{text}");
                return Ok(());
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }
    if args.iter().any(|arg| arg == "--check") {
        println!("hekit {}", env!("CARGO_PKG_VERSION"));
        let board = hekit::board::Board::default();
        let profile =
            hekit::profile::to_pretty(&board).expect("el perfil por defecto se serializa");
        let _ = hekit::profile::parse_document(&profile).expect("el perfil por defecto se lee");
        println!(
            "perfil JSON de {} teclas listo",
            hekit::layout::keys().len()
        );
        return Ok(());
    }
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([720.0, 520.0])
            .with_title("hekit — MINI 60 HE PRO"),
        ..Default::default()
    };
    eframe::run_native("hekit", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
