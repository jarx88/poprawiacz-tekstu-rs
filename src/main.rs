use poprawiacz_tekstu_rs::app::MultiAPICorrector;
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("poprawiacz_tekstu_rs=info")))
        .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("PoprawiaczTekstuRs - Multi-API Text Corrector")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "PoprawiaczTekstuRs",
        native_options,
        Box::new(|cc| Ok(Box::new(MultiAPICorrector::new(cc)))),
    )?;

    Ok(())
}

fn load_icon() -> egui::IconData {
    let size = 32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    
    for _ in 0..(size * size) {
        rgba.push(16);
        rgba.push(163);
        rgba.push(127);
        rgba.push(255);
    }

    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_framework_works() {
        assert!(true);
    }
}
