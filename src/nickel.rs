use std::fmt::Display;
use std::path::Path;
use std::net::ToSocketAddrs;
use router::{Router, HttpRouter, Matcher};
use middleware::{MiddlewareStack, Middleware, ErrorHandler};
use server::Server;
use hyper::method::Method;
use hyper::status::StatusCode;

//pre defined middleware
use default_error_handler::DefaultErrorHandler;

/// Nickel is the application object. It's the surface that
/// holds all public APIs.
pub struct Nickel{
    middleware_stack: MiddlewareStack,
}

impl HttpRouter for Nickel {
    fn add_route<M: Into<Matcher>, H: Middleware>(&mut self, method: Method, matcher: M, handler: H) {
        let mut router = Router::new();
        router.add_route(method, matcher, handler);
        self.utilize(router);
    }
}

impl Nickel {
    /// Creates an instance of Nickel with default error handling.
    pub fn new() -> Nickel {
        let mut middleware_stack = MiddlewareStack::new();

        // Hook up the default error handler by default. Users are
        // free to cancel it out from their custom error handler if
        // they don't like the default behaviour.
        middleware_stack.add_error_handler(DefaultErrorHandler);

        Nickel { middleware_stack: middleware_stack }
    }

    /// Registers a middleware handler which will be invoked among other middleware
    /// handlers before each request. Middleware can be stacked and is invoked in the
    /// same order it was registered.
    ///
    /// A middleware handler is nearly identical to a regular route handler with the only
    /// difference that it expects a result of either Action or NickelError.
    /// That is to indicate whether other middleware handlers (if any) further
    /// down the stack should continue or if the middleware invocation should
    /// be stopped after the current handler.
    ///
    /// # Examples
    /// ```{rust}
    /// # #[macro_use] extern crate nickel;
    /// # fn main() {
    /// use nickel::Nickel;
    /// let mut server = Nickel::new();
    /// server.utilize(middleware! { |req|
    ///     println!("logging request: {:?}", req.origin.uri);
    /// });
    /// # }
    /// ```
    pub fn utilize<T: Middleware>(&mut self, handler: T){
        self.middleware_stack.add_middleware(handler);
    }

    /// Registers an error handler which will be invoked among other error handler
    /// as soon as any regular handler returned an error
    ///
    /// A error handler is nearly identical to a regular middleware handler with the only
    /// difference that it takes an additional error parameter or type `NickelError.
    ///
    /// # Examples
    ///
    /// ```{rust}
    /// # extern crate nickel;
    /// # fn main() {
    /// use std::io::Write;
    /// use nickel::{Nickel, Request, Response, Continue, Halt};
    /// use nickel::{NickelError, Action};
    /// use nickel::status::StatusCode::NotFound;
    ///
    /// fn error_handler(err: &mut NickelError, req: &mut Request) -> Action {
    ///    if let Some(ref mut res) = err.stream {
    ///        if res.status() == NotFound {
    ///            let _ = res.write_all(b"<h1>Call the police!</h1>");
    ///            return Halt(())
    ///        }
    ///    }
    ///
    ///     Continue(())
    /// }
    ///
    /// let mut server = Nickel::new();
    ///
    /// let ehandler: fn(&mut NickelError, &mut Request) -> Action = error_handler;
    ///
    /// server.handle_error(ehandler)
    /// # }
    /// ```
    pub fn handle_error<T: ErrorHandler>(&mut self, handler: T){
        self.middleware_stack.add_error_handler(handler);
    }

    /// Create a new middleware to serve as a router.
    ///
    ///
    /// # Examples
    /// ```{rust}
    /// #[macro_use] extern crate nickel;
    /// use nickel::{Nickel, HttpRouter};
    ///
    /// fn main() {
    ///     let mut server = Nickel::new();
    ///     let mut router = Nickel::router();
    ///
    ///     router.get("/foo", middleware! {
    ///         "Hi from /foo"
    ///     });
    ///
    ///     server.utilize(router);
    /// }
    /// ```
    pub fn router() -> Router {
        Router::new()
    }

    /// Bind and listen for connections on the given host and port.
    ///
    /// # Panics
    ///
    /// Panics if `addr` is an invalid `SocketAddr`.
    ///
    /// # Examples
    /// ```{rust,no_run}
    /// use nickel::Nickel;
    ///
    /// let mut server = Nickel::new();
    /// server.listen("127.0.0.1:6767");
    /// ```
    pub fn listen<T: ToSocketAddrs>(mut self, addr: T) {
        self.middleware_stack.add_middleware(middleware! {
            (StatusCode::NotFound, "File Not Found")
        });

        let server = Server::new(self.middleware_stack);
        let listener = server.serve(addr).unwrap();

        println!("Listening on http://{}", listener.socket);
        println!("Ctrl-C to shutdown server");
    }
    
    pub fn listen_https<T: ToSocketAddrs + Display>(mut self, addr: T, cert: &Path, key: &Path) {
        self.middleware_stack.add_middleware(middleware! {
            (StatusCode::NotFound, "File Not Found")
        });

        let listener = Server::new(self.middleware_stack).serve_https(addr, cert, key).unwrap();
        
        println!("Listening on https://{}", listener.socket);
        println!("Ctrl-C to shutdown server");

    }
    
    pub fn listen_with_threads<T: ToSocketAddrs>(mut self, addr: T, n: usize) {
        self.middleware_stack.add_middleware(middleware! {
            (StatusCode::NotFound, "File Not Found")
        });

        let server = Server::new(self.middleware_stack);
        let listener = server.serve_with_threads(addr, n).unwrap();

        println!("Listening on http://{}", listener.socket);
        println!("Ctrl-C to shutdown server");
    }
    
    pub fn listen_https_with_threads<T: ToSocketAddrs + Display>(mut self, addr: T, cert: &Path, key: &Path, n: usize) {
        self.middleware_stack.add_middleware(middleware! {
            (StatusCode::NotFound, "File Not Found")
        });

        let listener = Server::new(self.middleware_stack).serve_https_with_threads(addr, cert, key, n).unwrap();
        
        println!("Listening on https://{}", listener.socket);
        println!("Ctrl-C to shutdown server");

    }
}

#[test]
#[should_panic]
fn invalid_listen_addr() {
    Nickel::new().listen("127.0.0.1.6667");
}
