use env_logger::Env;

fn main() -> anyhow::Result<()> {
  env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
  hello_tcp::run()
}
