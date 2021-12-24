use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::create_dir;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Once, RwLock, RwLockReadGuard};
use std::task::{Context, Poll};
use std::time::Duration;

use log::{error, info};
use rusqlite::{Connection, OpenFlags};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::oneshot::{channel, Sender};
use tokio_native_tls::native_tls::{Identity, Protocol};
use tokio_native_tls::TlsAcceptor;
use ws_jsonrpc::handler::Handler;
use ws_jsonrpc::ws::request::Request;
use ws_jsonrpc::ws::response::{Response, NOT_FOUND, OK};
use ws_jsonrpc::ws::websocket::WebSocket;

use crate::db::setup;
use crate::service::methods;

static mut SERVER: Option<RwLock<Server>> = None;

fn server() -> &'static RwLock<Server> {
    static INIT: Once = Once::new();
    INIT.call_once(|| unsafe { SERVER = Some(RwLock::new(Server::new())) });
    unsafe { SERVER.as_ref().unwrap() }
}

fn reset_server() {
    *server().write().unwrap() = Server::new();
}

#[derive(Debug)]
enum Message {
    // 监听 0.0.0.0，回复端口号
    ListenAnyAddr(Sender<crate::Result<u16>>),

    // 停止监听 0.0.0.0
    CloseAnyAddr,

    // 获取 0.0.0.0 端口号
    QueryAnyAddrPort(Sender<Option<io::Result<u16>>>),

    // 退出
    #[allow(dead_code)]
    Shutdown,
}

struct Server {
    addr: Option<SocketAddr>,
    channel: Option<UnboundedSender<Message>>,
    db: Option<Connection>,
}

impl Server {
    pub const fn new() -> Server {
        Server {
            addr: None,
            channel: None,
            db: None,
        }
    }
}

#[derive(Default)]
struct NetworkServer {
    listener: Option<TcpListener>,
    acceptor: Option<Arc<TlsAcceptor>>,
}

impl NetworkServer {
    async fn create() -> crate::Result<Self> {
        let acceptor = create_tls_acceptor()?;
        let listener = TcpListener::bind("0.0.0.0:0").await.map_err(err!())?;
        let addr = listener.local_addr().map_err(err!())?;
        info!("network server started at {}", addr);
        Ok(Self {
            listener: Some(listener),
            acceptor: Some(acceptor),
        })
    }

    fn accept(&self) -> Accept {
        Accept(&self.listener)
    }

    fn port(&self) -> Option<io::Result<u16>> {
        Some(self.listener.as_ref()?.local_addr().map(|v| v.port()))
    }
}

struct Accept<'a>(&'a Option<TcpListener>);

impl<'a> Future for Accept<'a> {
    type Output = io::Result<(TcpStream, SocketAddr)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0 {
            Some(ref listener) => listener.poll_accept(cx),
            None => Poll::Pending,
        }
    }
}

pub async fn start(
    addr: impl ToSocketAddrs,
    data_dir: &str,
    on_started: impl FnOnce(SocketAddr),
) -> crate::Result<()> {
    let mut guard = server().write().unwrap();
    let server = &mut *guard;
    match server.addr {
        Some(addr) => on_started(addr),
        _ => {
            let mut sig_int = signal(SignalKind::interrupt()).map_err(err!())?;
            let mut sig_term = signal(SignalKind::terminate()).map_err(err!())?;
            let (tx, mut rx) = unbounded_channel();

            let db = init_database(data_dir)?;
            let listener = TcpListener::bind(addr).await.map_err(err!())?;
            let addr = listener.local_addr().map_err(err!())?;
            info!("server started at {}", addr);

            server.addr = Some(addr);
            server.channel = Some(tx);
            server.db = Some(db);
            drop(guard);
            on_started(addr);

            let mut network_server = NetworkServer::default();
            let handler = create_handler();
            loop {
                tokio::select! {
                    accept = listener.accept() => {
                        match accept {
                            Ok((stream, addr)) => {
                                let handler = Arc::clone(&handler);
                                tokio::spawn(async move {
                                    if let Err(err) = handle_client(stream, &handler).await {
                                        error!("{} {:?}", addr, err);
                                    }
                                });
                            }
                            Err(err) => error!("{:?}", err!(err)),
                        }
                    }
                    accept = network_server.accept() => {
                        match accept {
                            Ok((stream, addr)) => {
                                let handler = Arc::clone(&handler);
                                let acceptor = Arc::clone(network_server.acceptor.as_ref().unwrap());
                                tokio::spawn(async move {
                                    match acceptor.accept(stream).await.map_err(err!()) {
                                        Ok(stream) => if let Err(err) = handle_client(stream, &handler).await {
                                            error!("{} {:?}", addr, err);
                                        }
                                        Err(err) => error!("{} {:?}", addr, err),
                                    }
                                });
                            }
                            Err(err) => error!("{:?}", err!(err)),
                        }
                    }
                    msg = rx.recv() => {
                        match msg {
                            Some(Message::ListenAnyAddr(reply)) => match network_server.port() {
                                Some(port) => {
                                    let _ = reply.send(port.map_err(err!()));
                                }
                                None => match NetworkServer::create().await {
                                    Ok(v) => {
                                        let _ = reply.send(v.port().unwrap().map_err(err!()));
                                        network_server = v;
                                    },
                                    Err(err) => {
                                        let _ = reply.send(Err(err));
                                    }
                                }
                            }
                            Some(Message::CloseAnyAddr) => {
                                network_server = NetworkServer::default();
                                info!("network server stopped");
                            }
                            Some(Message::QueryAnyAddrPort(reply)) => {
                                let _ = reply.send(network_server.port());
                            }
                            Some(Message::Shutdown) => {
                                info!("receive shutdown, stopping");
                                break;
                            }
                            None => unreachable!(),
                        }
                    }
                    _ = sig_int.recv() => {
                        info!("catch SIGINT, stopping");
                        break;
                    }
                    _ = sig_term.recv() => {
                        info!("catch SIGTERM, stopping");
                        break;
                    }
                }
            }
            reset_server();
        }
    }
    Ok(())
}

pub async fn query_network_port() -> crate::Result<Option<u16>> {
    let rx = match server().read().unwrap().channel {
        Some(ref sender) => {
            let (tx, rx) = channel();
            sender.send(Message::QueryAnyAddrPort(tx)).map_err(err!())?;
            rx
        }
        None => return Err(err!(Unavailable)),
    };
    match rx.await.map_err(err!())? {
        Some(port) => Ok(Some(port.map_err(err!())?)),
        None => Ok(None),
    }
}

pub async fn listen_any_addr() -> crate::Result<u16> {
    let rx = match server().read().unwrap().channel {
        Some(ref sender) => {
            let (tx, rx) = channel();
            sender.send(Message::ListenAnyAddr(tx)).map_err(err!())?;
            rx
        }
        None => return Err(err!(Unavailable)),
    };
    rx.await.map_err(err!())?
}

pub fn close_any_addr() -> crate::Result<()> {
    match server().read().unwrap().channel {
        Some(ref sender) => sender.send(Message::CloseAnyAddr).map_err(err!()),
        None => Err(err!(Unavailable)),
    }
}

#[allow(dead_code)]
pub fn stop() {
    match server().write().unwrap().channel {
        Some(ref sender) => {
            let _ = sender.send(Message::Shutdown);
        }
        None => {}
    }
}

fn create_tls_acceptor() -> crate::Result<Arc<TlsAcceptor>> {
    let identity = Identity::from_pkcs12(include_bytes!("vault.pfx"), "vault").map_err(err!())?;
    let acceptor = tokio_native_tls::native_tls::TlsAcceptor::builder(identity)
        .min_protocol_version(Some(Protocol::Tlsv10))
        .build()
        .map_err(err!())?;
    Ok(Arc::new(acceptor.into()))
}

#[derive(Debug)]
pub struct Unavailable;

impl Display for Unavailable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt("Unavailable", f)
    }
}

impl Error for Unavailable {}

pub struct DB<'a>(RwLockReadGuard<'a, Server>);

pub fn db() -> DB<'static> {
    DB(server().read().unwrap())
}

impl<'a> DB<'a> {
    pub fn conn(&self) -> Result<&Connection, Unavailable> {
        self.0.db.as_ref().ok_or(Unavailable)
    }
}

fn init_database(data_dir: &str) -> crate::Result<Connection> {
    let mut path = PathBuf::from(data_dir);
    if !path.exists() {
        create_dir(&path).map_err(err!())?;
    }
    path.push("database");

    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_CREATE
        | OpenFlags::SQLITE_OPEN_FULL_MUTEX;
    let mut db = Connection::open_with_flags(path, flags).map_err(err!())?;
    setup(&mut db)?;
    Ok(db)
}

fn create_handler() -> Arc<Handler> {
    let mut handler = Handler::new();
    handler.register(methods());
    Arc::new(handler)
}

const TIMEOUT: Duration = Duration::from_secs(60);

async fn handle_client(
    mut stream: impl AsyncRead + AsyncWrite + Unpin,
    handler: &Arc<Handler>,
) -> crate::Result<()> {
    let mut buf = vec![0u8; 1024];
    let req = Request::new(&mut stream, &mut buf, TIMEOUT)
        .await
        .map_err(err!())?;
    match req.uri().split('?').next().unwrap() {
        "/" => {
            let mut response = Response::bytes(OK, include_bytes!("../html/dist/index.html"));
            response.add_header("content-type", "text/html");
            response.write(&mut stream).await.map_err(err!())?;
        }
        "/js/app.js" => {
            let mut response = Response::bytes(OK, include_bytes!("../html/dist/js/app.js"));
            response.add_header("content-type", "text/javascript");
            response.write(&mut stream).await.map_err(err!())?;
        }
        "/ws" => match WebSocket::upgrade(&req, stream).await.map_err(err!())? {
            Some(ws) => handler.handle(ws).await.map_err(err!())?,
            None => {}
        },
        _ => {
            let response = Response::status(NOT_FOUND);
            response.write(&mut stream).await.map_err(err!())?;
        }
    }
    Ok(())
}
