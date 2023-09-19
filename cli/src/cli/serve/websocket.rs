use crate::cli::trace::WS_CONNECTION;
use actix::AsyncContext;
use actix::{Actor, Handler, Message, StreamHandler};
use actix_web_actors::ws;

#[derive(Message)]
#[rtype(result = "()")]
pub struct TraceEvent(pub String);

pub struct MyWs;

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        *WS_CONNECTION.lock().unwrap() = Some(ctx.address());
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(ping)) => {
                ctx.pong(&ping);
            }
            Ok(ws::Message::Text(text)) => {
                ctx.text(text);
            }
            _ => (),
        }
    }
}

impl Handler<TraceEvent> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: TraceEvent, ctx: &mut Self::Context) -> Self::Result {
        // Do something with msg.0, which will be the String
        // For instance, you can send it to the connected WebSocket client:
        ctx.text(msg.0);
    }
}

pub async fn ws_index(
    r: actix_web::HttpRequest,
    stream: actix_web::web::Payload,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    ws::start(MyWs {}, &r, stream)
}
