use ai_pcb::{AppState, openai::configured_engine, repository::Repository, router, seed};
use std::{env, error::Error, net::SocketAddr};

fn main() -> Result<(), Box<dyn Error>> {
    match dotenvy::from_path(".env") {
        Ok(_) => {}
        Err(dotenvy::Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "无法读取或解析 .env，请检查文件格式；未输出文件内容以保护密钥。",
            )
            .into());
        }
    }
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(serve())
}

async fn serve() -> Result<(), Box<dyn Error>> {
    let engine = configured_engine()?;
    let engine_info = engine.info();
    let address: SocketAddr = env::var("AI_PCB_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".into())
        .parse()?;
    if !address.ip().is_loopback() {
        return Err("运行时设置没有账号鉴权，服务仅允许绑定本机回环地址。".into());
    }
    let directory = env::var("AI_PCB_DATA_DIR").unwrap_or_else(|_| "data/projects".into());
    let repository = Repository::open(&directory)?;
    seed(&repository)?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!(
        "\n  AI-PCB · 让想法，连接现实。\n  http://{}\n  数据目录：{directory}\n  生成引擎：{} / {}\n  参考布局 · 未工程验证 · 不可投产\n",
        listener.local_addr()?,
        engine_info.provider,
        engine_info.model.as_deref().unwrap_or("local templates")
    );
    if let Some(origin) = engine_info.remote_origin {
        println!("  API 目标：{origin}（已配置，尚未验证连通性；需网页授权才发送需求）\n");
    }
    axum::serve(listener, router(AppState::new(repository, engine))).await?;
    Ok(())
}
