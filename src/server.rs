use std::net::ToSocketAddrs;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use hyper::Result as HttpResult;
use hyper::server::{Request, Response, Handler, Listening};
use hyper::server::Server as HyperServer;

use middleware::MiddlewareStack;
use request;
use response;

pub struct Server {
    middleware_stack: MiddlewareStack,
    templates: response::TemplateCache
}

// FIXME: Any better coherence solutions?
struct ArcServer(Arc<Server>);

impl Handler for ArcServer {
    fn handle<'a, 'k>(&'a self, req: Request<'a, 'k>, res: Response<'a>) {
        let nickel_req = request::Request::from_internal(req);
        let nickel_res = response::Response::from_internal(res, &self.0.templates);

        self.0.middleware_stack.invoke(nickel_req, nickel_res);
    }
}

impl Server {
    pub fn new(middleware_stack: MiddlewareStack) -> Server {
        Server {
            middleware_stack: middleware_stack,
            templates: RwLock::new(HashMap::new())
        }
    }

    pub fn serve<T: ToSocketAddrs>(self, addr: T) -> HttpResult<Listening> {
        let arc = ArcServer(Arc::new(self));
        let server = HyperServer::http(arc);
        server.listen(addr)
    }
    
    pub fn serve_https<T: ToSocketAddrs>(self, addr: T, cert: &Path, key: &Path) -> HttpResult<Listening> {
        let arc = ArcServer(Arc::new(self));
        let server = HyperServer::https(arc, cert, key);
        server.listen(addr)
    }
    
    pub fn serve_with_threads<T: ToSocketAddrs>(self, addr: T, n: usize) -> HttpResult<Listening> {
        let arc = ArcServer(Arc::new(self));
        let server = HyperServer::http(arc);
        server.listen_threads(addr, n)
    }
    
    pub fn serve_https_with_threads<T: ToSocketAddrs>(self, addr: T, cert: &Path, key: &Path, n: usize) -> HttpResult<Listening>  {
        let arc = ArcServer(Arc::new(self));
        let server = HyperServer::https(arc, cert, key);
        server.listen_threads(addr, n)
    }
}
