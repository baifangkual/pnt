mod app;

/// pnt bin run
fn main() -> anyhow::Result<()> {
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    app::pnt_run()
}
