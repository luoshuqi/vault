use std::fmt::Display;
use std::net::SocketAddr;
use std::panic::catch_unwind;

use android_logger::Config;
use jni::objects::{JClass, JObject, JString};
use jni::JNIEnv;
use log::{error, Level};
use tokio::runtime::Runtime;

use crate::server::{start as start_server, stop as stop_server};

#[no_mangle]
pub unsafe extern "system" fn Java_pub_trait_vault_Vault_start(
    env: JNIEnv,
    class: JClass,
    data_dir: JString,
    callback: JObject,
) {
    match catch_unwind(|| match start(env, class, data_dir, callback) {
        Ok(()) => {}
        Err(err) => {
            error!("{:?}", err);
            throw(env, err).unwrap();
        }
    }) {
        Ok(()) => {}
        Err(_) => {
            error!("panic");
            let _ = throw(env, "panic");
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_pub_trait_vault_Vault_stop(env: JNIEnv, _class: JClass) {
    match catch_unwind(|| stop_server()) {
        Ok(()) => {}
        Err(_) => {
            error!("panic");
            let _ = throw(env, "panic");
        }
    }
}

unsafe fn start(
    env: JNIEnv,
    _class: JClass,
    data_dir: JString,
    callback: JObject,
) -> crate::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("{}", info);
        error!("{}", info);
    }));

    init_logger();

    let on_started = |addr| match invoke_callback(env, addr, callback) {
        Ok(()) => {}
        Err(err) => {
            error!("{:?}", err);
            throw(env, err).unwrap();
        }
    };

    let data_dir = env.get_string(data_dir).map_err(err!())?;
    let data_dir = data_dir.to_str().map_err(err!())?;
    runtime().block_on(start_server("127.0.0.1:0", data_dir, on_started))
}

fn invoke_callback(env: JNIEnv, addr: SocketAddr, callback: JObject) -> crate::Result<()> {
    let addr = env.new_string(format!("http://{}", addr)).map_err(err!())?;
    env.call_method(
        callback,
        "apply",
        "(Ljava/lang/Object;)Ljava/lang/Object;",
        &[addr.into()],
    )
    .map_err(err!())?;
    Ok(())
}

fn runtime() -> Runtime {
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(err!())
    {
        Ok(runtime) => runtime,
        Err(err) => {
            error!("{:?}", err);
            panic!("{:?}", err);
        }
    }
}

fn throw(env: JNIEnv, error: impl Display) -> crate::Result<()> {
    if env.exception_check().map_err(err!())? {
        return Ok(());
    }

    match env
        .throw_new("java/lang/RuntimeException", &format!("{}", error))
        .map_err(err!())
    {
        Ok(()) => Ok(()),
        Err(err) => {
            error!("throw failed: {:?}", err);
            Err(err)
        }
    }
}

pub fn init_logger() {
    #[cfg(debug_assertions)]
    let level = Level::Debug;

    #[cfg(not(debug_assertions))]
    let level = Level::Info;

    android_logger::init_once(Config::default().with_min_level(level));
}
